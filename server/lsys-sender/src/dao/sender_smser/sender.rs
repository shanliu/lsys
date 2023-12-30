use std::{collections::HashSet, sync::Arc};

use lsys_core::{now_time, AppCore, FluentMessage, RequestEnv, TaskData};

use lsys_logger::dao::ChangeLogger;
use lsys_notify::dao::Notify;
use lsys_setting::dao::Setting;
use sqlx::Pool;
use tracing::warn;

use super::{
    SmsRecord, SmsSendNotify, SmsStatusQuery, SmsStatusTask, SmsStatusTaskAcquisition,
    SmsStatusTaskExecutor, SmsStatusTaskItem, SmsTask, SmsTaskAcquisition, SmsTaskData,
    SmsTaskItem,
};
use crate::{
    dao::{
        MessageCancel, MessageLogs, MessageReader, SenderConfig, SenderError, SenderResult,
        SenderTaskExecutor, SenderTplConfig,
    },
    model::{SenderSmsBodyModel, SenderSmsMessageModel, SenderType},
};
use lsys_core::TaskDispatch;

const SMSER_REDIS_PREFIX: &str = "sender-sms-";

pub struct SmsSender {
    pub tpl_config: Arc<SenderTplConfig>,
    pub sms_record: Arc<SmsRecord>,
    pub sms_notify: Arc<SmsSendNotify>,
    status_query: Arc<SmsStatusQuery>,
    redis: deadpool_redis::Pool,
    db: Pool<sqlx::MySql>,
    app_core: Arc<AppCore>,
    cancel: Arc<MessageCancel>,
    message_logs: Arc<MessageLogs>,
    message_reader: Arc<MessageReader<SenderSmsBodyModel, SenderSmsMessageModel>>,
    task_sender: TaskDispatch<u64, SmsTaskItem>,
    task_status: TaskDispatch<u64, SmsStatusTaskItem>,
    task_status_key: String,
    setting: Arc<Setting>,
    notify: Arc<Notify>,
}

