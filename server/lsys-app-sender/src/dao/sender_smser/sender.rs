use std::{collections::HashSet, sync::Arc};

use lsys_app::{
    dao::{AppNotify, AppNotifySender},
    model::{AppNotifyTryTimeMode, AppNotifyType},
};
use lsys_core::{
    fluent_message, now_time, AppCore, FluentMessage, IntoFluentMessage, RequestEnv, TaskData,
    TaskDispatchConfig, TaskNotify, TaskNotifyConfig, TaskTimeOutNotify, TimeOutTask,
    TimeOutTaskConfig, TimeOutTaskNotify,
};

use lsys_logger::dao::ChangeLoggerDao;
use lsys_setting::dao::SettingDao;
use sqlx::Pool;
use tracing::warn;

use super::{
    SmsRecord, SmsSendNotify, SmsStatusQuery, SmsStatusTask, SmsStatusTaskAcquisition,
    SmsStatusTaskExecutor, SmsStatusTaskItem, SmsTask, SmsTaskAcquisition, SmsTaskData,
    SmsTaskItem, SmsTaskSendTimeNotify,
};
use crate::{
    dao::{
        MessageCancel, MessageLogs, MessageReader, SenderConfig, SenderError, SenderResult,
        SenderTaskExecutor, SenderTplConfig, SenderWaitNotify,
    },
    model::{SenderSmsBodyModel, SenderSmsMessageModel, SenderSmsMessageStatus, SenderType},
};
use lsys_core::TaskDispatch;

const SMSER_REDIS_PREFIX: &str = "sender-sms";

pub const SMS_NOTIFY_METHOD: &str = "sms_notify";

pub struct SmsSenderConfig {
    pub sender_task_size: Option<usize>,
    pub sender_task_timeout: usize,
    pub notify_task_size: Option<usize>,
    pub notify_task_timeout: usize,
    pub notify_timeout: u64,
    pub wait_timeout: u8,
    //notify config
    pub notify_type: AppNotifyType,
    pub notify_try_max: u8,
    pub notify_try_mode: AppNotifyTryTimeMode,
    pub notify_try_delay: u16,
}

impl Default for SmsSenderConfig {
    fn default() -> Self {
        Self {
            sender_task_size: None,
            notify_task_size: None,
            sender_task_timeout: 300,
            notify_task_timeout: 300,
            notify_timeout: 1800,
            wait_timeout: 30, //接口回调等待超时
            notify_type: AppNotifyType::Http,
            notify_try_max: 3,
            notify_try_mode: AppNotifyTryTimeMode::Exponential,
            notify_try_delay: 60,
        }
    }
}

pub struct SmsSenderDao {
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

    task_status: TaskDispatch<u64, SmsStatusTaskItem>,
    task_status_notify: Arc<TaskNotify>,
    send_wait: Arc<SenderWaitNotify>,
    task_status_key: String,
    setting: Arc<SettingDao>,
    app_notify_sender: Arc<AppNotifySender>,
    task_sender: Arc<TaskDispatch<u64, SmsTaskItem>>,
    task_sender_task_timeout_notify: Arc<TaskTimeOutNotify>,
    task_sender_notify: Arc<TaskNotify>,
    task_sender_sendtime_notify: Arc<TimeOutTaskNotify>,
}

