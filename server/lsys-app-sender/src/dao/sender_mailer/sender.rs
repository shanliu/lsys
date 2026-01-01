use std::{collections::HashSet, sync::Arc};

use lsys_core::{
    fluent_message, now_time, AppCore, FluentMessage, RequestEnv, TaskData, TaskDispatchConfig,
    TaskNotify, TaskNotifyConfig, TaskTimeOutNotify, TimeOutTask, TimeOutTaskConfig,
    TimeOutTaskNotify,
};

use lsys_logger::dao::ChangeLoggerDao;
use lsys_setting::dao::SettingDao;
use sqlx::Pool;
use tracing::{debug, warn};

use super::{
    MailRecord, MailTaskAcquisition, MailTaskData, MailTaskItem, MailTaskSendTimeNotify, MailerTask,
};
use crate::{
    dao::{
        MessageCancel, MessageLogs, MessageReader, SenderConfig, SenderError, SenderResult,
        SenderTaskExecutor, SenderTplConfig, SenderWaitNotify,
    },
    model::{SenderMailBodyModel, SenderMailMessageModel, SenderMailMessageStatus, SenderType},
};
use lsys_core::IntoFluentMessage;
use lsys_core::TaskDispatch;

const MAILER_REDIS_PREFIX: &str = "sender-mail";

pub struct MailSenderConfig {
    pub sender_task_size: Option<usize>,
    pub sender_task_timeout: usize,
    pub notify_task_timeout: usize,
    pub wait_timeout: u8,
}

impl Default for MailSenderConfig {
    fn default() -> Self {
        Self {
            sender_task_size: None,
            sender_task_timeout: 300,
            notify_task_timeout: 300,
            wait_timeout: 30, //回调等待超时
        }
    }
}

pub struct MailSenderDao {
    pub tpl_config: Arc<SenderTplConfig>,
    pub mail_record: Arc<MailRecord>,
    db: Pool<sqlx::MySql>,
    redis: deadpool_redis::Pool,
    app_core: Arc<AppCore>,
    message_logs: Arc<MessageLogs>,
    cancel: Arc<MessageCancel>,
    message_reader: Arc<MessageReader<SenderMailBodyModel, SenderMailMessageModel>>,
    task: Arc<TaskDispatch<u64, MailTaskItem>>,
    send_wait: Arc<SenderWaitNotify>,
    task_notify: Arc<TaskNotify>,
    task_timeout_notify: Arc<TaskTimeOutNotify>,
    time_out_notify: Arc<TimeOutTaskNotify>,
}

