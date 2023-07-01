use std::{collections::HashSet, sync::Arc};

use lsys_core::{now_time, AppCore, FluentMessage, RequestEnv};

use lsys_logger::dao::ChangeLogger;
use lsys_setting::dao::Setting;
use sqlx::{MySql, Pool};
use tracing::warn;

use crate::{
    dao::{
        MessageCancel, MessageLogs, MessageReader, SenderConfig, SenderResult, SenderTaskExecutor,
        SenderTplConfig,
    },
    model::{SenderMailMessageModel, SenderType},
};

use super::{super::TaskDispatch, MailRecord, MailTaskAcquisition, MailTaskItem, MailerTask};

const MAILER_REDIS_PREFIX: &str = "sender-mail-";

pub struct MailSender {
    pub tpl_config: Arc<SenderTplConfig>,
    pub mail_record: Arc<MailRecord>,
    redis: deadpool_redis::Pool,
    db: Pool<sqlx::MySql>,
    app_core: Arc<AppCore>,
    message_logs: Arc<MessageLogs>,
    message_reader: Arc<MessageReader<SenderMailMessageModel>>,
    task: TaskDispatch<u64, MailTaskItem>,
}

impl MailSender {
    //发送
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        app_core: Arc<AppCore>,
        redis: deadpool_redis::Pool,
        db: Pool<MySql>,
        fluent: Arc<FluentMessage>,
        setting: Arc<Setting>,
        logger: Arc<ChangeLogger>,
        task_size: Option<usize>,
        task_timeout: usize,
        is_check: bool,
    ) -> Self {
        let config: Arc<SenderConfig> = Arc::new(SenderConfig::new(
            db.clone(),
            logger.clone(),
            SenderType::Mailer,
        ));
        let tpl_config = Arc::new(SenderTplConfig::new(
            db.clone(),
            setting,
            logger,
            SenderType::Mailer,
        ));
        let cancel = Arc::new(MessageCancel::new(db.clone(), SenderType::Mailer));
        let message_logs = Arc::new(MessageLogs::new(db.clone(), SenderType::Mailer));
        let message_reader = Arc::new(MessageReader::new(db.clone(), app_core.clone(), fluent));
        let mail_record = Arc::new(MailRecord::new(
            db.clone(),
            config,
            cancel,
            message_logs.clone(),
            message_reader.clone(),
        ));

        let task = TaskDispatch::new(
            format!("{}-notify", MAILER_REDIS_PREFIX),
            format!("{}-read-lock", MAILER_REDIS_PREFIX),
            format!("{}-run-task", MAILER_REDIS_PREFIX),
            format!("{}-run-num", MAILER_REDIS_PREFIX),
            task_size,
            task_timeout,
            is_check,
            task_timeout,
        );
        Self {
            tpl_config,
            redis,
            mail_record,
            app_core,
            db,
            message_logs,
            message_reader,
            task,
        }
    }
    //发送模板消息
    #[allow(clippy::too_many_arguments)]
    pub async fn send(
        &self,
        app_id: Option<u64>,
        mail: &[&str],
        tpl_id: &str,
        tpl_var: &str,
        send_time: &Option<u64>,
        user_id: &Option<u64>,
        reply_mail: &Option<String>,
        cancel_key: &Option<String>,
        max_try_num: &Option<u16>,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        let mail = mail
            .iter()
            .collect::<HashSet<_>>()
            .into_iter()
            .map(|e| e.to_string())
            .collect::<Vec<String>>();

        let nt = now_time().unwrap_or_default();
        let sendtime = send_time.unwrap_or(nt);
        let sendtime = if sendtime < nt { nt } else { sendtime };
        self.mail_record
            .send_check(app_id, tpl_id, &mail, sendtime)
            .await?;
        let id = self
            .mail_record
            .add(
                &mail,
                &app_id.unwrap_or_default(),
                tpl_id,
                tpl_var,
                &sendtime,
                reply_mail,
                user_id,
                cancel_key,
                max_try_num,
                env_data,
            )
            .await?;
        if send_time
            .map(|e| e - 1 <= now_time().unwrap_or_default())
            .unwrap_or(true)
        {
            let mut redis = self.redis.get().await?;
            if let Err(err) = self.task.notify(&mut redis).await {
                warn!("mail is add [{}] ,but send fail :{}", id, err)
            }
        }

        Ok(id)
    }
    //通过ID取消发送
    pub async fn cancal_from_message(
        &self,
        msg: &SenderMailMessageModel,
        user_id: &u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        let mut redis = self.redis.get().await?;
        let tdata = self.task.task_data(&mut redis).await?;
        if tdata.get(&msg.id).is_none() {
            self.mail_record
                .cancel_form_message(msg, user_id, env_data)
                .await?;
        }
        Ok(1)
    }
    //通过KEY取消发送
    pub async fn cancal_from_key(
        &self,
        cancel_key: &str,
        user_id: &u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        let data = self.mail_record.cancel_data(cancel_key).await?;
        let mut redis = self.redis.get().await?;
        let mut succ = 0;
        for tmp in data {
            let tdata = self.task.task_data(&mut redis).await?;
            if tdata.get(&tmp.id).is_none() {
                self.mail_record
                    .cancel_form_key(&tmp, user_id, env_data)
                    .await?;
                succ += 1;
            }
        }
        Ok(succ)
    }
    //后台发送任务，内部循环不退出
    pub async fn task(
        &self,
        se: Vec<Box<dyn SenderTaskExecutor<u64, MailTaskItem>>>,
    ) -> SenderResult<()> {
        let acquisition = Arc::new(MailTaskAcquisition::new(
            self.db.clone(),
            self.message_logs.clone(),
            self.message_reader.clone(),
        ));
        self.task
            .dispatch(
                self.app_core.clone(),
                acquisition.as_ref(),
                MailerTask::new(acquisition.to_owned(), self.tpl_config.clone(), se)?,
            )
            .await;
        Ok(())
    }
}