impl SmsSenderDao {
    //短信发送
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        app_core: Arc<AppCore>,
        redis: deadpool_redis::Pool,
        db: Pool<sqlx::MySql>,
        setting: Arc<SettingDao>,
        logger: Arc<ChangeLoggerDao>,
        app_notify: Arc<AppNotify>,
        sms_config: SmsSenderConfig,
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
        ));
        let sms_record = Arc::new(SmsRecord::new(
            db.clone(),
            config,
            logger,
            message_logs.clone(),
            message_reader.clone(),
        ));

        //任务触发句柄 && 任务后台任务-- start
        let task_sender_notify_config = Arc::new(TaskNotifyConfig::new(format!(
            "{}-sender",
            SMSER_REDIS_PREFIX
        )));
        let task_sender_notify = Arc::new(TaskNotify::new(
            redis.clone(),
            task_sender_notify_config.clone(),
        ));

        let task_sender_display_config = Arc::new(TaskDispatchConfig::new(
            task_sender_notify_config,
            sms_config.sender_task_timeout,
            true,
            sms_config.sender_task_size,
        ));
        let task_sender = Arc::new(TaskDispatch::new(
            redis.clone(),
            task_sender_notify.clone(),
            task_sender_display_config.clone(),
        ));
        //任务触发句柄 && 任务后台任务-- end

        //定时发送任务触发句柄 && 定时发送后台任务 --start
        let task_sender_sendtime_notify = Arc::new(TimeOutTaskNotify::new(
            redis.clone(),
            TimeOutTaskConfig::new(
                format!("{}-notify-timeout", SMSER_REDIS_PREFIX),
                sms_config.notify_task_timeout,
            ),
        ));

        let task_sender_task_timeout_notify = Arc::new(TaskTimeOutNotify::new(
            app_core.clone(),
            task_sender_notify.clone(),
            task_sender_sendtime_notify.clone(),
            task_sender_display_config,
        ));
        //定时发送任务触发句柄 && 定时发送后台任务 --end

        //后台同步短信发送结果触发句柄 && 后台同步短信发送结果同步任务 --start
        let task_status_notify_config = Arc::new(TaskNotifyConfig::new(format!(
            "{}-status",
            SMSER_REDIS_PREFIX
        )));
        let task_status_notify = Arc::new(TaskNotify::new(
            redis.clone(),
            task_status_notify_config.clone(),
        ));
        let task_status_display_config = Arc::new(TaskDispatchConfig::new(
            task_status_notify_config,
            sms_config.notify_task_timeout,
            true,
            sms_config.notify_task_size,
        ));
        let task_status = TaskDispatch::new(
            redis.clone(),
            task_status_notify.clone(),
            task_status_display_config.clone(),
        );
        //后台同步短信发送结果触发句柄 && 后台同步短信发送结果同步任务 --end

        let app_notify_sender = Arc::new(app_notify.sender_create(
            SMS_NOTIFY_METHOD,
            sms_config.notify_type,
            sms_config.notify_try_max,
            sms_config.notify_try_mode,
            sms_config.notify_try_delay,
            true,
        ));

        let task_status_key = format!("{}-status-data", SMSER_REDIS_PREFIX);
        let status_query = Arc::new(SmsStatusQuery::new(
            redis.clone(),
            &task_status_key,
            sms_config.notify_timeout,
        ));

        //发送结果等待
        let wait_status_key = format!("{}-status-data", SMSER_REDIS_PREFIX);
        let send_wait = Arc::new(SenderWaitNotify::new(
            &wait_status_key,
            redis.clone(),
            app_core.clone(),
            sms_config.wait_timeout,
        ));

        let sms_notify = Arc::new(SmsSendNotify::new(db.clone(), app_notify_sender.clone()));
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
            send_wait,
            app_notify_sender,
            task_sender_task_timeout_notify,
            task_sender_notify,
            task_sender_sendtime_notify,
            task_status_notify,
        }
    }
    pub async fn add_status_query(&self, items: &[&SenderSmsMessageModel]) -> SenderResult<()> {
        self.status_query.add_query(items).await?;
        if let Err(err) = self.task_status_notify.notify().await {
            warn!(
                "add status query task fail :{}",
                err.to_fluent_message().default_format()
            )
        }
        Ok(())
    }
    //发送模板消息
    #[allow(clippy::too_many_arguments)]
    pub async fn send<'t>(
        &self,
        app_id: Option<u64>,
        mobiles: &[(&'t str, &'t str)],
        tpl_key: &str,
        tpl_var: &str,
        send_time: Option<u64>,
        user_id: Option<u64>,
        max_try_num: Option<u8>,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<(
        u64,
        Vec<(u64, &'t str, &'t str, Result<bool, FluentMessage>)>,
    )> {
        let tmp = mobiles
            .iter()
            .collect::<HashSet<_>>()
            .into_iter()
            .map(|e| (e.0, e.1))
            .collect::<Vec<(&str, &str)>>();
        let max_try_num = max_try_num.unwrap_or(1);
        let nt = now_time().unwrap_or_default();
        let sendtime = send_time.unwrap_or(nt);
        let sendtime = if sendtime < nt { nt } else { sendtime };
        self.sms_record
            .send_check(app_id, tpl_key, &tmp, sendtime)
            .await?;
        let res = self
            .sms_record
            .add(
                &tmp,
                app_id.unwrap_or_default(),
                tpl_key,
                tpl_var,
                sendtime,
                user_id,
                max_try_num,
                env_data,
            )
            .await?;

        let mut wait = None;
        if max_try_num == 0 && mobiles.len() == 1 {
            if let Some((snid, _, _)) = res.1.first() {
                wait = Some(self.send_wait.message_wait(res.0, *snid).await);
            }
        };

        if let Err(err) = self
            .task_sender_task_timeout_notify
            .notify_at_time(sendtime)
            .await
        {
            warn!(
                "mail is add [{}] ,but notify fail :{}",
                res.0,
                err.to_fluent_message().default_format()
            )
        }

        let mut tmp = vec![];
        for e in mobiles {
            let msg_id = res
                .1
                .iter()
                .find(|t| t.1 == e.0 && t.2 == e.1)
                .map(|e| e.0)
                .unwrap_or_default();
            let item_res = if let Some(t) = wait.take() {
                self.send_wait
                    .wait_timeout(t)
                    .await
                    .map(|e| e.map_err(|c| fluent_message!("sms-send-fail", c)))
                    .unwrap_or_else(|e| {
                        Err(fluent_message!("sms-send-wait-fail", e.to_fluent_message()))
                    })
            } else {
                Ok(true)
            };
            tmp.push((msg_id, e.0, e.1, item_res));
        }
        Ok((res.0, tmp))
    }
    pub async fn cancal_from_message_snid_vec(
        &self,
        msg_snid_data: &[u64],
        user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<Vec<(u64, bool, Option<SenderError>)>> {
        let res = self
            .message_reader
            .find_message_by_snid_vec(msg_snid_data)
            .await?;
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
        user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<Vec<(u64, bool, Option<SenderError>)>> {
        let mut out = Vec::with_capacity(msg_data.len());
        let mut cancel_data = vec![];
        for tmp in msg_data {
            if SenderSmsMessageStatus::Init.eq(tmp.status) {
                cancel_data.push(tmp);
                continue;
            }
            if SenderSmsMessageStatus::IsCancel.eq(tmp.status) {
                out.push((tmp.snid, false, None));
                continue;
            }
            out.push((
                tmp.snid,
                false,
                Some(SenderError::System(fluent_message!(
                    "sms-send-cancel-status-bad"
                ))),
            ));
        }
        self.cancel
            .add(
                body.app_id,
                body.id,
                &cancel_data.iter().map(|e| e.id).collect::<Vec<_>>(),
                user_id,
                None,
            )
            .await?;

        for (msg, task_data) in self
            .task_is_run(cancel_data.iter().map(|e| (&e.id, *e)).collect::<Vec<_>>())
            .await?
        {
            let err = if task_data.is_none() {
                self.sms_record
                    .cancel_form_message(body, msg, user_id, env_data)
                    .await
                    .err()
            } else {
                Some(SenderError::System(
                    fluent_message!("sms-send-cancel-is-ing", //  "sms {} is sending:{}",
                        {
                            "mobile":&msg.mobile,
                            "msg_id":msg.id
                        }
                    ),
                ))
            };
            out.push((msg.snid, task_data.is_some(), err))
        }
        Ok(out)
    }
    //检查指定任务是否发送中
    pub async fn task_is_run<D>(
        &self,
        check_message_data: Vec<(&u64, D)>,
    ) -> SenderResult<Vec<(D, Option<TaskData>)>> {
        let mut tdata = self.task_sender.task_data().await?;
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
            self.send_wait.clone(),
            self.message_logs.clone(),
            self.message_reader.clone(),
        ));
        self.task_sender
            .dispatch(
                self.app_core.clone(),
                acquisition.as_ref(),
                Arc::new(SmsTask::new(
                    acquisition.to_owned(),
                    self.tpl_config.clone(),
                    se,
                )?),
            )
            .await;
        Ok(())
    }
    //发送等待回调处理监听
    pub async fn task_wait(&self) {
        self.send_wait.listen().await;
    }
    //指定发送时间到期监听处理
    pub async fn task_sendtime_notify(&self, channel_buffer: Option<usize>) {
        let task_send_time = Arc::new(SmsTaskSendTimeNotify::new(
            format!("{}-last-run-time", SMSER_REDIS_PREFIX),
            self.db.clone(),
            self.redis.clone(),
            self.task_sender_notify.clone(),
        ));
        TimeOutTask::<SmsTaskSendTimeNotify>::new(
            self.app_core.clone(),
            self.task_sender_sendtime_notify.clone(),
            task_send_time.clone(),
            task_send_time,
        )
        .listen(channel_buffer)
        .await;
    }
    //后台同步短信发送结果任务，内部循环不退出
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
                Arc::new(SmsStatusTask::new(
                    se,
                    self.sms_record.clone(),
                    self.db.clone(),
                    self.app_notify_sender.clone(),
                    self.setting.multiple.clone(),
                    self.message_logs.clone(),
                )?),
            )
            .await;
        Ok(())
    }
}
