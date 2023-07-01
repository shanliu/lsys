use std::{
    collections::HashMap,
    sync::{atomic::AtomicU32, Arc},
};

use async_trait::async_trait;
use lsys_core::now_time;

use lsys_setting::model::SettingModel;
use sqlx::{MySql, Pool};
use sqlx_model::Update;

use crate::{
    dao::{
        group_exec, MessageLogs, MessageReader, SenderError, SenderExecError, SenderResult,
        SenderTaskExecutor, SenderTaskExecutorBox, SenderTaskItem, SenderTplConfig,
        TaskAcquisition, TaskData, TaskExecutor, TaskItem, TaskRecord,
    },
    model::{
        SenderLogStatus, SenderSmsMessageModel, SenderSmsMessageModelRef, SenderSmsMessageStatus,
        SenderTplConfigModel,
    },
};

use super::SmsRecord;

pub struct SmsTaskItem {
    pub sms: SenderSmsMessageModel,
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
        self.sms.tpl_id.to_owned()
    }
}
#[async_trait]
pub trait SmsTaskExecutor: Sync + Send + 'static {
    async fn exec(
        &self,
        val: &SmsTaskItem,
        tpl_config: &SenderTplConfigModel,
        setting: &SettingModel,
        record: &SmsRecord,
    ) -> Result<(), SenderExecError>;
    fn setting_key(&self) -> String;
}

pub struct SmsTaskAcquisition {
    db: Pool<sqlx::MySql>,
    message_logs: Arc<MessageLogs>,
    message_reader: Arc<MessageReader<SenderSmsMessageModel>>,
}
impl SmsTaskAcquisition {
    pub fn new(
        db: Pool<sqlx::MySql>,
        message_logs: Arc<MessageLogs>,
        message_reader: Arc<MessageReader<SenderSmsMessageModel>>,
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
        item: &SmsTaskItem,
        res: &Result<(String, SenderTplConfigModel, SettingModel), String>,
    ) -> Result<(), String> {
        let val = &item.sms;
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
            SenderSmsMessageStatus::IsSend as i8
        } else {
            SenderSmsMessageStatus::SendFail as i8
        };
        let set_try_num = val.try_num + 1;
        let send_time = now_time().unwrap_or_default();
        let mut change = sqlx_model::model_option_set!(SenderSmsMessageModelRef,{
            send_time:send_time,
            try_num:set_try_num
        });
        if SenderSmsMessageStatus::IsSend.eq(status)
            || (SenderSmsMessageStatus::SendFail.eq(status) && val.try_num + 1 >= val.max_try_num)
        {
            change.status = Some(&status);
        }
        Update::<MySql, SenderSmsMessageModel, _>::new(change)
            .execute_by_pk(val, &self.db)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[async_trait]
impl TaskAcquisition<u64, SmsTaskItem> for SmsTaskAcquisition {
    //复用父结构体方法实现
    async fn read_record(
        &self,
        tasking_record: &HashMap<u64, TaskData>,
        limit: usize,
    ) -> Result<TaskRecord<u64, SmsTaskItem>, String> {
        let (app_res, next) = self
            .message_reader
            .read(tasking_record, SenderSmsMessageStatus::Init as i8, limit)
            .await
            .map_err(|e| e.to_string())?;
        let app_res = app_res
            .into_iter()
            .map(|e| SmsTaskItem { sms: e })
            .collect();
        Ok(TaskRecord::new(app_res, next))
    }
}

#[derive(Clone)]
pub struct SmsTask {
    inner: Arc<Vec<SenderTaskExecutorBox<u64, SmsTaskItem>>>,
    acquisition: Arc<SmsTaskAcquisition>,
    tpl_config: Arc<SenderTplConfig>,
    i: Arc<AtomicU32>,
}

impl SmsTask {
    pub fn new(
        acquisition: Arc<SmsTaskAcquisition>,
        tpl_config: Arc<SenderTplConfig>,
        se: Vec<Box<dyn SenderTaskExecutor<u64, SmsTaskItem>>>,
    ) -> SenderResult<SmsTask> {
        if se.is_empty() {
            return Err(SenderError::System("can't set task is empty".to_string()));
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
        self.acquisition
            .finish(
                &val,
                &group_exec(&val, &self.i, &self.tpl_config, self.inner.as_ref()).await,
            )
            .await
    }
}
