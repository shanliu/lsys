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

use super::task::{Task, TaskAcquisition,TaskRecord, TaskError, TaskExecutioner, TaskItem, TaskValue};
use sqlx_model::SqlQuote;

const SMSER_REDIS_PREFIX: &str = "sender-sms-";

//短信任务记录
#[derive(Clone)]
pub struct SmsTaskRecord {
    try_num: usize,
    db: Pool<sqlx::MySql>,
}
impl SmsTaskRecord {
    pub fn new(db: Pool<sqlx::MySql>, try_num: usize) -> Self {
        Self { db, try_num }
    }
    //读取短信任务数据
    pub async fn read(
        &self,
        tasking_record: &HashMap<u64, TaskValue>,
        limit: usize,
    ) -> Result<(Vec<SenderSmsMessageModel>, bool), sqlx::Error> {
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
            .await?;
        let next = if app_res.len() > limit {
            app_res.pop();
            true
        } else {
            false
        };
        Ok((app_res, next))
    }
    //添加短信任务
    pub async fn add(
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
    //完成指定短信任务
    pub async fn finish(
        &self,
        send_type: String,
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
            send_type:send_type,
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

pub struct SmsTaskItem<T: Send + Sync + 'static> {
    pub sms: SenderSmsMessageModel,
    pub attr: T,
}

impl<T: Send + Sync + 'static> TaskItem<u64> for SmsTaskItem<T> {
    fn to_task_pk(&self) -> u64 {
        self.sms.id
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

#[async_trait]
pub trait SmsTaskAcquisition<T: Send + Sync + 'static>:
    TaskAcquisition<u64, SmsTaskItem<T>> + Clone
{
    async fn read_record_attr(
        &self,
        res: Vec<SenderSmsMessageModel>,
    ) -> Result<Vec<SmsTaskItem<T>>, TaskError>;
    fn sms_record(&self) -> &SmsTaskRecord;
    async fn read_record(
        &self,
        tasking_record: &HashMap<u64, TaskValue>,
        limit: usize,
    ) -> Result<TaskRecord<u64, SmsTaskItem<T>>, TaskError> {
        let (app_res, next) = self
            .sms_record()
            .read(tasking_record, limit)
            .await
            .map_err(TaskError::Sqlx)?;
        let app_res = self.read_record_attr(app_res).await?;
        Ok(TaskRecord::new(app_res, next))
    }
}


pub struct Smser<A: SmsTaskAcquisition<T>, T: Send + Sync + 'static> {
    app_core: Arc<AppCore>,
    redis: deadpool_redis::Pool,
    db: Pool<MySql>,
    task: Task<u64, SmsTaskItem<T>>,
    acquisition: A,
}

impl<A: SmsTaskAcquisition<T>, T: Send + Sync + 'static> Smser<A, T> {
    //短信发送
    pub fn new(
        app_core: Arc<AppCore>,
        redis: deadpool_redis::Pool,
        db: Pool<MySql>,
        task_size: Option<usize>,
        task_timeout: usize,
        is_check: bool,
        acquisition: A,
    ) -> Self {
        let task = Task::new(
            format!("{}-notify", SMSER_REDIS_PREFIX),
            format!("{}-read-lock", SMSER_REDIS_PREFIX),
            format!("{}-run-task", SMSER_REDIS_PREFIX),
            format!("{}-run-num", SMSER_REDIS_PREFIX),
            task_size,
            task_timeout,
            is_check,
            task_timeout,
        );
        Self {
            app_core,
            db,
            redis,
            task,
            acquisition,
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
            .acquisition
            .sms_record()
            .add(mobile, tpl_id, tpl_var, area)
            .await
            .map_err(|e| e.to_string())?;
        let mut redis = self.redis.get().await.map_err(|e| e.to_string())?;
        if let Err(err) = self.task.notify(&mut redis).await {
            warn!("sms is add [{}] ,but send fail :{}", id, err)
        }
        Ok(id)
    }
    //后台发送任务，内部循环不退出
    pub async fn task<S: SmserTaskExecutioner<T, E>, E: TaskExecutioner<u64, SmsTaskItem<T>>>(
        &self,
    ) {
        self.task
            .dispatch(
                self.app_core.clone(),
                &self.acquisition,
                S::create(
                    self.app_core.clone(),
                    self.redis.clone(),
                    self.db.clone(),
                    self.acquisition.sms_record().to_owned(),
                ),
            )
            .await;
    }
}

pub trait SmserTaskExecutioner<T: Send + Sync + 'static, E: TaskExecutioner<u64, SmsTaskItem<T>>>:
    TaskExecutioner<u64, SmsTaskItem<T>>
{
    fn create(
        app_core: Arc<AppCore>,
        redis: deadpool_redis::Pool,
        db: Pool<MySql>,
        record: SmsTaskRecord,
    ) -> E;
}

mod sender_aliyun;
pub use sender_aliyun::*;