impl SmsSender {
    //短信发送
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        app_core: Arc<AppCore>,
        redis: deadpool_redis::Pool,
        db: Pool<sqlx::MySql>,
        setting: Arc<Setting>,
        fluent: Arc<FluentMessage>,
        logger: Arc<ChangeLogger>,
        notify: Arc<Notify>,
        sender_task_size: Option<usize>,
        notify_task_size: Option<usize>,
        task_timeout: usize,
        is_check: bool,
        notify_timeout: Option<u64>,
    ) -> Self {
        let config: Arc<SenderConfig> = Arc::new(SenderConfig::new(
            db.clone(),
            logger.clone(),
            SenderType::Smser,
        ));
        let tpl_config = Arc::new(SenderTplConfig::new(
            db.clone(),
            setting.clone(),
            logger.clone(),
            SenderType::Smser,
        ));
        let cancel = Arc::new(MessageCancel::new(db.clone(), SenderType::Smser));
        let message_logs = Arc::new(MessageLogs::new(db.clone(), SenderType::Smser));
        let message_reader = Arc::new(MessageReader::new(
            db.clone(),
            app_core.clone(),
            SenderType::Smser,
            fluent,
        ));
        let sms_record = Arc::new(SmsRecord::new(
            db.clone(),
            config,
            logger,
            message_logs.clone(),
            message_reader.clone(),
        ));

        let task_sender = TaskDispatch::new(
            format!("{}-sender-notify", SMSER_REDIS_PREFIX),
            format!("{}-sender-read-lock", SMSER_REDIS_PREFIX),
            format!("{}-sender-run-task", SMSER_REDIS_PREFIX),
            sender_task_size,
            task_timeout,
            is_check,
            task_timeout,
        );

        let task_status = TaskDispatch::new(
            format!("{}-status-notify", SMSER_REDIS_PREFIX),
            format!("{}-status-read-lock", SMSER_REDIS_PREFIX),
            format!("{}-status-run-task", SMSER_REDIS_PREFIX),
            notify_task_size,
            task_timeout,
            is_check,
            task_timeout,
        );
        let task_status_key = format!("{}-status-data", SMSER_REDIS_PREFIX);
        let sms_notify = Arc::new(SmsSendNotify::new(db.clone(), notify.clone()));
        let status_query = Arc::new(SmsStatusQuery::new(
            redis.clone(),
            task_status_key.clone(),
            notify_timeout.unwrap_or(1800),
        ));

        Self {
            tpl_config,
            redis,
            sms_record,
            app_core,
            db,
            message_logs,
            message_reader,
            task_sender,
            task_status,
            cancel,
            task_status_key,
            setting,
            sms_notify,
            status_query,
            notify,
        }
    }
    pub async fn add_status_query(&self, items: &[&SenderSmsMessageModel]) -> SenderResult<()> {
        self.status_query.add_query(items).await?;
        let mut redis = self.redis.get().await?;
        if let Err(err) = self.task_status.notify(&mut redis).await {
            warn!("add status query task fail :{}", err)
        }
        Ok(())
    }
    //发送模板消息
    #[allow(clippy::too_many_arguments)]
    pub async fn send<'t>(
        &self,
        app_id: Option<u64>,
        mobiles: &[(&'t str, &'t str)],
        tpl_id: &str,
        tpl_var: &str,
        send_time: &Option<u64>,
        user_id: &Option<u64>,
        max_try_num: &Option<u8>,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<(u64, Vec<(u64, &'t str, &'t str)>)> {
        let tmp = mobiles
            .iter()
            .collect::<HashSet<_>>()
            .into_iter()
            .map(|e| (e.0, e.1))
            .collect::<Vec<(&str, &str)>>();

        let nt = now_time().unwrap_or_default();
        let sendtime = send_time.unwrap_or(nt);
        let sendtime = if sendtime < nt { nt } else { sendtime };
        self.sms_record
            .send_check(app_id, tpl_id, &tmp, sendtime)
            .await?;
        let res = self
            .sms_record
            .add(
                &tmp,
                &app_id.unwrap_or_default(),
                tpl_id,
                tpl_var,
                &sendtime,
                user_id,
                max_try_num,
                env_data,
            )
            .await?;
        if send_time
            .map(|e| e - 1 <= now_time().unwrap_or_default())
            .unwrap_or(true)
        {
            let mut redis = self.redis.get().await?;
            if let Err(err) = self.task_sender.notify(&mut redis).await {
                warn!("sms is add [{}] ,but send fail :{}", res.0, err)
            }
        }
        let tmp = mobiles
            .iter()
            .map(|e| {
                (
                    res.1
                        .iter()
                        .find(|t| t.1 == e.0 && t.2 == e.1)
                        .map(|e| e.0)
                        .unwrap_or_default(),
                    e.0,
                    e.1,
                )
            })
            .collect::<Vec<_>>();
        Ok((res.0, tmp))
    }
    pub async fn cancal_from_message_id_vec(
        &self,
        msg_data: &[u64],
        user_id: &u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<Vec<(u64, bool, Option<SenderError>)>> {
        let res = self.message_reader.find_message_by_id_vec(msg_data).await?;
        if res.is_empty() {
            return Ok(vec![]);
        }
        let b_res_id = res.iter().map(|e| e.sender_body_id).collect::<Vec<_>>();
        let b_res = self.message_reader.find_body_by_id_vec(&b_res_id).await?;
        let mut out = vec![];
        for b_tmp in b_res {
            let tmp = res
                .iter()
                .filter(|c| c.sender_body_id == b_tmp.id)
                .collect::<Vec<_>>();

            out.extend(
                self.cancal_from_message(&b_tmp, &tmp, user_id, env_data)
                    .await?,
            )
        }
        Ok(out)
    }
    //通过ID取消发送
    pub async fn cancal_from_message(
        &self,
        body: &SenderSmsBodyModel,
        msg_data: &[&SenderSmsMessageModel],
        user_id: &u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<Vec<(u64, bool, Option<SenderError>)>> {
        self.cancel
            .add(
                &body.app_id,
                &body.id,
                &msg_data.iter().map(|e| e.id).collect::<Vec<_>>(),
                user_id,
                None,
            )
            .await?;

        let mut out = Vec::with_capacity(msg_data.len());
        for (msg, task_data) in self
            .task_is_run(msg_data.iter().map(|e| (&e.id, *e)).collect::<Vec<_>>())
            .await?
        {
            let err = if task_data.is_none() {
                self.sms_record
                    .cancel_form_message(body, msg, user_id, env_data)
                    .await
                    .err()
            } else {
                Some(SenderError::System(format!(
                    "sms {} is sending:{}",
                    msg.mobile, msg.id
                )))
            };
            out.push((msg.id, task_data.is_none(), err))
        }
        Ok(out)
    }
    //检查指定任务是否发送中
    pub async fn task_is_run<D>(
        &self,
        check_message_data: Vec<(&u64, D)>,
    ) -> SenderResult<Vec<(D, Option<TaskData>)>> {
        let mut redis = self.redis.get().await?;
        let mut tdata = self.task_sender.task_data(&mut redis).await?;
        let mut out = Vec::with_capacity(check_message_data.len());
        for (mid, data) in check_message_data {
            out.push((data, tdata.remove(mid)));
        }
        Ok(out)
    }
    //后台发送任务，内部循环不退出
    pub async fn task_sender(
        &self,
        se: Vec<Box<dyn SenderTaskExecutor<u64, SmsTaskItem, SmsTaskData>>>,
    ) -> SenderResult<()> {
        let acquisition = Arc::new(SmsTaskAcquisition::new(
            self.db.clone(),
            self.message_logs.clone(),
            self.message_reader.clone(),
        ));
        self.task_sender
            .dispatch(
                self.app_core.clone(),
                acquisition.as_ref(),
                SmsTask::new(acquisition.to_owned(), self.tpl_config.clone(), se)?,
            )
            .await;
        Ok(())
    }
    //后台发送任务，内部循环不退出
    pub async fn task_status_query(
        &self,
        se: Vec<Box<dyn SmsStatusTaskExecutor>>,
    ) -> SenderResult<()> {
        let acquisition =
            SmsStatusTaskAcquisition::new(self.redis.clone(), self.task_status_key.clone());
        self.task_status
            .dispatch(
                self.app_core.clone(),
                &acquisition,
                SmsStatusTask::new(
                    se,
                    self.sms_record.clone(),
                    self.db.clone(),
                    self.notify.clone(),
                    self.setting.multiple.clone(),
                    self.message_logs.clone(),
                )?,
            )
            .await;
        Ok(())
    }
}
