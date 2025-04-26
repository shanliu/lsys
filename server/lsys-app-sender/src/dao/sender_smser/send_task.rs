use std::{
    collections::HashMap,
    sync::{atomic::AtomicU32, Arc},
};

use async_trait::async_trait;
use lsys_core::{db::WhereOption, fluent_message, now_time, IntoFluentMessage};

use crate::{
    dao::{
        group_exec, MessageLogs, MessageReader, SenderError, SenderExecError, SenderResult,
        SenderTaskAcquisition, SenderTaskData, SenderTaskExecutor, SenderTaskExecutorBox,
        SenderTaskItem, SenderTaskResultItem, SenderTaskStatus, SenderTplConfig, SenderWaitNotify,
    },
    model::{
        SenderLogStatus, SenderMessageCancelModel, SenderSmsBodyModel, SenderSmsBodyModelRef,
        SenderSmsBodyStatus, SenderSmsMessageModel, SenderSmsMessageStatus, SenderTplConfigModel,
    },
};
use lsys_core::db::{ModelTableName, SqlExpr, Update};
use lsys_core::sql_format;
use lsys_core::{TaskAcquisition, TaskData, TaskExecutor, TaskItem, TaskRecord};
use lsys_setting::model::SettingModel;
use sqlx::Pool;
use tracing::warn;

use super::SmsRecord;
use lsys_core::db::SqlQuote;

pub struct SmsTaskItem {
    pub sms: SenderSmsBodyModel,
}

impl TaskItem<u64> for SmsTaskItem {
    fn to_task_pk(&self) -> u64 {
        self.sms.id
    }
}
impl SenderTaskItem<u64> for SmsTaskItem {
    fn app_id(&self) -> u64 {
        self.sms.app_id
    }
    fn tpl_id(&self) -> String {
        self.sms.tpl_id.to_owned()
    }
}

pub struct SmsTaskData {
    pub(crate) data: Vec<SenderSmsMessageModel>,
}
impl SenderTaskData for SmsTaskData {
    fn to_pks(&self) -> Vec<u64> {
        self.data.iter().map(|e| e.id).collect()
    }
}

impl SmsTaskData {
    pub fn new(data: Vec<SenderSmsMessageModel>) -> Self {
        Self { data }
    }
}

#[async_trait]
pub trait SmsTaskExecutor: Sync + Send + 'static {
    async fn exec(
        &self,
        val: &SmsTaskItem,
        tpl_config: &SenderTplConfigModel,
        setting: &SettingModel,
        record: &SmsRecord,
    ) -> Result<(), SenderExecError>;
    fn setting_key(&self) -> String;
}

pub struct SmsTaskAcquisition {
    db: Pool<sqlx::MySql>,
    wait_notify: Arc<SenderWaitNotify>,
    message_logs: Arc<MessageLogs>,
    message_reader: Arc<MessageReader<SenderSmsBodyModel, SenderSmsMessageModel>>,
}
impl SmsTaskAcquisition {
    pub(crate) fn new(
        db: Pool<sqlx::MySql>,
        wait_notify: Arc<SenderWaitNotify>,
        message_logs: Arc<MessageLogs>,
        message_reader: Arc<MessageReader<SenderSmsBodyModel, SenderSmsMessageModel>>,
    ) -> Self {
        Self {
            db,
            wait_notify,
            message_logs,
            message_reader,
        }
    }

    async fn cancel_data_ids(&self, record: &SmsTaskData) -> Vec<u64> {
        let msg_id = record.data.iter().map(|e| e.id).collect::<Vec<_>>();
        let cancel_sql = sql_format!(
            "select sender_message_id from {} where sender_message_id in ({})",
            SenderMessageCancelModel::table_name(),
            msg_id
        );
        match sqlx::query_scalar::<_, u64>(&cancel_sql)
            .fetch_all(&self.db)
            .await
        {
            Ok(d) => d,
            Err(err) => {
                warn!("select cancel data fail:{}", err);
                vec![]
            }
        }
    }

    async fn send_record_clear(&self, item: &SmsTaskItem) {
        let sql = sql_format!(
            "select id from {} where sender_body_id={} and status={} limit 1",
            SenderSmsMessageModel::table_name(),
            item.sms.id,
            SenderSmsMessageStatus::Init as i8
        );
        if let Err(err) = sqlx::query_scalar::<_, u64>(&sql).fetch_one(&self.db).await {
            match err {
                sqlx::Error::RowNotFound => self.send_task_body_finish(item.sms.id).await,
                _ => {
                    warn!("sms finish task ,check status fail{}", err)
                }
            }
        }
    }
    async fn send_task_body_finish(&self, item_id: u64) {
        let finish_time = now_time().unwrap_or_default();
        let change = lsys_core::model_option_set!(SenderSmsBodyModelRef,{
            status:SenderSmsBodyStatus::Finish as i8,
            finish_time:finish_time
        });
        if let Err(err) = Update::<SenderSmsBodyModel, _>::new(change)
            .execute_by_where(&WhereOption::Where(sql_format!("id={}", item_id)), &self.db)
            .await
        {
            warn!("sms change finish status fail{}", err)
        }
    }
}

