use std::{
    collections::HashMap,
    sync::{atomic::AtomicU32, Arc},
};

use async_trait::async_trait;
use lsys_core::now_time;

use crate::{
    dao::{
        group_exec, MessageLogs, MessageReader, SenderError, SenderExecError, SenderResult,
        SenderTaskAcquisition, SenderTaskData, SenderTaskExecutor, SenderTaskExecutorBox,
        SenderTaskItem, SenderTaskResult, SenderTaskStatus, SenderTplConfig,
    },
    model::{
        SenderLogStatus, SenderMessageCancelModel, SenderSmsBodyModel, SenderSmsBodyModelRef,
        SenderSmsBodyStatus, SenderSmsMessageModel, SenderSmsMessageStatus, SenderTplConfigModel,
    },
};
use lsys_core::{TaskAcquisition, TaskData, TaskExecutor, TaskItem, TaskRecord};
use lsys_setting::model::SettingModel;
use sqlx::{MySql, Pool};
use sqlx_model::{sql_format, ModelTableName, SqlExpr, Update};
use tracing::{info, warn};

use super::SmsRecord;
use sqlx_model::SqlQuote;

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
    message_logs: Arc<MessageLogs>,
    message_reader: Arc<MessageReader<SenderSmsBodyModel, SenderSmsMessageModel>>,
}
impl SmsTaskAcquisition {
    pub fn new(
        db: Pool<sqlx::MySql>,
        message_logs: Arc<MessageLogs>,
        message_reader: Arc<MessageReader<SenderSmsBodyModel, SenderSmsMessageModel>>,
    ) -> Self {
        Self {
            db,
            message_logs,
            message_reader,
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
            .map_err(|e| e.to_string())?;

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
                    WHERE status not in ({}) and id in (select sender_message_id from {} where {});
                "#,
                    SenderSmsBodyModel::table_name(),
                    SenderSmsMessageStatus::IsCancel as i8,
                    &[
                        SenderSmsMessageStatus::IsSend as i8,
                        SenderSmsMessageStatus::IsReceived as i8,
                    ],
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
            let status = SenderSmsBodyStatus::Finish as i8;
            let finish_time = now_time().unwrap_or_default();
            let change = sqlx_model::model_option_set!(SenderSmsBodyModelRef,{
                status:status,
                finish_time:finish_time
            });
            if let Err(err) = Update::<MySql, SenderSmsBodyModel, _>::new(change)
                .execute_by_scalar_pk(record.sms.id, &self.db)
                .await
            {
                warn!("not find any message,change finish status fail{}", err);
            } else {
                info!("not find any message,change sms body is finish");
            }
        }

        Ok(SmsTaskData::new(app_res))
    }
    async fn finish_send_task(
        &self,
        setting: &SettingModel,
        item: &SmsTaskItem,
        record: &SmsTaskData,
        exec_res: SenderTaskResult,
    ) {
        let msg_id = record.data.iter().map(|e| e.id).collect::<Vec<_>>();
        let cancel_sql = sql_format!(
            "select sender_message_id from {} where sender_message_id in ({})",
            SenderMessageCancelModel::table_name(),
            msg_id
        );
        let cancel_data = match sqlx::query_scalar::<_, u64>(&cancel_sql)
            .fetch_all(&self.db)
            .await
        {
            Ok(d) => d,
            Err(err) => {
                warn!("select cancel data fail:{}", err);
                vec![]
            }
        };

        match exec_res {
            Ok(res_items) => {
                let mut log_data = Vec::with_capacity(res_items.len());
                for res_item in res_items {
                    let sql = match res_item.status {
                        SenderTaskStatus::Completed => {
                            log_data.push((
                                res_item.id,
                                SenderLogStatus::Succ,
                                res_item.send_id.clone(),
                            ));
                            let ntime = now_time().unwrap_or_default();
                            sql_format!(
                                r#"UPDATE {}
                                    SET try_num=try_num+1,status={},res_data={},send_time={},receive_time={},setting_id={}
                                    WHERE id={};
                                "#,
                                SenderSmsMessageModel::table_name(),
                                SenderSmsMessageStatus::IsReceived as i8,
                                res_item.id,
                                ntime,
                                ntime,
                                setting.id,
                                res_item.send_id
                            )
                        }
                        SenderTaskStatus::Progress => {
                            log_data.push((
                                res_item.id,
                                SenderLogStatus::Succ,
                                res_item.send_id.clone(),
                            ));
                            let ntime = now_time().unwrap_or_default();
                            sql_format!(
                                r#"UPDATE {}
                                    SET try_num=try_num+1,status={},res_data={},send_time={},setting_id={}
                                    WHERE id={};
                                "#,
                                SenderSmsMessageModel::table_name(),
                                SenderSmsMessageStatus::IsSend as i8,
                                res_item.id,
                                res_item.send_id,
                                setting.id,
                                ntime
                            )
                        }
                        SenderTaskStatus::Failed(retry) => {
                            log_data.push((
                                res_item.id,
                                SenderLogStatus::Fail,
                                res_item.message.to_string(),
                            ));
                            if retry {
                                sql_format!(
                                    r#"UPDATE {}
                                            SET try_num=try_num+1,status={}
                                            WHERE id={};
                                        "#,
                                    SenderSmsMessageModel::table_name(),
                                    SqlExpr(if cancel_data.contains(&res_item.id) {
                                        sql_format!("{}", SenderSmsMessageStatus::IsCancel as i8)
                                    } else {
                                        sql_format!(
                                            "if(try_num>={},{},status)",
                                            item.sms.max_try_num,
                                            SenderSmsMessageStatus::SendFail as i8,
                                        )
                                    }),
                                    res_item.id,
                                )
                            } else {
                                sql_format!(
                                    r#"UPDATE {}
                                            SET try_num=try_num+1,status={}
                                            WHERE id={};
                                        "#,
                                    SenderSmsMessageModel::table_name(),
                                    SenderSmsMessageStatus::SendFail as i8,
                                    res_item.id,
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
            }
            Err(err) => {
                let fail_ids = msg_id
                    .into_iter()
                    .filter(|t| !cancel_data.contains(t))
                    .collect::<Vec<_>>();
                if !fail_ids.is_empty() {
                    let sql = match err {
                        SenderExecError::Finish(_) => {
                            sql_format!(
                                r#"UPDATE {}
                                SET try_num=try_num+1,status={}
                                WHERE id in ({});
                            "#,
                                SenderSmsMessageModel::table_name(),
                                SenderSmsMessageStatus::SendFail as i8,
                                fail_ids,
                            )
                        }
                        SenderExecError::Next(_) => {
                            sql_format!(
                                r#"UPDATE {}
                                SET try_num=try_num+1,status=if(try_num>={},{},status)
                                WHERE id in ({});
                            "#,
                                SenderSmsMessageModel::table_name(),
                                item.sms.max_try_num,
                                SenderSmsMessageStatus::SendFail as i8,
                                fail_ids
                            )
                        }
                    };
                    if let Err(err) = sqlx::query(sql.as_str()).execute(&self.db).await {
                        warn!("change finish status fail{}", err);
                        return;
                    }
                }
                if !cancel_data.is_empty() {
                    let sql = match err {
                        SenderExecError::Finish(_) => {
                            sql_format!(
                                r#"UPDATE {}
                            SET try_num=try_num+1,status={}
                            WHERE id in ({});
                        "#,
                                SenderSmsMessageModel::table_name(),
                                SenderSmsMessageStatus::IsCancel as i8,
                                cancel_data,
                            )
                        }
                        SenderExecError::Next(_) => {
                            sql_format!(
                                r#"UPDATE {}
                            SET try_num=try_num+1,status=if(try_num>={},{},{})
                            WHERE id in ({});
                        "#,
                                SenderSmsMessageModel::table_name(),
                                item.sms.max_try_num,
                                SenderSmsMessageStatus::SendFail as i8,
                                SenderSmsMessageStatus::IsCancel as i8,
                                cancel_data,
                            )
                        }
                    };
                    if let Err(err) = sqlx::query(sql.as_str()).execute(&self.db).await {
                        warn!("change cancel status fail{}", err);
                        return;
                    }
                }
                let log_data = record
                    .data
                    .iter()
                    .map(|e| (e.id, SenderLogStatus::Fail, err.to_string()))
                    .collect::<Vec<_>>();
                self.message_logs
                    .add_exec_log(&item.app_id(), &log_data, &setting.setting_key)
                    .await;
            }
        }
        let sql = sql_format!(
            "select id from {} where sender_body_id={} and status={} limit 1",
            SenderSmsMessageModel::table_name(),
            item.sms.id,
            SenderSmsMessageStatus::Init as i8
        );
        if let Err(err) = sqlx::query_scalar::<_, u64>(&sql).fetch_one(&self.db).await {
            match err {
                sqlx::Error::RowNotFound => {
                    let status = SenderSmsBodyStatus::Finish as i8;
                    let finish_time = now_time().unwrap_or_default();
                    let change = sqlx_model::model_option_set!(SenderSmsBodyModelRef,{
                        status:status,
                        finish_time:finish_time
                    });
                    if let Err(err) = Update::<MySql, SenderSmsBodyModel, _>::new(change)
                        .execute_by_scalar_pk(item.sms.id, &self.db)
                        .await
                    {
                        warn!("change finish status fail{}", err)
                    }
                }
                _ => {
                    warn!("finish task ,check status fail{}", err)
                }
            }
        }
    }
}

#[async_trait]
impl TaskAcquisition<u64, SmsTaskItem> for SmsTaskAcquisition {
    //复用父结构体方法实现
    async fn read_send_task(
        &self,
        tasking_record: &HashMap<u64, TaskData>,
        limit: usize,
    ) -> Result<TaskRecord<u64, SmsTaskItem>, String> {
        let (app_res, next) = self
            .message_reader
            .read_task(tasking_record, SenderSmsBodyStatus::Init as i8, limit)
            .await
            .map_err(|e| e.to_string())?;
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
            return Err(SenderError::System("can't set task is empty".to_string()));
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
