use crate::{
    dao::{
        group_exec, MessageLogs, MessageReader, SenderError, SenderExecError, SenderResult,
        SenderTaskAcquisition, SenderTaskData, SenderTaskExecutor, SenderTaskExecutorBox,
        SenderTaskItem, SenderTaskResultItem, SenderTaskStatus, SenderTplConfig, SenderWaitNotify,
    },
    model::{
        SenderLogStatus, SenderMailBodyModel, SenderMailBodyStatus,
        SenderMailMessageModel, SenderMailMessageStatus, SenderMessageCancelModel,
    },
};
use async_trait::async_trait;
use lsys_core::db::{TableMeta, QueryBuilderExt, FieldValue};
use lsys_core::fluent_message;
use lsys_core::fluents::IntoFluentMessage;
use lsys_core::task_dispatch::{TaskAcquisition, TaskData, TaskExecutor, TaskItem, TaskRecord};
use lsys_core::utils::now_time;
use lsys_setting::model::SettingModel;
use std::{
    collections::HashMap,
    sync::{atomic::AtomicU32, Arc},
};

use lsys_core::db::Update;
use sqlx::{MySql, Pool, QueryBuilder, Row};
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
        self.mail.tpl_key.to_owned()
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
        let mut qb: QueryBuilder<'_, MySql> = QueryBuilder::new(format!(
            "select sender_message_id from {}",
            SenderMessageCancelModel::table_name(),
        ));
        qb.push_where().field_in_copied("sender_message_id", &msg_id);
        match qb.build().map(|row: sqlx::mysql::MySqlRow| -> u64 {
            row.get(0)
        }).fetch_all(&self.db).await
        {
            Ok(d) => d,
            Err(err) => {
                warn!("select cancel data fail:{}", err);
                vec![]
            }
        }
    }
    async fn send_record_clear(&self, item: &MailTaskItem) {
        let sql = format!(
            "select id from {} where sender_body_id=? and status=? limit 1",
            SenderMailMessageModel::table_name(),
        );
        if let Err(err) = sqlx::query_scalar::<_, u64>(&sql)
            .bind(item.mail.id)
            .bind(SenderMailMessageStatus::Init as i8)
            .fetch_one(&self.db).await {
            match err {
                sqlx::Error::RowNotFound => self.send_task_body_finish(item).await,
                _ => {
                    warn!("finish task ,check status fail{}", err)
                }
            }
        }
    }
    async fn send_task_body_finish(&self, item: &MailTaskItem) {
        let finish_time = now_time().unwrap_or_default();
        if let Err(err) = Update::<_,SenderMailBodyModel>::new()
            .set(SenderMailBodyModel::STATUS, SenderMailBodyStatus::Finish as i8)
            .set(SenderMailBodyModel::FINISH_TIME, finish_time)
            .execute(&self.db, |qb| {
                qb.push_where().field_eq("id", item.mail.id);
            })
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
            .map_err(|e| e.to_fluent_message().default_format())?;

        if app_res.is_empty() {
            if let Err(err) = Update::<_, SenderMailMessageModel>::new()
                .set(SenderMailMessageModel::STATUS, SenderMailMessageStatus::IsCancel as i8)
                .execute(&self.db, |qb| {
                    qb.push_where().field_eq("status", SenderMailMessageStatus::Init as i8);
                    qb.push_and().push(format!(
                        "id IN (SELECT sender_message_id FROM {}",
                        SenderMessageCancelModel::table_name(),
                    ));
                    qb.push_where().field_eq("sender_body_id", record.mail.id);
                    if !sending_data.is_empty() {
                        qb.push_and().field_not_in_copied("sender_message_id", sending_data);
                    }
                    qb.push(")");
                })
                .await
            {
                warn!(
                    "mail clear message cancel status fail[{}]{}",
                    record.mail.id, err
                );
            }
        }

        if sending_data.is_empty() && app_res.is_empty() {
            self.send_task_body_finish(record).await;
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
        match error {
            SenderExecError::Finish(_) => {
                if let Err(err) = Update::<_, SenderMailMessageModel>::new()
                    .set(SenderMailMessageModel::TRY_NUM, FieldValue::Expr("try_num+1".into()))
                    .set(SenderMailMessageModel::STATUS, SenderMailMessageStatus::SendFail as i8)
                    .execute(&self.db, |qb| {
                        qb.push_where().field_eq("sender_body_id", item.mail.id);
                        qb.push_and().field_eq("status", SenderMailMessageStatus::Init as i8);
                        if !in_task_id.is_empty() {
                            qb.push_and().field_not_in_copied("id", in_task_id);
                        }
                    })
                    .await
                {
                    warn!("change finish status fail{}", err);
                    return;
                }
            }
            SenderExecError::Next(_) => {
                let cancel_sql = format!(
                    "select sender_message_id from {} where sender_body_id =?",
                    SenderMessageCancelModel::table_name(),
                );
                let cancel_data = match sqlx::query_scalar::<_, u64>(&cancel_sql)
                    .bind(item.mail.id)
                    .fetch_all(&self.db)
                    .await
                {
                    Ok(d) => d,
                    Err(err) => {
                        warn!("select cancel data fail[all]:{}", err);
                        vec![]
                    }
                };

                let max_try_num = item.mail.max_try_num;
                let sender_body_id = item.mail.id;
                if let Err(err) = Update::<_, SenderMailMessageModel>::new()
                    .set(SenderMailMessageModel::TRY_NUM, FieldValue::Expr("try_num+1".into()))
                    .set(SenderMailMessageModel::STATUS, FieldValue::Dynamic(Box::new(move |qb| {
                        qb.push("if(");
                        qb.field_gte("try_num", max_try_num);
                        qb.push(",");
                        qb.push_bind(SenderMailMessageStatus::SendFail as i8);
                        qb.push(",");
                        if cancel_data.is_empty() {
                            qb.push("status)");
                        } else {
                            qb.push("if(");
                            qb.field_in_copied("id", &cancel_data);
                            qb.push(",");
                            qb.push_bind(SenderMailMessageStatus::IsCancel as i8);
                            qb.push(",status))");
                        }
                    })))
                    .execute(&self.db, |qb| {
                        qb.push_where().field_eq("sender_body_id", sender_body_id);
                        qb.push_and().field_eq("status", SenderMailMessageStatus::Init as i8);
                        if !in_task_id.is_empty() {
                            qb.push_and().field_not_in_copied("id", in_task_id);
                        }
                    })
                    .await
                {
                    warn!("change finish status fail{}", err);
                    return;
                }
            }
        };

        let mut msg_qb: QueryBuilder<'_, MySql> = QueryBuilder::new(format!(
            "select id from {}",
            SenderMailMessageModel::table_name(),
        ));
        msg_qb.push_where().field_eq("sender_body_id", item.mail.id);
        if !in_task_id.is_empty() {
            msg_qb.push_and().field_not_in_copied("id", in_task_id);
        }
        if let Ok(id_items) = msg_qb.build().map(|row: sqlx::mysql::MySqlRow| -> u64 {
            row.get(0)
        }).fetch_all(&self.db).await
        {
            let err_str = error.to_string();
            let log_data = id_items
                .into_iter()
                .map(|e| (e, SenderLogStatus::Fail, err_str.as_str()))
                .collect::<Vec<_>>();
            self.message_logs
                .add_exec_log(
                    item.app_id(),
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
            let exec_result = match res_item.status {
                SenderTaskStatus::Completed => {
                    self.wait_notify
                        .msg_notify(&item.mail.reply_host, res_item.snid, Ok(true))
                        .await;

                    log_data.push((
                        res_item.id,
                        SenderLogStatus::Succ,
                        res_item.send_id.as_str(),
                    ));
                    use lsys_core::db::Update;
                    let ntime = now_time().unwrap_or_default();
                    let send_id_str = res_item.send_id.clone();
                    let setting_id_val = setting.id;
                    let res_id = res_item.id;
                    Update::<_, SenderMailMessageModel>::new()
                        .set(SenderMailMessageModel::TRY_NUM, FieldValue::Expr("try_num+1".into()))
                        .set(SenderMailMessageModel::STATUS, SenderMailMessageStatus::IsReceived as i8)
                        .set(SenderMailMessageModel::RES_DATA, send_id_str)
                        .set(SenderMailMessageModel::SEND_TIME, ntime)
                        .set(SenderMailMessageModel::RECEIVE_TIME, ntime)
                        .set(SenderMailMessageModel::SETTING_ID, setting_id_val)
                        .execute(&self.db, |qb| {
                            qb.push_where().field_eq("id", res_id);
                        })
                        .await
                        .map(|_| sqlx::mysql::MySqlQueryResult::default())
                }
                SenderTaskStatus::Progress => {
                    self.wait_notify
                        .msg_notify(&item.mail.reply_host, res_item.snid, Ok(false))
                        .await;

                    log_data.push((
                        res_item.id,
                        SenderLogStatus::Succ,
                        res_item.send_id.as_str(),
                    ));
                    use lsys_core::db::Update;
                    let ntime = now_time().unwrap_or_default();
                    let send_id_str = res_item.send_id.clone();
                    let setting_id_val = setting.id;
                    let res_id = res_item.id;
                    Update::<_, SenderMailMessageModel>::new()
                        .set(SenderMailMessageModel::TRY_NUM, FieldValue::Expr("try_num+1".into()))
                        .set(SenderMailMessageModel::STATUS, SenderMailMessageStatus::IsSend as i8)
                        .set(SenderMailMessageModel::RES_DATA, send_id_str)
                        .set(SenderMailMessageModel::SEND_TIME, ntime)
                        .set(SenderMailMessageModel::SETTING_ID, setting_id_val)
                        .execute(&self.db, |qb| {
                            qb.push_where().field_eq("id", res_id);
                        })
                        .await
                        .map(|_| sqlx::mysql::MySqlQueryResult::default())
                }
                SenderTaskStatus::Failed(retry) => {
                    self.wait_notify
                        .msg_notify(
                            &item.mail.reply_host,
                            res_item.snid,
                            Err(res_item.message.to_owned()),
                        )
                        .await;

                    log_data.push((
                        res_item.id,
                        SenderLogStatus::Fail,
                        res_item.message.as_str(),
                    ));

                    if retry {
                        let max_try_num = item.mail.max_try_num;
                        let res_id = res_item.id;
                        let is_cancel = cancel_data.contains(&res_item.id);
                        Update::<_, SenderMailMessageModel>::new()
                            .set(SenderMailMessageModel::TRY_NUM, FieldValue::Expr("try_num+1".into()))
                            .set(SenderMailMessageModel::STATUS, FieldValue::Dynamic(Box::new(move |qb| {
                                qb.push("if(");
                                qb.field_gte("try_num", max_try_num);
                                qb.push(",");
                                qb.push_bind(SenderMailMessageStatus::SendFail as i8);
                                qb.push(",");
                                if is_cancel {
                                    qb.push_bind(SenderMailMessageStatus::IsCancel as i8);
                                } else {
                                    qb.push("status");
                                }
                                qb.push(")");
                            })))
                            .execute(&self.db, |qb| {
                                qb.push_where().field_eq("id", res_id);
                                qb.push_and().field_eq("status", SenderMailMessageStatus::Init as i8);
                            })
                            .await
                            .map(|_| sqlx::mysql::MySqlQueryResult::default())
                    } else {
                        Update::<_, SenderMailMessageModel>::new()
                            .set(SenderMailMessageModel::TRY_NUM, FieldValue::Expr("try_num+1".into()))
                            .set(SenderMailMessageModel::STATUS, SenderMailMessageStatus::SendFail as i8)
                            .execute(&self.db, |qb| {
                                qb.push_where().field_eq("id", res_item.id);
                                qb.push_and().field_eq("status", SenderMailMessageStatus::Init as i8);
                            })
                            .await
                            .map(|_| sqlx::mysql::MySqlQueryResult::default())
                    }
                }
            };
            if let Err(err) = exec_result {
                warn!("change message status fail[{}]{}", res_item.id, err);
                continue;
            }
        }
        self.message_logs
            .add_exec_log(item.app_id(), &log_data, &setting.setting_key)
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

        for tmp in record.data.iter() {
            self.wait_notify
                .msg_notify(&item.mail.reply_host, tmp.snid, Err(error.to_string()))
                .await;
        }

        match error {
            SenderExecError::Finish(_) => {
                if let Err(err) = Update::<_, SenderMailMessageModel>::new()
                    .set(SenderMailMessageModel::TRY_NUM, FieldValue::Expr("try_num+1".into()))
                    .set(SenderMailMessageModel::STATUS, SenderMailMessageStatus::SendFail as i8)
                    .execute(&self.db, |qb| {
                        qb.push_where().field_in_copied("id", &fail_ids);
                        qb.push_and().field_eq("status", SenderMailMessageStatus::Init as i8);
                    })
                    .await
                {
                    warn!("change finish status fail{}", err);
                    return;
                }
            }
            SenderExecError::Next(_) => {
                let cancel_data = self.cancel_data_ids(record).await;

                let max_try_num = item.mail.max_try_num;
                if let Err(err) = Update::<_, SenderMailMessageModel>::new()
                    .set(SenderMailMessageModel::TRY_NUM, FieldValue::Expr("try_num+1".into()))
                    .set(SenderMailMessageModel::STATUS, FieldValue::Dynamic(Box::new(move |qb| {
                        qb.push("if(");
                        qb.field_gte("try_num", max_try_num);
                        qb.push(",");
                        qb.push_bind(SenderMailMessageStatus::SendFail as i8);
                        qb.push(",");
                        if cancel_data.is_empty() {
                            qb.push("status)");
                        } else {
                            qb.push("if(");
                            qb.field_in_copied("id", &cancel_data);
                            qb.push(",");
                            qb.push_bind(SenderMailMessageStatus::IsCancel as i8);
                            qb.push(",status))");
                        }
                    })))
                    .execute(&self.db, |qb| {
                        qb.push_where().field_in_copied("id", &fail_ids);
                        qb.push_and().field_eq("status", SenderMailMessageStatus::Init as i8);
                    })
                    .await
                {
                    warn!("change finish status fail{}", err);
                    return;
                }
            }
        };
        let err_str = error.to_string();
        let log_data = record
            .data
            .iter()
            .map(|e| (e.id, SenderLogStatus::Fail, err_str.as_str()))
            .collect::<Vec<_>>();
        self.message_logs
            .add_exec_log(item.app_id(), &log_data, &setting.setting_key)
            .await;
        self.send_record_clear(item).await;
    }
}

#[async_trait]
impl TaskAcquisition<u64, MailTaskItem> for MailTaskAcquisition {
    //复用父结构体方法实现
    async fn read_exec_task(
        &self,
        tasking_record: &HashMap<u64, TaskData>,
        limit: usize,
    ) -> Result<TaskRecord<u64, MailTaskItem>, String> {
        let (app_res, next) = self
            .message_reader
            .read_task(tasking_record, SenderMailMessageStatus::Init as i8, limit)
            .await
            .map_err(|e| e.to_fluent_message().default_format())?;
        let app_res = app_res
            .into_iter()
            .map(|e| MailTaskItem { mail: e })
            .collect();
        Ok(TaskRecord::new(app_res, next))
    }
}

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