#[async_trait]
impl SenderTaskAcquisition<u64, SmsTaskItem, SmsTaskData> for SmsTaskAcquisition {
    async fn read_send_record(
        &self,
        record: &SmsTaskItem,
        sending_data: &[u64],
        limit: u16,
    ) -> Result<SmsTaskData, String> {
        let app_res = self
            .message_reader
            .read_message(
                record,
                sending_data,
                SenderSmsMessageStatus::Init as i8,
                limit,
            )
            .await
            .map_err(|e| e.to_fluent_message().default_format())?;

        if app_res.is_empty() {
            let sql_where = sql_format!(
                r#"sender_body_id={} {} ;
            "#,
                record.sms.id,
                SqlExpr(if sending_data.is_empty() {
                    "".to_string()
                } else {
                    sql_format!(" and sender_message_id not in ({})", sending_data)
                })
            );
            if let Err(err) = sqlx::query(
                sql_format!(
                    r#"UPDATE {}
                    SET status={}
                    WHERE status ={} and id in (select sender_message_id from {} where {});
                "#,
                    SenderSmsMessageModel::table_name(),
                    SenderSmsMessageStatus::IsCancel as i8,
                    SenderSmsMessageStatus::Init as i8,
                    SenderMessageCancelModel::table_name(),
                    SqlExpr(sql_where),
                )
                .as_str(),
            )
            .execute(&self.db)
            .await
            {
                warn!(
                    "sms clear message cancel status fail[{}]{}",
                    record.sms.id, err
                );
            }
        }

        if sending_data.is_empty() && app_res.is_empty() {
            self.send_task_body_finish(record.sms.id).await;
        }

        Ok(SmsTaskData::new(app_res))
    }

