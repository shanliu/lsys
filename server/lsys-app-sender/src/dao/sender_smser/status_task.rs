use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;

use lsys_app::dao::AppNotify;
use lsys_core::fluent_message;
use lsys_core::now_time;
use lsys_core::IntoFluentMessage;
use lsys_setting::dao::MultipleSetting;

use lsys_lib_sms::SendNotifyStatus;
use lsys_setting::model::SettingModel;

use lsys_core::db::ModelTableName;
use lsys_core::db::SqlQuote;
use lsys_core::sql_format;
use lsys_lib_sms::SendDetailItem;
use redis::AsyncCommands;
use sqlx::Pool;
use tracing::warn;

use crate::dao::MessageLogs;
use crate::dao::SenderResult;
use crate::model::SenderLogStatus;

use crate::{
    dao::{SenderError, SenderExecError},
    model::{SenderSmsMessageModel, SenderSmsMessageStatus},
};
use lsys_core::{TaskAcquisition, TaskData, TaskExecutor, TaskItem, TaskRecord};

use super::add_notify_callback;
use super::SmsRecord;

pub struct SmsStatusQuery {
    notify_data_key: String,
    timeout: u64,
    redis: deadpool_redis::Pool,
}

impl SmsStatusQuery {
    pub fn new(redis: deadpool_redis::Pool, notify_data_key: &str, timeout: u64) -> Self {
        Self {
            redis,
            timeout,
            notify_data_key: notify_data_key.to_string(),
        }
    }
}

impl SmsStatusQuery {
    pub async fn add_query(&self, items: &[&SenderSmsMessageModel]) -> SenderResult<()> {
        let ntime = now_time().unwrap_or_default();
        let query_ids = items
            .iter()
            .filter(|e| {
                e.send_time + self.timeout < ntime && SenderSmsMessageStatus::IsSend.eq(e.status)
            })
            .map(|e| e.id)
            .collect::<Vec<u64>>();
        if query_ids.is_empty() {
            return Ok(());
        }
        let mut conn = self.redis.get().await?;
        let _: () = conn.sadd(&self.notify_data_key, query_ids).await?;

        Ok(())
    }
}

pub struct SmsStatusTaskItem(u64);
impl TaskItem<u64> for SmsStatusTaskItem {
    fn to_task_pk(&self) -> u64 {
        self.0
    }
}

#[async_trait]
pub trait SmsStatusTaskExecutor: Sync + Send + 'static {
    //适配器标识key
    fn setting_key(&self) -> String;
    //实现查询指定短信发送状态
    //可能查询时返回多个
    async fn exec(
        &self,
        msg: &SenderSmsMessageModel,
        setting: &SettingModel,
    ) -> Result<Vec<SendDetailItem>, SenderExecError>;
}

pub struct SmsStatusTaskAcquisition {
    notify_data_key: String,
    redis: deadpool_redis::Pool,
}
impl SmsStatusTaskAcquisition {
    pub fn new(redis: deadpool_redis::Pool, notify_data_key: String) -> Self {
        SmsStatusTaskAcquisition {
            redis,
            notify_data_key,
        }
    }
}

#[async_trait]
impl TaskAcquisition<u64, SmsStatusTaskItem> for SmsStatusTaskAcquisition {
    //复用父结构体方法实现
    async fn read_exec_task(
        &self,
        tasking_record: &HashMap<u64, TaskData>,
        limit: usize,
    ) -> Result<TaskRecord<u64, SmsStatusTaskItem>, String> {
        // 根据执行中最大的id 查一批id加入任务
        //从redis中获取数据，排除掉 tasking_record keys
        //如果获取量小于limit next 为false
        let (data, next) = {
            let mut conn = self.redis.get().await.map_err(|e| e.to_string())?;
            if tasking_record.is_empty() {
                let mut tmp: Vec<u64> = conn
                    .srandmember_multiple(&self.notify_data_key, limit + 1)
                    .await
                    .map_err(|e| e.to_string())?;
                if tmp.len() > limit {
                    tmp.pop();
                    (tmp, true)
                } else {
                    (tmp, false)
                }
            } else {
                let items = tasking_record
                    .iter()
                    .map(|e| e.0.to_owned())
                    .collect::<Vec<u64>>();
                let mut iter: redis::AsyncIter<'_, u64> = conn
                    .sscan(&self.notify_data_key)
                    .await
                    .map_err(|e| e.to_string())?;
                let mut out = Vec::with_capacity(limit);
                let mut next = false;
                while let Some(tmp) = iter.next_item().await {
                    if items.contains(&tmp) {
                        continue;
                    }
                    if out.len() == limit {
                        next = true;
                        break;
                    }
                    out.push(tmp);
                }
                (out, next)
            }
        };
        Ok(TaskRecord::new(
            data.into_iter().map(SmsStatusTaskItem).collect(),
            next,
        ))
    }
}

