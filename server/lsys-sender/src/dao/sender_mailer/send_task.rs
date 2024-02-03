use crate::{
    dao::{
        group_exec, MessageLogs, MessageReader, SenderError, SenderExecError, SenderResult,
        SenderTaskAcquisition, SenderTaskData, SenderTaskExecutor, SenderTaskExecutorBox,
        SenderTaskItem, SenderTaskResultItem, SenderTaskStatus, SenderTplConfig, SenderWaitNotify,
    },
    model::{
        SenderLogStatus, SenderMailBodyModel, SenderMailBodyModelRef, SenderMailBodyStatus,
        SenderMailMessageModel, SenderMailMessageStatus, SenderMessageCancelModel,
    },
};
use async_trait::async_trait;
use lsys_core::{fluent_message, now_time};
use lsys_core::{TaskAcquisition, TaskData, TaskExecutor, TaskItem, TaskRecord};
use lsys_setting::model::SettingModel;
use sqlx_model::{ModelTableName, SqlExpr, SqlQuote};
use std::{
    collections::HashMap,
    sync::{atomic::AtomicU32, Arc},
};

use sqlx::{MySql, Pool};
use sqlx_model::{sql_format, Update};
use tracing::warn;

//短信任务记录

pub struct MailTaskItem {
    pub mail: SenderMailBodyModel,
}

impl TaskItem<u64> for MailTaskItem {
    fn to_task_pk(&self) -> u64 {
        self.mail.id
    }
}
impl SenderTaskItem<u64> for MailTaskItem {
    fn app_id(&self) -> u64 {
        self.mail.app_id
    }
    fn tpl_id(&self) -> String {
        self.mail.tpl_id.to_owned()
    }
}

pub struct MailTaskData {
    pub(crate) data: Vec<SenderMailMessageModel>,
}

impl MailTaskData {
    pub fn new(data: Vec<SenderMailMessageModel>) -> Self {
        Self { data }
    }
}

impl SenderTaskData for MailTaskData {
    fn to_pks(&self) -> Vec<u64> {
        self.data.iter().map(|e| e.id).collect()
    }
}

pub struct MailTaskAcquisition {
    db: Pool<sqlx::MySql>,
    wait_notify: Arc<SenderWaitNotify>,
    message_logs: Arc<MessageLogs>,
    message_reader: Arc<MessageReader<SenderMailBodyModel, SenderMailMessageModel>>,
}

impl MailTaskAcquisition {
    pub(crate) fn new(
        db: Pool<sqlx::MySql>,
        wait_notify: Arc<SenderWaitNotify>,
        message_logs: Arc<MessageLogs>,
        message_reader: Arc<MessageReader<SenderMailBodyModel, SenderMailMessageModel>>,
    ) -> Self {
        Self {
            db,
            wait_notify,
            message_logs,
            message_reader,
        }
    }
    async fn cancel_data_ids(&self, record: &MailTaskData) -> Vec<u64> {
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
    async fn send_record_clear(&self, item: &MailTaskItem) {
        let sql = sql_format!(
            "select id from {} where sender_body_id={} and status={} limit 1",
            SenderMailMessageModel::table_name(),
            item.mail.id,
            SenderMailMessageStatus::Init as i8
        );
        if let Err(err) = sqlx::query_scalar::<_, u64>(&sql).fetch_one(&self.db).await {
            match err {
                sqlx::Error::RowNotFound => self.send_task_body_finish(item.mail.id).await,
                _ => {
                    warn!("finish task ,check status fail{}", err)
                }
            }
        }
    }
    async fn send_task_body_finish(&self, item_id: u64) {
        let finish_time = now_time().unwrap_or_default();
        let change = sqlx_model::model_option_set!(SenderMailBodyModelRef,{
            status:SenderMailBodyStatus::Finish as i8,
            finish_time:finish_time
        });
        if let Err(err) = Update::<MySql, SenderMailBodyModel, _>::new(change)
            .execute_by_scalar_pk(item_id, &self.db)
            .await
        {
            warn!("mail change finish status fail{}", err)
        }
    }
}

#[async_trait]
impl SenderTaskAcquisition<u64, MailTaskItem, MailTaskData> for MailTaskAcquisition {
    async fn read_send_record(
        &self,
        record: &MailTaskItem,
        sending_data: &[u64],
        limit: u16,
    ) -> Result<MailTaskData, String> {
        let app_res = self
            .message_reader
            .read_message(
                record,
                sending_data,
                SenderMailMessageStatus::Init as i8,
                limit,
            )
            .await
            .map_err(|e| e.to_string())?;

        if app_res.is_empty() {
            let sql_where = sql_format!(
                r#"sender_body_id={} {} ;
            "#,
                record.mail.id,
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
                    WHERE status = {} and  id in (select sender_message_id from {} where {});
                "#,
                    SenderMailMessageModel::table_name(),
                    SenderMailMessageStatus::IsCancel as i8,
                    SenderMailMessageStatus::Init as i8,
                    SenderMessageCancelModel::table_name(),
                    SqlExpr(sql_where),
                )
                .as_str(),
            )
            .execute(&self.db)
            .await
            {
                warn!(
                    "mail clear message cancel status fail[{}]{}",
                    record.mail.id, err
                );
            }
        }