    async fn task_send_fail(
        &self,
        item: &SmsTaskItem,
        in_task_id: &[u64],
        error: &SenderExecError,
        setting: Option<&SettingModel>,
    ) {
        self.wait_notify
            .body_notify(&item.sms.reply_host, item.sms.id, Err(error.to_string()))
            .await;
        let sql = match error {
            SenderExecError::Finish(_) => {
                sql_format!(
                    r#"UPDATE {}
                        SET try_num=try_num+1,status={}
                        WHERE sender_body_id  ={}  and status={} {};
                    "#,
                    SenderSmsMessageModel::table_name(),
                    SenderSmsMessageStatus::SendFail as i8,
                    item.sms.id,
                    SenderSmsMessageStatus::Init as i8,
                    SqlExpr(if in_task_id.is_empty() {
                        "".to_string()
                    } else {
                        sql_format!("and id not in ({})", in_task_id)
                    }),
                )
            }
            SenderExecError::Next(_) => {
                let cancel_sql = sql_format!(
                    "select sender_message_id from {} where sender_body_id ={}",
                    SenderMessageCancelModel::table_name(),
                    item.sms.id
                );
                let cancel_data = match sqlx::query_scalar::<_, u64>(&cancel_sql)
                    .fetch_all(&self.db)
                    .await
                {
                    Ok(d) => d,
                    Err(err) => {
                        warn!("select cancel data fail[all]:{}", err);
                        vec![]
                    }
                };

                sql_format!(
                    r#"UPDATE {}
                            SET try_num=try_num+1,status=if(try_num>={},{},{})
                            WHERE sender_body_id  ={} and status={} {};
                        "#,
                    SenderSmsMessageModel::table_name(),
                    item.sms.max_try_num,
                    SenderSmsMessageStatus::SendFail as i8,
                    SqlExpr(if cancel_data.is_empty() {
                        "status".to_string()
                    } else {
                        sql_format!(
                            "if ( id in ({}),{},status)",
                            cancel_data,
                            SenderSmsMessageStatus::IsCancel as i8
                        )
                    }),
                    item.sms.id,
                    SenderSmsMessageStatus::Init as i8,
                    SqlExpr(if in_task_id.is_empty() {
                        "".to_string()
                    } else {
                        sql_format!("and id not in ({})", in_task_id)
                    }),
                )
            }
        };
        if let Err(err) = sqlx::query(sql.as_str()).execute(&self.db).await {
            warn!("change finish status fail{}", err);
            return;
        }
        let msg_ids_sql = sql_format!(
            r#"select id from {} WHERE sender_body_id={} {};
        "#,
            SenderSmsMessageModel::table_name(),
            item.sms.id,
            SqlExpr(if in_task_id.is_empty() {
                "".to_string()
            } else {
                sql_format!("and id not in ({})", in_task_id)
            }),
        );
        if let Ok(id_items) = sqlx::query_scalar::<_, u64>(msg_ids_sql.as_str())
            .fetch_all(&self.db)
            .await
        {
            let err_str = error.to_string();
            let log_data = id_items
                .into_iter()
                .map(|e| (e, SenderLogStatus::Fail, err_str.as_str()))
                .collect::<Vec<_>>();
            self.message_logs
                .add_exec_log(
                    &item.app_id(),
                    &log_data,
                    setting.map(|t| t.setting_key.as_str()).unwrap_or(""),
                )
                .await;
        }
        self.send_record_clear(item).await;
    }
    async fn task_send_success(
        &self,
        setting: &SettingModel,
        item: &SmsTaskItem,
        record: &SmsTaskData,
        res_items: &[SenderTaskResultItem],
    ) {
        let cancel_data = self.cancel_data_ids(record).await;
        let mut log_data = Vec::with_capacity(res_items.len());
        for res_item in res_items {
            let sql = match res_item.status {
                SenderTaskStatus::Completed => {
                    self.wait_notify
                        .msg_notify(&item.sms.reply_host, res_item.id, Ok(true))
                        .await;

                    log_data.push((
                        res_item.id,
                        SenderLogStatus::Succ,
                        res_item.send_id.as_str(),
                    ));
                    let ntime = now_time().unwrap_or_default();
                    sql_format!(
                        r#"UPDATE {}
                            SET try_num=try_num+1,status={},res_data={},send_time={},receive_time={},setting_id={}
                            WHERE id={};
                        "#,
                        SenderSmsMessageModel::table_name(),
                        SenderSmsMessageStatus::IsReceived as i8,
                        res_item.send_id,
                        ntime,
                        ntime,
                        setting.id,
                        res_item.id,
                    )
                }
                SenderTaskStatus::Progress => {
                    self.wait_notify
                        .msg_notify(&item.sms.reply_host, res_item.id, Ok(false))
                        .await;

                    log_data.push((
                        res_item.id,
                        SenderLogStatus::Succ,
                        res_item.send_id.as_str(),
                    ));
                    let ntime = now_time().unwrap_or_default();
                    sql_format!(
                        r#"UPDATE {}
                            SET try_num=try_num+1,status={},res_data={},send_time={},setting_id={}
                            WHERE id={};
                        "#,
                        SenderSmsMessageModel::table_name(),
                        SenderSmsMessageStatus::IsSend as i8,
                        res_item.send_id,
                        ntime,
                        setting.id,
                        res_item.id,
                    )
                }
                SenderTaskStatus::Failed(retry) => {
                    log_data.push((
                        res_item.id,
                        SenderLogStatus::Fail,
                        res_item.message.as_str(),
                    ));
                    if retry {
                        sql_format!(
                            r#"UPDATE {}
                                SET try_num=try_num+1,status=if(try_num>={},{},{})
                                WHERE id={} and status={};
                            "#,
                            SenderSmsMessageModel::table_name(),
                            item.sms.max_try_num,
                            SenderSmsMessageStatus::SendFail as i8,
                            SqlExpr(if cancel_data.contains(&res_item.id) {
                                sql_format!("{}", SenderSmsMessageStatus::IsCancel as i8)
                            } else {
                                "status".to_string()
                            }),
                            res_item.id,
                            SenderSmsMessageStatus::Init as i8,
                        )
                    } else {
                        self.wait_notify
                            .msg_notify(
                                &item.sms.reply_host,
                                res_item.id,
                                Err(res_item.message.to_owned()),
                            )
                            .await;

                        sql_format!(
                            r#"UPDATE {}
                                SET try_num=try_num+1,status={}
                                WHERE id={} and status={};
                            "#,
                            SenderSmsMessageModel::table_name(),
                            SenderSmsMessageStatus::SendFail as i8,
                            res_item.id,
                            SenderSmsMessageStatus::Init as i8,
                        )
                    }
                }
            };
            if let Err(err) = sqlx::query(sql.as_str()).execute(&self.db).await {
                warn!("change message status fail[{}]{}", res_item.id, err);
                continue;
            }
        }
        self.message_logs
            .add_exec_log(&item.app_id(), &log_data, &setting.setting_key)
            .await;
        self.send_record_clear(item).await;
    }
    //完成指定短信任务回调
    async fn task_record_send_fail(
        &self,
        setting: &SettingModel,
        item: &SmsTaskItem,
        record: &SmsTaskData,
        error: &SenderExecError,
    ) {
        let fail_ids = record.data.iter().map(|e| e.id).collect::<Vec<_>>();
        for tmp in fail_ids.iter() {
            self.wait_notify
                .msg_notify(&item.sms.reply_host, *tmp, Err(error.to_string()))
                .await;
        }
        let sql = match error {
            SenderExecError::Finish(_) => {
                sql_format!(
                    r#"UPDATE {}
                    SET try_num=try_num+1,status={}
                    WHERE id in ({}) and status={};
                "#,
                    SenderSmsMessageModel::table_name(),
                    SenderSmsMessageStatus::SendFail as i8,
                    fail_ids,
                    SenderSmsMessageStatus::Init as i8,
                )
            }
            SenderExecError::Next(_) => {
                let cancel_data = self.cancel_data_ids(record).await;

                sql_format!(
                    r#"UPDATE {}
                    SET try_num=try_num+1,status=if(try_num>={},{},{})
                    WHERE id in ({}) and status={};
                "#,
                    SenderSmsMessageModel::table_name(),
                    item.sms.max_try_num,
                    SenderSmsMessageStatus::SendFail as i8,
                    SqlExpr(if cancel_data.is_empty() {
                        "status".to_string()
                    } else {
                        sql_format!(
                            "if ( id in ({}),{},status)",
                            cancel_data,
                            SenderSmsMessageStatus::IsCancel as i8
                        )
                    }),
                    fail_ids,
                    SenderSmsMessageStatus::Init as i8,
                )
            }
        };
        if let Err(err) = sqlx::query(sql.as_str()).execute(&self.db).await {
            warn!("change finish status fail{}", err);
            return;
        };
        let err_str = error.to_string();
        let log_data = record
            .data
            .iter()
            .map(|e| (e.id, SenderLogStatus::Fail, err_str.as_str()))
            .collect::<Vec<_>>();
        self.message_logs
            .add_exec_log(&item.app_id(), &log_data, &setting.setting_key)
            .await;
        self.send_record_clear(item).await;
    }
}

