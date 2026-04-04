use crate::{
    dao::{
        group_exec, MessageLogs, MessageReader, SenderError, SenderExecError, SenderResult,
        SenderTaskAcquisition, SenderTaskData, SenderTaskExecutor, SenderTaskExecutorBox,
        SenderTaskItem, SenderTaskResultItem, SenderTaskStatus, SenderTplConfig, SenderWaitNotify,
    },
    model::{
        SenderLogStatus, SenderMessageCancelModel, SenderSmsBodyModel,
        SenderSmsBodyStatus, SenderSmsMessageModel, SenderSmsMessageStatus,
    },
};
use async_trait::async_trait;
use lsys_core::db::{TableMeta, Update, QueryBuilderExt, FieldValue};
use lsys_core::fluent_message;
use lsys_core::fluents::IntoFluentMessage;
use lsys_core::task_dispatch::{TaskAcquisition, TaskData, TaskExecutor, TaskItem, TaskRecord};
use lsys_core::utils::now_time;
use lsys_setting::model::SettingModel;
use sqlx::{MySql, Pool, QueryBuilder, Row};
use std::{
    collections::HashMap,
    sync::{atomic::AtomicU32, Arc},
};
use tracing::warn;

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
        self.sms.tpl_key.to_owned()
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
    async fn send_record_clear(&self, item: &SmsTaskItem) {
        let sql = format!(
            "select id from {} where sender_body_id=? and status=? limit 1",
            SenderSmsMessageModel::table_name(),
        );
        if let Err(err) = sqlx::query_scalar::<_, u64>(&sql)
            .bind(item.sms.id)
            .bind(SenderSmsMessageStatus::Init as i8)
            .fetch_one(&self.db)
            .await {
            match err {
                sqlx::Error::RowNotFound => self.send_task_body_finish(item).await,
                _ => {
                    warn!("sms finish task ,check status fail{}", err)
                }
            }
        }
    }
    async fn send_task_body_finish(&self, item: &SmsTaskItem) {
        let finish_time = now_time().unwrap_or_default();
        if let Err(err) = Update::<_,SenderSmsBodyModel>::new()
            .set(SenderSmsBodyModel::STATUS, SenderSmsBodyStatus::Finish as i8)
            .set(SenderSmsBodyModel::FINISH_TIME, finish_time)
            .execute(&self.db, |qb| {
                qb.push_where().field_eq("id", item.sms.id);
            })
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
            if let Err(err) = Update::<_, SenderSmsMessageModel>::new()
                .set(SenderSmsMessageModel::STATUS, SenderSmsMessageStatus::IsCancel as i8)
                .execute(&self.db, |qb| {
                    qb.push_where().field_eq("status", SenderSmsMessageStatus::Init as i8);
                    qb.push_and().push(format!(
                        "id IN (SELECT sender_message_id FROM {}",
                        SenderMessageCancelModel::table_name(),
                    ));
                    qb.push_where().field_eq("sender_body_id", record.sms.id);
                    if !sending_data.is_empty() {
                        qb.push_and().field_not_in_copied("sender_message_id", sending_data);
                    }
                    qb.push(")");
                })
                .await
            {
                warn!(
                    "sms clear message cancel status fail[{}]{}",
                    record.sms.id, err
                );
            }
        }

        if sending_data.is_empty() && app_res.is_empty() {
            self.send_task_body_finish(record).await;
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
        match error {
            SenderExecError::Finish(_) => {
                if let Err(err) = Update::<_, SenderSmsMessageModel>::new()
                    .set(SenderSmsMessageModel::TRY_NUM, FieldValue::Expr("try_num+1".into()))
                    .set(SenderSmsMessageModel::STATUS, SenderSmsMessageStatus::SendFail as i8)
                    .execute(&self.db, |qb| {
                        qb.push_where().field_eq("sender_body_id", item.sms.id);
                        qb.push_and().field_eq("status", SenderSmsMessageStatus::Init as i8);
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
                    .bind(item.sms.id)
                    .fetch_all(&self.db)
                    .await
                {
                    Ok(d) => d,
                    Err(err) => {
                        warn!("select cancel data fail[all]:{}", err);
                        vec![]
                    }
                };

                let max_try_num = item.sms.max_try_num;
                let sender_body_id = item.sms.id;
                if let Err(err) = Update::<_, SenderSmsMessageModel>::new()
                    .set(SenderSmsMessageModel::TRY_NUM, FieldValue::Dynamic(Box::new(|qb| {
                        qb.push("try_num+1");
                    })))
                    .set(SenderSmsMessageModel::STATUS, FieldValue::Dynamic(Box::new(move |qb| {
                        qb.push("if(");
                        qb.field_gte("try_num", max_try_num);
                        qb.push(",");
                        qb.push_bind(SenderSmsMessageStatus::SendFail as i8);
                        qb.push(",");
                        if cancel_data.is_empty() {
                            qb.push("status)");
                        } else {
                            qb.push("if(");
                            qb.field_in_copied("id", &cancel_data);
                            qb.push(",");
                            qb.push_bind(SenderSmsMessageStatus::IsCancel as i8);
                            qb.push(",status))");
                        }
                    })))
                    .execute(&self.db, |qb| {
                        qb.push_where().field_eq("sender_body_id", sender_body_id);
                        qb.push_and().field_eq("status", SenderSmsMessageStatus::Init as i8);
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
            SenderSmsMessageModel::table_name(),
        ));
        msg_qb.push_where().field_eq("sender_body_id", item.sms.id);
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
        item: &SmsTaskItem,
        record: &SmsTaskData,
        res_items: &[SenderTaskResultItem],
    ) {
        let cancel_data = self.cancel_data_ids(record).await;
        let mut log_data = Vec::with_capacity(res_items.len());
        for res_item in res_items {
            let exec_result = match res_item.status {
                SenderTaskStatus::Completed => {
                    self.wait_notify
                        .msg_notify(&item.sms.reply_host, res_item.snid, Ok(true))
                        .await;

                    log_data.push((
                        res_item.id,
                        SenderLogStatus::Succ,
                        res_item.send_id.as_str(),
                    ));
                    let ntime = now_time().unwrap_or_default();
                    let sql = format!(
                        r#"UPDATE {}
                            SET try_num=try_num+1,status=?,res_data=?,send_time=?,receive_time=?,setting_id=?
                            WHERE id=?;
                        "#,
                        SenderSmsMessageModel::table_name(),
                    );
                    sqlx::query(&sql)
                        .bind(SenderSmsMessageStatus::IsReceived as i8)
                        .bind(&res_item.send_id)
                        .bind(ntime)
                        .bind(ntime)
                        .bind(setting.id)
                        .bind(res_item.id)
                        .execute(&self.db)
                        .await
                }
                SenderTaskStatus::Progress => {
                    self.wait_notify
                        .msg_notify(&item.sms.reply_host, res_item.snid, Ok(false))
                        .await;

                    log_data.push((
                        res_item.id,
                        SenderLogStatus::Succ,
                        res_item.send_id.as_str(),
                    ));
                    let ntime = now_time().unwrap_or_default();
                    let sql = format!(
                        r#"UPDATE {}
                            SET try_num=try_num+1,status=?,res_data=?,send_time=?,setting_id=?
                            WHERE id=?;
                        "#,
                        SenderSmsMessageModel::table_name(),
                    );
                    sqlx::query(&sql)
                        .bind(SenderSmsMessageStatus::IsSend as i8)
                        .bind(&res_item.send_id)
                        .bind(ntime)
                        .bind(setting.id)
                        .bind(res_item.id)
                        .execute(&self.db)
                        .await
                }
                SenderTaskStatus::Failed(retry) => {
                    log_data.push((
                        res_item.id,
                        SenderLogStatus::Fail,
                        res_item.message.as_str(),
                    ));
                    if retry {
                        let sql_retry = if cancel_data.contains(&res_item.id) {
                            format!(
                                r#"UPDATE {}
                                    SET try_num=try_num+1,status=if(try_num>=?,?,?)
                                    WHERE id=? and status=?;
                                "#,
                                SenderSmsMessageModel::table_name(),
                            )
                        } else {
                            format!(
                                r#"UPDATE {}
                                    SET try_num=try_num+1,status=if(try_num>=?,?,status)
                                    WHERE id=? and status=?;
                                "#,
                                SenderSmsMessageModel::table_name(),
                            )
                        };
                        let mut query = sqlx::query(&sql_retry)
                            .bind(item.sms.max_try_num)
                            .bind(SenderSmsMessageStatus::SendFail as i8);
                        if cancel_data.contains(&res_item.id) {
                            query = query.bind(SenderSmsMessageStatus::IsCancel as i8);
                        }
                        query
                            .bind(res_item.id)
                            .bind(SenderSmsMessageStatus::Init as i8)
                            .execute(&self.db).await
                    } else {
                        self.wait_notify
                            .msg_notify(
                                &item.sms.reply_host,
                                res_item.snid,
                                Err(res_item.message.to_owned()),
                            )
                            .await;

                        let sql_no_retry = format!(
                            r#"UPDATE {}
                                SET try_num=try_num+1,status=?
                                WHERE id=? and status=?;
                            "#,
                            SenderSmsMessageModel::table_name(),
                        );
                        sqlx::query(&sql_no_retry)
                            .bind(SenderSmsMessageStatus::SendFail as i8)
                            .bind(res_item.id)
                            .bind(SenderSmsMessageStatus::Init as i8)
                            .execute(&self.db).await
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
        item: &SmsTaskItem,
        record: &SmsTaskData,
        error: &SenderExecError,
    ) {
        let fail_ids = record.data.iter().map(|e| e.id).collect::<Vec<_>>();
        for tmp in record.data.iter() {
            self.wait_notify
                .msg_notify(&item.sms.reply_host, tmp.snid, Err(error.to_string()))
                .await;
        }
        match error {
            SenderExecError::Finish(_) => {
                if let Err(err) = Update::<_, SenderSmsMessageModel>::new()
                    .set(SenderSmsMessageModel::TRY_NUM, FieldValue::Expr("try_num+1".into()))
                    .set(SenderSmsMessageModel::STATUS, SenderSmsMessageStatus::SendFail as i8)
                    .execute(&self.db, |qb| {
                        qb.push_where().field_in_copied("id", &fail_ids);
                        qb.push_and().field_eq("status", SenderSmsMessageStatus::Init as i8);
                    })
                    .await
                {
                    warn!("change finish status fail{}", err);
                    return;
                }
            }
            SenderExecError::Next(_) => {
                let cancel_data = self.cancel_data_ids(record).await;

                let max_try_num = item.sms.max_try_num;
                if let Err(err) = Update::<_, SenderSmsMessageModel>::new()
                    .set(SenderSmsMessageModel::TRY_NUM, FieldValue::Dynamic(Box::new(|qb| {
                        qb.push("try_num+1");
                    })))
                    .set(SenderSmsMessageModel::STATUS, FieldValue::Dynamic(Box::new(move |qb| {
                        qb.push("if(");
                        qb.field_gte("try_num", max_try_num);
                        qb.push(",");
                        qb.push_bind(SenderSmsMessageStatus::SendFail as i8);
                        qb.push(",");
                        if cancel_data.is_empty() {
                            qb.push("status)");
                        } else {
                            qb.push("if(");
                            qb.field_in_copied("id", &cancel_data);
                            qb.push(",");
                            qb.push_bind(SenderSmsMessageStatus::IsCancel as i8);
                            qb.push(",status))");
                        }
                    })))
                    .execute(&self.db, |qb| {
                        qb.push_where().field_in_copied("id", &fail_ids);
                        qb.push_and().field_eq("status", SenderSmsMessageStatus::Init as i8);
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
