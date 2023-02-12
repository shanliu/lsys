use std::{collections::HashMap, sync::Arc};

use crate::model::{
    SenderSmsHistoryModel, SenderSmsHistoryModelRef, SenderSmsMessageModel,
    SenderSmsMessageModelRef, SenderSmsMessageStatus,
};
use async_trait::async_trait;
use lsys_core::{now_time, AppCore};

use sqlx::{MySql, Pool};
use sqlx_model::{sql_format, Insert, Select, Update};

use tracing::warn;

use super::task::{
    Task, TaskAcquisition, TaskError, TaskExecutioner, TaskItem, TaskRecord, TaskValue,
};
use sqlx_model::SqlQuote;

const SMSER_REDIS_PREFIX: &str = "sender-sms-";

pub struct Smser {
    app_core: Arc<AppCore>,
    redis: deadpool_redis::Pool,
    db: Pool<MySql>,
    task: Task<u64, SenderSmsMessageModel>,
    record: SmsTaskRecord,
}

impl Smser {
    //短信发送
    pub fn new(
        app_core: Arc<AppCore>,
        redis: deadpool_redis::Pool,
        db: Pool<MySql>,
        is_check: bool,
        check_timeout: usize,
        try_num: usize,
    ) -> Self {
        let task = Task::new(
            format!("{}-notify", SMSER_REDIS_PREFIX),
            format!("{}-read-lock", SMSER_REDIS_PREFIX),
            format!("{}-run-task", SMSER_REDIS_PREFIX),
            format!("{}-run-num", SMSER_REDIS_PREFIX),
            is_check,
            check_timeout,
        );
        let record = SmsTaskRecord::new(db.clone(), try_num);
        Self {
            app_core,
            db,
            redis,
            task,
            record,
        }
    }
    //发送模板消息
    pub async fn send(
        &self,
        area: &str,
        mobile: &str,
        tpl_id: &str,
        tpl_var: &str,
    ) -> Result<u64, String> {
        let id = self
            .record
            .add_send(mobile, tpl_id, tpl_var, area)
            .await
            .map_err(|e| e.to_string())?;
        let mut redis = self.redis.get().await.map_err(|e| e.to_string())?;
        if let Err(err) = self.task.notify(&mut redis).await {
            warn!("sms is add [{}] ,but send fail :{}", id, err)
        }
        Ok(id)
    }
    //后台发送任务，内部循环不退出
    pub async fn task<
        S: SmserTaskExecutioner<E>,
        E: TaskExecutioner<u64, SenderSmsMessageModel>,
    >(
        &self,
    ) {
        self.task
            .dispatch(
                self.app_core.clone(),
                &self.record,
                S::create(
                    self.app_core.clone(),
                    self.redis.clone(),
                    self.db.clone(),
                    self.record.clone(),
                ),
            )
            .await;
    }
}

pub trait SmserTaskExecutioner<E: TaskExecutioner<u64, SenderSmsMessageModel>>:
    TaskExecutioner<u64, SenderSmsMessageModel>
{
    fn create(
        app_core: Arc<AppCore>,
        redis: deadpool_redis::Pool,
        db: Pool<MySql>,
        record: SmsTaskRecord,
    ) -> E;
}

impl TaskItem<u64> for SenderSmsMessageModel {
    fn to_task_pk(&self) -> u64 {
        self.id
    }
    fn to_task_value(&self) -> TaskValue {
        TaskValue {
            host: hostname::get()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            time: now_time().unwrap_or_default(),
        }
    }
}

#[derive(Clone)]
pub struct SmsTaskRecord {
    db: Pool<MySql>,
    try_num: usize,
}

impl SmsTaskRecord {
    pub fn new(db: Pool<MySql>, try_num: usize) -> Self {
        Self { db, try_num }
    }
    pub async fn add_send(
        &self,
        mobile: &str,
        tpl_id: &str,
        tpl_var: &str,
        area: &str,
    ) -> Result<u64, sqlx::Error> {
        let add_time = now_time().unwrap_or_default();
        let tpl_id = tpl_id.to_owned();
        let tpl_var = tpl_var.to_owned();
        let area = area.to_owned();
        let mobile = mobile.to_owned();
        let idata = sqlx_model::model_option_set!(SenderSmsMessageModelRef,{
            mobile:mobile,
            area:area,
            tpl_id:tpl_id,
            tpl_var:tpl_var,
            try_num:0,
            status:SenderSmsMessageStatus::Init as i8,
            add_time:add_time,
            send_time:0,
        });
        let id = Insert::<sqlx::MySql, SenderSmsMessageModel, _>::new(idata)
            .execute(&self.db)
            .await?
            .last_insert_id();
        Ok(id)
    }
    pub async fn finish_send(
        &self,
        val: &SenderSmsMessageModel,
        res: &Result<(), String>,
    ) -> Result<(), sqlx::Error> {
        let (status, err_msg) = match res {
            Ok(()) => (SenderSmsMessageStatus::IsSend as i8, "".to_string()),
            Err(err) => (SenderSmsMessageStatus::SendFail as i8, err.to_owned()),
        };
        let set_try_num = val.try_num + 1;
        let send_time = now_time().unwrap_or_default();
        let idata = sqlx_model::model_option_set!(SenderSmsHistoryModelRef,{
            sms_message_id:val.id,
            status:status,
            send_message:err_msg,
            send_time:send_time,
        });
        let tmp = Insert::<sqlx::MySql, SenderSmsHistoryModel, _>::new(idata)
            .execute(&self.db)
            .await;
        if let Err(ie) = tmp {
            warn!("sms[{}] is send ,add history fail : {:?}", val.id, ie);
        }
        let mut change = sqlx_model::model_option_set!(SenderSmsMessageModelRef,{
            send_time:send_time,
            try_num:set_try_num
        });
        if SenderSmsMessageStatus::IsSend.eq(status)
            || (SenderSmsMessageStatus::SendFail.eq(status) && val.try_num as usize >= self.try_num)
        {
            change.status = Some(&status);
        }
        Update::<MySql, SenderSmsMessageModel, _>::new(change)
            .execute_by_pk(val, &self.db)
            .await?;
        Ok(())
    }
}
#[async_trait]
impl TaskAcquisition<u64, SenderSmsMessageModel> for SmsTaskRecord {
    async fn read_record(
        &self,
        tasking_record: &HashMap<u64, TaskValue>,
        limit: usize,
    ) -> Result<TaskRecord<u64, SenderSmsMessageModel>, TaskError> {
        let mut sql_vec = vec![];
        sql_vec.push(sql_format!("status = {}", SenderSmsMessageStatus::Init));
        let ids = tasking_record.keys().copied().collect::<Vec<u64>>();
        if !ids.is_empty() {
            sql_vec.push(sql_format!(" id not in ({})", ids));
        }
        let mut app_res = Select::type_new::<SenderSmsMessageModel>()
            .fetch_all_by_where::<SenderSmsMessageModel, _>(
                &sqlx_model::WhereOption::Where(format!(
                    "{} order by id asc limit {}",
                    sql_vec.join(" and "),
                    limit + 1
                )),
                &self.db,
            )
            .await
            .map_err(TaskError::Sqlx)?;
        if app_res.len() > limit {
            app_res.pop();
            return Ok(TaskRecord::new(app_res, true));
        }
        Ok(TaskRecord::new(app_res, false))
    }
}

mod sender_aliyun;
pub use sender_aliyun::*;
