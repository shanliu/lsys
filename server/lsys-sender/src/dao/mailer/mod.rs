use std::collections::HashMap;

use super::{
    SenderResult, {TaskAcquisition, TaskData, TaskItem, TaskRecord},
};
use crate::model::SenderMailMessageModel;
use async_trait::async_trait;

pub struct MailTaskItem<T: Send + Sync> {
    pub mail: SenderMailMessageModel,
    pub attr: T,
}

impl<T: Send + Sync> TaskItem<u64> for MailTaskItem<T> {
    fn to_task_pk(&self) -> u64 {
        self.mail.id
    }
}

#[async_trait]
pub trait MailTaskAcquisition<T: Send + Sync>: TaskAcquisition<u64, MailTaskItem<T>> {
    async fn read_record_attr(
        &self,
        res: Vec<SenderMailMessageModel>,
    ) -> SenderResult<Vec<MailTaskItem<T>>>;
    fn sms_record(&self) -> &MailTaskRecord;
    async fn read_record(
        &self,
        tasking_record: &HashMap<u64, TaskData>,
        limit: usize,
    ) -> SenderResult<TaskRecord<u64, MailTaskItem<T>>> {
        let (app_res, next) = self.sms_record().read(tasking_record, limit).await?;
        let app_res = self.read_record_attr(app_res).await?;
        Ok(TaskRecord::new(app_res, next))
    }
}

#[async_trait]
pub trait MailerTaskExecutioner<T: Send + Sync>: Sync + Send + 'static {
    async fn exec(&self, val: MailTaskItem<T>, record: &MailTaskRecord) -> SenderResult<()>;
}

mod task_record;
pub use task_record::*;
mod sender;
pub use sender::*;
mod sender_smtp;
pub use sender_smtp::*;
