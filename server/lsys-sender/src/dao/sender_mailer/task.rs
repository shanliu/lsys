use std::{
    collections::HashMap,
    sync::{atomic::AtomicU32, Arc},
};

use crate::{
    dao::{
        group_exec, MessageLogs, MessageReader, SenderError, SenderResult, SenderTaskExecutor,
        SenderTaskExecutorBox, SenderTaskItem, SenderTplConfig, TaskAcquisition, TaskData,
        TaskExecutor, TaskItem, TaskRecord,
    },
    model::{
        SenderLogStatus, SenderMailMessageModel, SenderMailMessageModelRef,
        SenderMailMessageStatus, SenderTplConfigModel,
    },
};
use async_trait::async_trait;
use lsys_core::now_time;

use lsys_setting::model::SettingModel;

use sqlx::{MySql, Pool};
use sqlx_model::Update;

//短信任务记录

pub struct MailTaskItem {
    pub mail: SenderMailMessageModel,
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

pub struct MailTaskAcquisition {
    db: Pool<sqlx::MySql>,
    message_logs: Arc<MessageLogs>,
    message_reader: Arc<MessageReader<SenderMailMessageModel>>,
}

impl MailTaskAcquisition {
    pub fn new(
        db: Pool<sqlx::MySql>,
        message_logs: Arc<MessageLogs>,
        message_reader: Arc<MessageReader<SenderMailMessageModel>>,
    ) -> Self {
        Self {
            db,
            message_logs,
            message_reader,
        }
    }
    //完成指定短信任务回调
    pub async fn finish(
        &self,
        item: &MailTaskItem,
        res: &Result<(String, SenderTplConfigModel, SettingModel), String>,
    ) -> Result<(), String> {
        let val = &item.mail;
        let ok = res.is_ok();
        let (send_note, message) = match res {
            Ok((s, _, _)) => (s.as_str(), ""),
            Err(err) => ("", err.as_str()),
        };
        self.message_logs
            .add_finish_log(
                val.app_id,
                val.id,
                &if ok {
                    SenderLogStatus::Succ
                } else {
                    SenderLogStatus::Fail
                },
                send_note,
                message,
            )
            .await;
        let status = if ok {
            SenderMailMessageStatus::IsSend as i8
        } else {
            SenderMailMessageStatus::SendFail as i8
        };
        let set_try_num = val.try_num + 1;
        let send_time = now_time().unwrap_or_default();
        let mut change = sqlx_model::model_option_set!(SenderMailMessageModelRef,{
            send_time:send_time,
            try_num:set_try_num
        });
        if SenderMailMessageStatus::IsSend.eq(status)
            || (SenderMailMessageStatus::SendFail.eq(status) && val.try_num + 1 >= val.max_try_num)
        {
            change.status = Some(&status);
        }
        Update::<MySql, SenderMailMessageModel, _>::new(change)
            .execute_by_pk(val, &self.db)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[async_trait]
impl TaskAcquisition<u64, MailTaskItem> for MailTaskAcquisition {
    //复用父结构体方法实现
    async fn read_record(
        &self,
        tasking_record: &HashMap<u64, TaskData>,
        limit: usize,
    ) -> Result<TaskRecord<u64, MailTaskItem>, String> {
        let (app_res, next) = self
            .message_reader
            .read(tasking_record, SenderMailMessageStatus::Init as i8, limit)
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
    inner: Arc<Vec<SenderTaskExecutorBox<u64, MailTaskItem>>>,
    acquisition: Arc<MailTaskAcquisition>,
    tpl_config: Arc<SenderTplConfig>,
    i: Arc<AtomicU32>,
}

impl MailerTask {
    pub fn new(
        acquisition: Arc<MailTaskAcquisition>,
        tpl_config: Arc<SenderTplConfig>,
        se: Vec<Box<dyn SenderTaskExecutor<u64, MailTaskItem>>>,
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
        self.acquisition
            .finish(
                &val,
                &group_exec(&val, &self.i, &self.tpl_config, self.inner.as_ref()).await,
            )
            .await
    }
}
