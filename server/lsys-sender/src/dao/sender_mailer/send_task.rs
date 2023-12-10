use crate::{
    dao::{
        group_exec, MessageLogs, MessageReader, SenderError, SenderExecError, SenderResult,
        SenderTaskAcquisition, SenderTaskData, SenderTaskExecutor, SenderTaskExecutorBox,
        SenderTaskItem, SenderTaskResult, SenderTaskStatus, SenderTplConfig,
    },
    model::{
        SenderLogStatus, SenderMailBodyModel, SenderMailBodyModelRef, SenderMailBodyStatus,
        SenderMailMessageModel, SenderMailMessageStatus, SenderMessageCancelModel,
    },
};
use async_trait::async_trait;
use lsys_core::now_time;
use lsys_core::{TaskAcquisition, TaskData, TaskExecutor, TaskItem, TaskRecord};
use lsys_setting::model::SettingModel;
use sqlx_model::{ModelTableName, SqlExpr, SqlQuote};
use std::{
    collections::HashMap,
    sync::{atomic::AtomicU32, Arc},
};

use sqlx::{MySql, Pool};
use sqlx_model::{sql_format, Update};
use tracing::{info, warn};

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
    message_logs: Arc<MessageLogs>,
    message_reader: Arc<MessageReader<SenderMailBodyModel, SenderMailMessageModel>>,
}

impl MailTaskAcquisition {
    pub fn new(
        db: Pool<sqlx::MySql>,
        message_logs: Arc<MessageLogs>,
        message_reader: Arc<MessageReader<SenderMailBodyModel, SenderMailMessageModel>>,
    ) -> Self {
        Self {
            db,
            message_logs,
            message_reader,
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
                    WHERE status not in ({}) and  id in (select sender_message_id from {} where {});
                "#,
                    SenderMailMessageModel::table_name(),
                    SenderMailMessageStatus::IsCancel as i8,
                    &[
                        SenderMailMessageStatus::IsSend as i8,
                        SenderMailMessageStatus::IsReceived as i8,
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
                    "mail clear message cancel status fail[{}]{}",
                    record.mail.id, err
                );
            }
        }

        if sending_data.is_empty() && app_res.is_empty() {
            let status = SenderMailBodyStatus::Finish as i8;
            let finish_time = now_time().unwrap_or_default();
            let change = sqlx_model::model_option_set!(SenderMailBodyModelRef,{
                status:status,
                finish_time:finish_time
            });
            if let Err(err) = Update::<MySql, SenderMailBodyModel, _>::new(change)
                .execute_by_scalar_pk(record.mail.id, &self.db)
                .await
            {
                warn!("not find any message,change finish status fail{}", err);
            } else {
                info!("not find any message,change mail body is finish");
            }
        }

        Ok(MailTaskData::new(app_res))
    }
    //完成指定短信任务回调
    async fn finish_send_task(
        &self,
        setting: &SettingModel,
        item: &MailTaskItem,
        record: &MailTaskData,
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
                        SenderTaskStatus::Completed | SenderTaskStatus::Progress => {
                            log_data.push((
                                res_item.id,
                                SenderLogStatus::Succ,
                                res_item.send_id.clone(),
                            ));
                            sql_format!(
                                r#"UPDATE {}
                                    SET try_num=try_num+1,status={},res_data={},send_time={},setting_id={}
                                    WHERE id={};
                                "#,
                                SenderMailMessageModel::table_name(),
                                SenderMailMessageStatus::IsSend as i8,
                                res_item.id,
                                now_time().unwrap_or_default(),
                                setting.id,
                                res_item.send_id
                            )
                        }
                        SenderTaskStatus::Failed(retry) => {
                            log_data.push((res_item.id, SenderLogStatus::Fail, res_item.message));

                            if retry {
                                sql_format!(
                                    r#"UPDATE {}
                                            SET try_num=try_num+1,status={}
                                            WHERE id={};
                                        "#,
                                    SenderMailMessageModel::table_name(),
                                    SqlExpr(if cancel_data.contains(&res_item.id) {
                                        sql_format!("{}", SenderMailMessageStatus::IsCancel as i8)
                                    } else {
                                        sql_format!(
                                            "if(try_num>={},{},status)",
                                            item.mail.max_try_num,
                                            SenderMailMessageStatus::SendFail as i8,
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
                                    SenderMailMessageModel::table_name(),
                                    SenderMailMessageStatus::SendFail as i8,
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
                                SenderMailMessageModel::table_name(),
                                SenderMailMessageStatus::SendFail as i8,
                                fail_ids,
                            )
                        }
                        SenderExecError::Next(_) => {
                            sql_format!(
                                r#"UPDATE {}
                            SET try_num=try_num+1,status=if(try_num>={},{},status)
                            WHERE id in ({});
                        "#,
                                SenderMailMessageModel::table_name(),
                                item.mail.max_try_num,
                                SenderMailMessageStatus::SendFail as i8,
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
                                SenderMailMessageModel::table_name(),
                                SenderMailMessageStatus::IsCancel as i8,
                                cancel_data,
                            )
                        }
                        SenderExecError::Next(_) => {
                            sql_format!(
                                r#"UPDATE {}
                        SET try_num=try_num+1,status=if(try_num>={},{},{})
                        WHERE id in ({});
                    "#,
                                SenderMailMessageModel::table_name(),
                                item.mail.max_try_num,
                                SenderMailMessageStatus::SendFail as i8,
                                SenderMailMessageStatus::IsCancel as i8,
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
            SenderMailMessageModel::table_name(),
            item.mail.id,
            SenderMailMessageStatus::Init as i8
        );
        if let Err(err) = sqlx::query_scalar::<_, u64>(&sql).fetch_one(&self.db).await {
            match err {
                sqlx::Error::RowNotFound => {
                    let status = SenderMailBodyStatus::Finish as i8;
                    let finish_time = now_time().unwrap_or_default();
                    let change = sqlx_model::model_option_set!(SenderMailBodyModelRef,{
                        status:status,
                        finish_time:finish_time
                    });
                    if let Err(err) = Update::<MySql, SenderMailBodyModel, _>::new(change)
                        .execute_by_scalar_pk(item.mail.id, &self.db)
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
            return Err(SenderError::System("can't set task is empty".to_string()));
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