#[derive(Clone)]
pub struct SmsStatusTask {
    inner: Arc<Vec<Box<dyn SmsStatusTaskExecutor>>>,
    recrod: Arc<SmsRecord>,
    db: Pool<sqlx::MySql>,
    setting: Arc<MultipleSetting>,
    message_logs: Arc<MessageLogs>,
    notify: Arc<AppNotify>,
}

impl SmsStatusTask {
    pub fn new(
        inner: Vec<Box<dyn SmsStatusTaskExecutor>>,
        recrod: Arc<SmsRecord>,
        db: Pool<sqlx::MySql>,
        notify: Arc<AppNotify>,
        setting: Arc<MultipleSetting>,
        message_logs: Arc<MessageLogs>,
    ) -> Result<Self, SenderError> {
        if inner.is_empty() {
            return Err(SenderError::System(fluent_message!(
                "sms-status-task-empty"
            )));
            // return Err(SenderError::System("can't set task is empty".to_string()));
        }
        Ok(Self {
            inner: Arc::new(inner.into_iter().collect::<Vec<_>>()),
            recrod,
            db,
            setting,
            message_logs,
            notify,
        })
    }
}

#[async_trait]
impl TaskExecutor<u64, SmsStatusTaskItem> for SmsStatusTask {
    async fn exec(&self, val: SmsStatusTaskItem) -> Result<(), String> {
        let sms = self
            .recrod
            .find_message_by_id(&val.0)
            .await
            .map_err(|e| e.to_fluent_message().default_format())?;

        let body = self
            .recrod
            .find_body_by_id(&sms.sender_body_id)
            .await
            .map_err(|e| e.to_fluent_message().default_format())?;

        match self.setting.find(None, sms.setting_id).await {
            Ok(setting) => {
                for tmp in self.inner.iter() {
                    if tmp.setting_key() == setting.setting_key {
                        match tmp.exec(&sms, &setting).await {
                            Ok(notify_data) => {
                                for ntmp in notify_data {
                                    match ntmp.status {
                                        SendNotifyStatus::Progress => {}
                                        SendNotifyStatus::Completed => {
                                            if let Err(err) = sqlx::query(
                                                sql_format!(
                                                    r#"UPDATE {}
                                                    SET status={}
                                                    WHERE setting_id={} and res_data={};
                                                "#,
                                                    SenderSmsMessageModel::table_name(),
                                                    SenderSmsMessageStatus::IsReceived as i8,
                                                    setting.id,
                                                    ntmp.send_id
                                                )
                                                .as_str(),
                                            )
                                            .execute(&self.db)
                                            .await
                                            {
                                                warn!("sms change to succ fail[{}]{}", val.0, err);
                                            }
                                            self.message_logs
                                                .add_exec_log(
                                                    &body.app_id,
                                                    &[(
                                                        sms.id,
                                                        SenderLogStatus::NotifySucc,
                                                        &ntmp.message,
                                                    )],
                                                    &setting.setting_key,
                                                )
                                                .await;
                                        }
                                        SendNotifyStatus::Failed => {
                                            if let Err(err) = sqlx::query(
                                                sql_format!(
                                                    r#"UPDATE {}
                                                    SET status={}
                                                    WHERE setting_id={} and res_data={};
                                                "#,
                                                    SenderSmsMessageModel::table_name(),
                                                    SenderSmsMessageStatus::SendFail as i8,
                                                    setting.id,
                                                    ntmp.send_id
                                                )
                                                .as_str(),
                                            )
                                            .execute(&self.db)
                                            .await
                                            {
                                                warn!(
                                                    "sms change to fail is fail[{}]{}",
                                                    val.0, err
                                                );
                                            }

                                            self.message_logs
                                                .add_exec_log(
                                                    &body.app_id,
                                                    &[(
                                                        sms.id,
                                                        SenderLogStatus::NotifyFail,
                                                        &ntmp.message,
                                                    )],
                                                    &setting.setting_key,
                                                )
                                                .await;
                                        }
                                    }
                                    add_notify_callback(
                                        &self.db,
                                        &self.notify,
                                        body.app_id,
                                        sms.id,
                                    )
                                    .await;
                                }
                                return Ok(());
                            }
                            Err(err) => match err {
                                SenderExecError::Finish(_) => {
                                    break;
                                }
                                SenderExecError::Next(err) => {
                                    return Err(err);
                                }
                            },
                        }
                    }
                }
            }
            Err(err) => {
                return Err(err.to_fluent_message().default_format());
            }
        };
        return Err(format!("not find any status apatar :{}", val.0));
    }
}
