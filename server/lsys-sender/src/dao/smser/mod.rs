use std::collections::HashMap;

use super::{
    SenderResult, {TaskAcquisition, TaskData, TaskItem, TaskRecord},
};
use crate::model::SenderSmsMessageModel;
use async_trait::async_trait;

pub struct SmsTaskItem<T: Send + Sync> {
    pub sms: SenderSmsMessageModel,
    pub attr: T,
}

impl<T: Send + Sync> TaskItem<u64> for SmsTaskItem<T> {
    fn to_task_pk(&self) -> u64 {
        self.sms.id
    }
}

#[async_trait]
pub trait SmsTaskAcquisition<T: Send + Sync>: TaskAcquisition<u64, SmsTaskItem<T>> {
    async fn read_record_attr(
        &self,
        res: Vec<SenderSmsMessageModel>,
    ) -> SenderResult<Vec<SmsTaskItem<T>>>;
    fn sms_record(&self) -> &SmsTaskRecord;
    async fn read_record(
        &self,
        tasking_record: &HashMap<u64, TaskData>,
        limit: usize,
    ) -> SenderResult<TaskRecord<u64, SmsTaskItem<T>>> {
        let (app_res, next) = self.sms_record().read(tasking_record, limit).await?;
        let app_res = self.read_record_attr(app_res).await?;
        Ok(TaskRecord::new(app_res, next))
    }
}

#[async_trait]
pub trait SmserTaskExecutioner<T: Send + Sync>: Sync + Send + 'static {
    async fn exec(&self, val: SmsTaskItem<T>, record: &SmsTaskRecord) -> SenderResult<()>;
}

mod task_record;
pub use task_record::*;
mod sender;
pub use sender::*;
mod sender_aliyun;
pub use sender_aliyun::*;
