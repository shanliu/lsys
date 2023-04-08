use std::collections::HashMap;

use crate::model::SenderSmsMessageModel;
use async_trait::async_trait;
use lsys_core::now_time;

use super::{
    task::{TaskAcquisition, TaskExecutioner, TaskItem, TaskRecord, TaskValue},
    SenderResult,
};

pub struct SmsTaskItem<T: Send + Sync + 'static + Clone> {
    pub sms: SenderSmsMessageModel,
    pub attr: T,
}

impl<T: Send + Sync + 'static + Clone> TaskItem<u64> for SmsTaskItem<T> {
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
pub trait SmsTaskAcquisition<T: Send + Sync + 'static + Clone>:
    TaskAcquisition<u64, SmsTaskItem<T>> + Clone
{
    async fn read_record_attr(
        &self,
        res: Vec<SenderSmsMessageModel>,
    ) -> SenderResult<Vec<SmsTaskItem<T>>>;
    fn sms_record(&self) -> &SmsTaskRecord;
    async fn read_record(
        &self,
        tasking_record: &HashMap<u64, TaskValue>,
        limit: usize,
    ) -> SenderResult<TaskRecord<u64, SmsTaskItem<T>>> {
        let (app_res, next) = self.sms_record().read(tasking_record, limit).await?;
        let app_res = self.read_record_attr(app_res).await?;
        Ok(TaskRecord::new(app_res, next))
    }
}

#[async_trait]
pub trait SmserTaskExecutioner<T: Send + Sync + 'static + Clone>:
    Sync + Send + 'static + Clone
{
    async fn exec(&self, val: SmsTaskItem<T>, record: &SmsTaskRecord) -> SenderResult<()>;
}

#[derive(Clone)]
pub struct SmserTask<T: Send + Sync + 'static + Clone, SE: SmserTaskExecutioner<T>> {
    inner: SE,
    record: SmsTaskRecord,
    marker_t: std::marker::PhantomData<T>,
}

impl<T: Send + Sync + 'static + Clone, SE: SmserTaskExecutioner<T>> SmserTask<T, SE> {
    pub fn new(record: SmsTaskRecord, se: SE) -> SmserTask<T, SE> {
        SmserTask {
            inner: se,
            record,
            marker_t: std::marker::PhantomData::default(),
        }
    }
}

#[async_trait]
impl<T: Send + Sync + 'static + Clone, SE: SmserTaskExecutioner<T>>
    TaskExecutioner<u64, SmsTaskItem<T>> for SmserTask<T, SE>
{
    async fn exec(&self, val: SmsTaskItem<T>) -> SenderResult<()> {
        self.inner.exec(val, &self.record).await
    }
}

mod task_record;
pub use task_record::*;
mod sender;
pub use sender::*;
mod sender_aliyun;
pub use sender_aliyun::*;