impl MailSenderDao {
    //发送
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        app_core: Arc<AppCore>,
        redis: deadpool_redis::Pool,
        db: Pool<sqlx::MySql>,
        setting: Arc<SettingDao>,
        logger: Arc<ChangeLoggerDao>,
        mail_config: MailSenderConfig,
    ) -> Self {
        let config: Arc<SenderConfig> = Arc::new(SenderConfig::new(
            db.clone(),
            logger.clone(),
            SenderType::Mailer,
        ));
        let tpl_config = Arc::new(SenderTplConfig::new(
            db.clone(),
            setting,
            logger.clone(),
            SenderType::Mailer,
        ));
        let cancel = Arc::new(MessageCancel::new(db.clone(), SenderType::Mailer));
        let message_logs = Arc::new(MessageLogs::new(db.clone(), SenderType::Mailer));
        let message_reader = Arc::new(MessageReader::new(
            db.clone(),
            app_core.clone(),
            SenderType::Mailer,
        ));
        let mail_record = Arc::new(MailRecord::new(
            db.clone(),
            config,
            logger,
            message_logs.clone(),
            message_reader.clone(),
        ));

        let task_notify_config = Arc::new(TaskNotifyConfig::new(format!(
            "{}-sender",
            MAILER_REDIS_PREFIX
        )));
        let task_notify = Arc::new(TaskNotify::new(redis.clone(), task_notify_config.clone()));
        let display_config = Arc::new(TaskDispatchConfig::new(
            task_notify_config,
            mail_config.sender_task_timeout,
            true,
            mail_config.sender_task_size,
        ));
        let task = Arc::new(TaskDispatch::new(
            redis.clone(),
            task_notify.clone(),
            display_config.clone(),
        ));

        let time_out_notify = Arc::new(TimeOutTaskNotify::new(
            redis.clone(),
            TimeOutTaskConfig::new(
                format!("{}-notify-timeout", MAILER_REDIS_PREFIX),
                mail_config.notify_task_timeout,
            ),
        ));

        let task_timeout_notify = Arc::new(TaskTimeOutNotify::new(
            app_core.clone(),
            task_notify.clone(),
            time_out_notify.clone(),
            display_config,
        ));

        let wait_status_key = format!("{}-status-data", MAILER_REDIS_PREFIX);
        let send_wait = Arc::new(SenderWaitNotify::new(
            &wait_status_key,
            redis.clone(),
            app_core.clone(),
            mail_config.wait_timeout,
        ));

        Self {
            tpl_config,
            mail_record,
            app_core,
            db,
            redis,
            message_logs,
            message_reader,
            task,
            send_wait,
            cancel,
            task_timeout_notify,
            time_out_notify,
            task_notify,
        }
    }
    //发送模板消息
    #[allow(clippy::too_many_arguments)]
    pub async fn send<'t>(
        &self,
        app_id: Option<u64>,
        mail: &[&'t str],
        tpl_key: &str,
        tpl_var: &str,
        send_time: Option<u64>,
        user_id: Option<u64>,
        reply_mail: Option<&str>,
        max_try_num: Option<u8>,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<(u64, Vec<(u64, &'t str, Result<bool, FluentMessage>)>)> {
        let tmp = mail
            .iter()
            .collect::<HashSet<_>>()
            .into_iter()
            .copied()
            .collect::<Vec<_>>();
        let nt = now_time().unwrap_or_default();
        let sendtime = send_time.unwrap_or(nt);
        let sendtime = if sendtime < nt { nt } else { sendtime };
        let max_try_num = max_try_num.unwrap_or(0);
        self.mail_record
            .send_check(app_id, tpl_key, &tmp, sendtime)
            .await?;
        let res = self
            .mail_record
            .add(
                &tmp,
                app_id.unwrap_or_default(),
                tpl_key,
                tpl_var,
                sendtime,
                reply_mail,
                user_id,
                max_try_num,
                env_data,
            )
            .await?;

        let mut wait = None;
        if max_try_num == 0 && mail.len() == 1 {
            if let Some((snid, _)) = res.1.first() {
                wait = Some(self.send_wait.message_wait(res.0, *snid).await);
            }
        };

        if let Err(err) = self.task_timeout_notify.notify_at_time(sendtime).await {
            warn!(
                "mail is add [{}] ,but notify fail :{}",
                res.0,
                err.to_fluent_message().default_format()
            )
        }

        let mut tmp = vec![];
        for e in mail {
            let msg_id = res
                .1
                .iter()
                .find(|t| t.1 == *e)
                .map(|e| e.0)
                .unwrap_or_default();
            let item_res = if let Some(t) = wait.take() {
                self.send_wait
                    .wait_timeout(t)
                    .await
                    .map(|e| e.map_err(|c| fluent_message!("mail-send-fail", c)))
                    .unwrap_or_else(|e| {
                        Err(fluent_message!(
                            "mail-send-wait-fail",
                            e.to_fluent_message()
                        ))
                    })
            } else {
                Ok(true)
            };
            tmp.push((msg_id, *e, item_res));
        }
        Ok((res.0, tmp))
    }
    //通过ID取消发送
    pub async fn cancal_from_message(
        &self,
        body: &SenderMailBodyModel,
        msg_data: &[&SenderMailMessageModel],
        user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<Vec<(u64, bool, Option<SenderError>)>> {
        //消息id,是否在任务中，错误消息

        let mut out = Vec::with_capacity(msg_data.len());
        let mut cancel_data = vec![];
        for tmp in msg_data {
            if SenderMailMessageStatus::Init.eq(tmp.status) {
                cancel_data.push(tmp);
                continue;
            }
            if SenderMailMessageStatus::IsCancel.eq(tmp.status) {
                out.push((tmp.snid, false, None));
                continue;
            }
            out.push((
                tmp.snid,
                false,
                Some(SenderError::System(fluent_message!(
                    "mail-send-cancel-status-bad"
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
                self.mail_record
                    .cancel_form_message(body, msg, user_id, env_data)
                    .await
                    .err()
            } else {
                Some(SenderError::System(
                    fluent_message!("mail-send-cancel-is-ing",
                        {
                            "to_mail":&msg.to_mail,
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
        debug!("check mail task is run :{}", check_message_data.len());
        let mut tdata = self.task.task_data().await?;
        debug!("task data len :{}", tdata.len());
        let mut out = Vec::with_capacity(check_message_data.len());
        for (mid, data) in check_message_data {
            out.push((data, tdata.remove(mid)));
        }
        Ok(out)
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
            //查找指定body下的msg
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
    //发送等待回调处理监听
    pub async fn task_wait(&self) {
        self.send_wait.listen().await;
    }
    //指定发送时间到期监听处理
    pub async fn task_sendtime_notify(&self, channel_buffer: Option<usize>) {
        let task_send_time = Arc::new(MailTaskSendTimeNotify::new(
            format!("{}-last-run-time", MAILER_REDIS_PREFIX),
            self.db.clone(),
            self.redis.clone(),
            self.task_notify.clone(),
        ));
        TimeOutTask::<MailTaskSendTimeNotify>::new(
            self.app_core.clone(),
            self.time_out_notify.clone(),
            task_send_time.clone(),
            task_send_time,
        )
        .listen(channel_buffer)
        .await;
    }
    //后台发送任务，内部循环不退出
    pub async fn task_sender(
        &self,
        se: Vec<Box<dyn SenderTaskExecutor<u64, MailTaskItem, MailTaskData>>>,
    ) -> SenderResult<()> {
        let acquisition = Arc::new(MailTaskAcquisition::new(
            self.db.clone(),
            self.send_wait.clone(),
            self.message_logs.clone(),
            self.message_reader.clone(),
        ));
        self.task
            .dispatch(
                self.app_core.clone(),
                acquisition.as_ref(),
                Arc::new(MailerTask::new(
                    acquisition.to_owned(),
                    self.tpl_config.clone(),
                    se,
                )?),
            )
            .await;
        Ok(())
    }
}
