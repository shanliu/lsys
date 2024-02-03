use lsys_app::model::AppsModel;
use lsys_core::{fluent_message, AppCore, RequestEnv};
use lsys_logger::dao::ChangeLogger;
use lsys_sender::{
    dao::{MailSender, SenderError, SenderSmtpConfig, SmtpSenderTask},
    model::{SenderMailBodyModel, SenderMailMessageModel},
};
use lsys_setting::dao::Setting;
use lsys_user::dao::account::check_email;
use serde_json::json;
use sqlx::{MySql, Pool};
use std::{collections::HashMap, sync::Arc};
use tera::Context;

pub struct WebAppMailer {
    pub mailer: Arc<MailSender>,
    db: Pool<MySql>,
    logger: Arc<ChangeLogger>,
    pub smtp_sender: SenderSmtpConfig,
}

impl WebAppMailer {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        app_core: Arc<AppCore>,
        redis: deadpool_redis::Pool,
        db: Pool<MySql>,
        setting: Arc<Setting>,
        logger: Arc<ChangeLogger>,
        task_size: Option<usize>,
        task_timeout: usize,
        is_check: bool,
    ) -> Self {
        let mailer = Arc::new(MailSender::new(
            app_core,
            redis,
            db.clone(),
            setting.clone(),
            logger.clone(),
            task_size,
            task_timeout,
            is_check,
            None,
        ));
        let smtp_sender =
            SenderSmtpConfig::new(setting.multiple.clone(), mailer.tpl_config.clone());
        Self {
            mailer,
            smtp_sender,
            db,
            logger,
        }
    }
    pub async fn send_valid_code(
        &self,
        to: &str,
        code: &str,
        ttl: &usize,
        env_data: Option<&RequestEnv>,
    ) -> Result<(), SenderError> {
        let mut context = Context::new();
        context.insert("code", code);
        context.insert("ttl", ttl);
        self.send(
            "valid_code",
            to,
            &context.into_json().to_string(),
            &Some(0),
            env_data,
        )
        .await
        .map(|_| ())
    }
    // 发送接口
    async fn send(
        &self,
        tpl_id: &str,
        to: &str,
        body: &str,
        max_try_num: &Option<u8>,
        env_data: Option<&RequestEnv>,
    ) -> Result<u64, SenderError> {
        check_email(to).map_err(|e| SenderError::System(fluent_message!("mail-send-check", e)))?;
        let mut out = self
            .mailer
            .send(
                None,
                &[to],
                tpl_id,
                body,
                &None,
                &None,
                &None,
                max_try_num,
                env_data,
            )
            .await?;

        Ok(match out.1.pop() {
            Some(tmp1) => {
                tmp1.2.map_err(SenderError::System)?;
                tmp1.0
            }
            None => {
                return Err(SenderError::System(fluent_message!(
                    "mail-send-check",
                    "unkown error"
                )))
            }
        })
    }
    // 后台任务
    pub async fn task_sender(&self) -> Result<(), SenderError> {
        let task = SmtpSenderTask::new(self.db.clone(), self.logger.clone());
        self.mailer.task_sender(vec![Box::new(task)]).await
    }
    // 后台任务
    pub async fn task_wait(&self) {
        self.mailer.task_wait().await
    }
    // 取消发送接口
    pub async fn send_cancel(
        &self,
        body: &SenderMailBodyModel,
        message: &[&SenderMailMessageModel],
        user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> Result<Vec<(u64, bool, Option<SenderError>)>, SenderError> {
        self.mailer
            .cancal_from_message(body, message, &user_id, env_data)
            .await
    }
    // 发送接口
    #[allow(clippy::too_many_arguments)]
    pub async fn app_send<'t>(
        &self,
        app: &AppsModel,
        tpl_id: &str,
        to: &[&'t str],
        body: &HashMap<String, String>,
        send_time: &Option<u64>,
        reply: &Option<String>,
        max_try_num: &Option<u8>,
        env_data: Option<&RequestEnv>,
    ) -> Result<Vec<(u64, &'t str)>, SenderError> {
        for tmp in to.iter() {
            check_email(tmp)
                .map_err(|e| SenderError::System(fluent_message!("mail-send-check", e)))?;
        }
        if let Some(ref cr) = reply {
            if !cr.is_empty() {
                check_email(cr).map_err(|e| {
                    SenderError::System(fluent_message!("mail-send-reply-check",{
                        "msg":e,
                        "reply":cr
                    }))
                })?;
            }
        }
        let tos = to.to_vec();
        self.mailer
            .send(
                Some(app.id),
                &tos,
                tpl_id,
                &json!(body).to_string(),
                send_time,
                &Some(app.user_id),
                reply,
                max_try_num,
                env_data,
            )
            .await
            .map(|e| e.1.into_iter().map(|e| (e.0, e.1)).collect::<Vec<_>>())
    }
    // APP 取消发送
    pub async fn app_send_cancel(
        &self,
        app: &AppsModel,
        snid_data: &[u64],
        env_data: Option<&RequestEnv>,
    ) -> Result<Vec<(u64, bool, Option<SenderError>)>, SenderError> {
        self.mailer
            .cancal_from_message_snid_vec(snid_data, &app.user_id, env_data)
            .await
    }
}