#[async_trait]
impl TaskAcquisition<u64, SmsTaskItem> for SmsTaskAcquisition {
    //复用父结构体方法实现
    async fn read_exec_task(
        &self,
        tasking_record: &HashMap<u64, TaskData>,
        limit: usize,
    ) -> Result<TaskRecord<u64, SmsTaskItem>, String> {
        let (app_res, next) = self
            .message_reader
            .read_task(tasking_record, SenderSmsBodyStatus::Init as i8, limit)
            .await
            .map_err(|e| e.to_fluent_message().default_format())?;
        let app_res = app_res
            .into_iter()
            .map(|e| SmsTaskItem { sms: e })
            .collect();
        Ok(TaskRecord::new(app_res, next))
    }
}

#[derive(Clone)]
pub struct SmsTask {
    inner: Arc<Vec<SenderTaskExecutorBox<u64, SmsTaskItem, SmsTaskData>>>,
    acquisition: Arc<SmsTaskAcquisition>,
    tpl_config: Arc<SenderTplConfig>,
    i: Arc<AtomicU32>,
}

impl SmsTask {
    pub fn new(
        acquisition: Arc<SmsTaskAcquisition>,
        tpl_config: Arc<SenderTplConfig>,
        se: Vec<Box<dyn SenderTaskExecutor<u64, SmsTaskItem, SmsTaskData>>>,
    ) -> SenderResult<SmsTask> {
        if se.is_empty() {
            return Err(SenderError::System(fluent_message!("sms-task-empty")));
            // return Err(SenderError::System("can't set task is empty".to_string()));
        }
        Ok(SmsTask {
            inner: Arc::new(
                se.into_iter()
                    .map(|e| (e, AtomicU32::new(0)))
                    .collect::<Vec<_>>(),
            ),
            acquisition,
            tpl_config,
            i: AtomicU32::new(0).into(),
        })
    }
}

#[async_trait]
impl TaskExecutor<u64, SmsTaskItem> for SmsTask {
    async fn exec(&self, val: SmsTaskItem) -> Result<(), String> {
        group_exec(
            self.acquisition.as_ref(),
            &val,
            &self.i,
            &self.tpl_config,
            self.inner.as_ref(),
        )
        .await
    }
}