        if sending_data.is_empty() && app_res.is_empty() {
            self.send_task_body_finish(record.mail.id).await;
        }

        Ok(MailTaskData::new(app_res))
    }
    async fn task_send_fail(
        &self,
        item: &MailTaskItem,
        in_task_id: &[u64],
        error: &SenderExecError,
        setting: Option<&SettingModel>,
    ) {
        self.wait_notify
            .body_notify(&item.mail.reply_host, item.mail.id, Err(error.to_string()))
            .await;
        let sql = match error {
            SenderExecError::Finish(_) => {
                sql_format!(
                    r#"UPDATE {}
                            SET try_num=try_num+1,status={}
                            WHERE sender_body_id  ={} and status={}  {};
                        "#,
                    SenderMailMessageModel::table_name(),
                    SenderMailMessageStatus::SendFail as i8,
                    item.mail.id,
                    SenderMailMessageStatus::Init as i8,
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
                    item.mail.id
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
                    SenderMailMessageModel::table_name(),
                    item.mail.max_try_num,
                    SenderMailMessageStatus::SendFail as i8,
                    SqlExpr(if cancel_data.is_empty() {
                        "status".to_string()
                    } else {
                        sql_format!(
                            " if ( id  in ({}),{},status)",
                            cancel_data,
                            SenderMailMessageStatus::IsCancel
                        )
                    }),
                    item.mail.id,
                    SenderMailMessageStatus::Init as i8,
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
            SenderMailMessageModel::table_name(),
            item.mail.id,
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
            let log_data = id_items
                .into_iter()
                .map(|e| (e, SenderLogStatus::Fail, error.to_string()))
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
        item: &MailTaskItem,
        record: &MailTaskData,
        res_items: &[SenderTaskResultItem],
    ) {
        let cancel_data = self.cancel_data_ids(record).await;

        let mut log_data = Vec::with_capacity(res_items.len());
        for res_item in res_items {
            let sql = match res_item.status {
                SenderTaskStatus::Completed => {
                    self.wait_notify
                        .msg_notify(&item.mail.reply_host, res_item.id, Ok(true))
                        .await;

                    log_data.push((res_item.id, SenderLogStatus::Succ, res_item.send_id.clone()));
                    let ntime = now_time().unwrap_or_default();
                    sql_format!(
                        r#"UPDATE {}
                            SET try_num=try_num+1,status={},res_data={},send_time={},receive_time={},setting_id={}
                            WHERE id={};
                        "#,
                        SenderMailMessageModel::table_name(),
                        SenderMailMessageStatus::IsReceived as i8,
                        res_item.send_id,
                        ntime,
                        ntime,
                        setting.id,
                        res_item.id,
                    )
                }
                SenderTaskStatus::Progress => {
                    self.wait_notify
                        .msg_notify(&item.mail.reply_host, res_item.id, Ok(false))
                        .await;

                    log_data.push((res_item.id, SenderLogStatus::Succ, res_item.send_id.clone()));
                    let ntime = now_time().unwrap_or_default();
                    sql_format!(
                        r#"UPDATE {}
                            SET try_num=try_num+1,status={},res_data={},send_time={},setting_id={}
                            WHERE id={};
                        "#,
                        SenderMailMessageModel::table_name(),
                        SenderMailMessageStatus::IsSend as i8,
                        res_item.send_id,
                        ntime,
                        setting.id,
                        res_item.id,
                    )
                }
                SenderTaskStatus::Failed(retry) => {
                    self.wait_notify
                        .msg_notify(
                            &item.mail.reply_host,
                            res_item.id,
                            Err(res_item.message.to_owned()),
                        )
                        .await;

                    log_data.push((
                        res_item.id,
                        SenderLogStatus::Fail,
                        res_item.message.to_owned(),
                    ));

                    if retry {
                        sql_format!(
                            r#"UPDATE {}
                                SET try_num=try_num+1,status=if(try_num>={},{},{})
                                WHERE id={} and status={};
                            "#,
                            SenderMailMessageModel::table_name(),
                            item.mail.max_try_num,
                            SenderMailMessageStatus::SendFail as i8,
                            SqlExpr(if cancel_data.contains(&res_item.id) {
                                sql_format!("{}", SenderMailMessageStatus::IsCancel as i8)
                            } else {
                                "status".to_string()
                            }),
                            res_item.id,
                            SenderMailMessageStatus::Init as i8,
                        )
                    } else {
                        sql_format!(
                            r#"UPDATE {}
                                SET try_num=try_num+1,status={}
                                WHERE id={} and status={};
                            "#,
                            SenderMailMessageModel::table_name(),
                            SenderMailMessageStatus::SendFail as i8,
                            res_item.id,
                            SenderMailMessageStatus::Init as i8,
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
        item: &MailTaskItem,
        record: &MailTaskData,
        error: &SenderExecError,
    ) {
        let fail_ids = record.data.iter().map(|e| e.id).collect::<Vec<_>>();

        for tmp in fail_ids.iter() {
            self.wait_notify
                .msg_notify(&item.mail.reply_host, *tmp, Err(error.to_string()))
                .await;
        }

        let sql = match error {
            SenderExecError::Finish(_) => {
                sql_format!(
                    r#"UPDATE {}
                            SET try_num=try_num+1,status={}
                            WHERE id in ({}) and status={};
                        "#,
                    SenderMailMessageModel::table_name(),
                    SenderMailMessageStatus::SendFail as i8,
                    fail_ids,
                    SenderMailMessageStatus::Init as i8,
                )
            }
            SenderExecError::Next(_) => {
                let cancel_data = self.cancel_data_ids(record).await;

                sql_format!(
                    r#"UPDATE {}
                    SET try_num=try_num+1,status=if(try_num>={},{},{})
                    WHERE id in ({}) and status={};
                "#,
                    SenderMailMessageModel::table_name(),
                    item.mail.max_try_num,
                    SenderMailMessageStatus::SendFail as i8,
                    SqlExpr(if cancel_data.is_empty() {
                        "status".to_string()
                    } else {
                        sql_format!(
                            "if ( id in ({}),{},status)",
                            cancel_data,
                            SenderMailMessageStatus::IsCancel as i8
                        )
                    }),
                    fail_ids,
                    SenderMailMessageStatus::Init as i8,
                )
            }
        };
        if let Err(err) = sqlx::query(sql.as_str()).execute(&self.db).await {
            warn!("change finish status fail{}", err);
            return;
        }
        let log_data = record
            .data
            .iter()
            .map(|e| (e.id, SenderLogStatus::Fail, error.to_string()))
            .collect::<Vec<_>>();
        self.message_logs
            .add_exec_log(&item.app_id(), &log_data, &setting.setting_key)
            .await;
        self.send_record_clear(item).await;
    }
}

#[async_trait]
impl TaskAcquisition<u64, MailTaskItem> for MailTaskAcquisition {
    //复用父结构体方法实现
    async fn read_send_task(
        &self,
        tasking_record: &HashMap<u64, TaskData>,
        limit: usize,
    ) -> Result<TaskRecord<u64, MailTaskItem>, String> {
        let (app_res, next) = self
            .message_reader
            .read_task(tasking_record, SenderMailMessageStatus::Init as i8, limit)
            .await
            .map_err(|e| e.to_string())?;
        let app_res = app_res
            .into_iter()
            .map(|e| MailTaskItem { mail: e })
            .collect();
        Ok(TaskRecord::new(app_res, next))
    }
}

#[derive(Clone)]
pub struct MailerTask {
    inner: Arc<Vec<SenderTaskExecutorBox<u64, MailTaskItem, MailTaskData>>>,
    acquisition: Arc<MailTaskAcquisition>,
    tpl_config: Arc<SenderTplConfig>,
    i: Arc<AtomicU32>,
}

impl MailerTask {
    pub fn new(
        acquisition: Arc<MailTaskAcquisition>,
        tpl_config: Arc<SenderTplConfig>,
        se: Vec<Box<dyn SenderTaskExecutor<u64, MailTaskItem, MailTaskData>>>,
    ) -> SenderResult<MailerTask> {
        if se.is_empty() {
            // "can't set task is empty".to_string()
            return Err(SenderError::System(fluent_message!("mail-task-empty")));
        }
        Ok(MailerTask {
            inner: Arc::new(
                se.into_iter()
                    .map(|e| (e, AtomicU32::new(0)))
                    .collect::<Vec<_>>(),
            ),
            i: AtomicU32::new(0).into(),
            tpl_config,
            acquisition,
        })
    }
}

#[async_trait]
impl TaskExecutor<u64, MailTaskItem> for MailerTask {
    async fn exec(&self, val: MailTaskItem) -> Result<(), String> {
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
