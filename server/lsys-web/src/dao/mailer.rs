use config::ConfigError;
use lsys_app::model::AppsModel;
use lsys_core::{AppCore, FluentMessage, RequestEnv};
use lsys_logger::dao::ChangeLogger;
use lsys_sender::{
    dao::{MailSender, SenderError, SenderSmtpConfig, SmtpSenderTask},
    model::{SenderMailBodyModel, SenderMailMessageModel},
};
use lsys_setting::dao::Setting;
use lsys_user::dao::account::{check_email, UserAccountError};
use serde_json::json;
use sqlx::{MySql, Pool};
use std::{collections::HashMap, sync::Arc};
use tera::Context;

pub enum WebAppMailerError {
    Config(ConfigError),
    System(String),
    Tera(tera::Error),
}

impl From<ConfigError> for WebAppMailerError {
    fn from(err: ConfigError) -> Self {
        WebAppMailerError::Config(err)
    }
}
impl ToString for WebAppMailerError {
    fn to_string(&self) -> String {
        match self {
            WebAppMailerError::Config(err) => {
                format!("config error:{}", err)
            }
            WebAppMailerError::System(err) => {
                format!("error:{}", err)
            }
            WebAppMailerError::Tera(err) => {
                format!("tpl error:{}", err)
            }
        }
    }
}
impl From<tera::Error> for WebAppMailerError {
    fn from(err: tera::Error) -> Self {
        WebAppMailerError::Tera(err)
    }
}
impl From<SenderError> for WebAppMailerError {
    fn from(err: SenderError) -> Self {
        WebAppMailerError::System(err.to_string())
    }
}

impl From<WebAppMailerError> for UserAccountError {
    fn from(err: WebAppMailerError) -> Self {
        UserAccountError::System(err.to_string())
    }
}

pub struct WebAppMailer {
    fluent: Arc<FluentMessage>,
    pub mailer: Arc<MailSender>,
    db: Pool<MySql>,
    logger: Arc<ChangeLogger>,
    pub smtp_sender: SenderSmtpConfig,
}

impl WebAppMailer {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        app_core: Arc<AppCore>,
        fluent: Arc<FluentMessage>,
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
            fluent.clone(),
            setting.clone(),
            logger.clone(),
            task_size,
            task_timeout,
            is_check,
        ));
        let smtp_sender = SenderSmtpConfig::new(
            fluent.clone(),
            setting.multiple.clone(),
            mailer.tpl_config.clone(),
        );
        Self {
            mailer,
            fluent,
            smtp_sender,
            db,
            logger,
        }
    }
    // pub fn tpl_config(&self) -> &SenderTplConfig {
    //     &self.mailer.tpl_config
    // }
    // pub fn mail_record(&self) -> &MailRecord {
    //     &self.mailer.mail_record
    // }
    // pub fn smtp_sender(&self) -> &SenderSmtpConfig {
    //     &self.smtp_sender
    // }
    pub async fn send_valid_code(
        &self,
        to: &str,
        code: &str,
        ttl: &usize,
        env_data: Option<&RequestEnv>,
    ) -> Result<(), WebAppMailerError> {
        let mut context = Context::new();
        context.insert("code", code);
        context.insert("ttl", ttl);
        self.send(
            "valid_code",
            to,
            &context.into_json().to_string(),
            &Some(1),
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
    ) -> Result<u64, WebAppMailerError> {
        check_email(&self.fluent, to).map_err(|e| WebAppMailerError::System(e.to_string()))?;
        let out = self
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
            .await
            .map_err(|e| WebAppMailerError::System(e.to_string()))?;
        Ok(out.1.first().map(|e| e.0).unwrap_or_default())
    }
    // 后台任务
    pub async fn task(&self) -> Result<(), WebAppMailerError> {
        let task = SmtpSenderTask::new(self.db.clone(), self.fluent.clone(), self.logger.clone());
        Ok(self.mailer.task(vec![Box::new(task)]).await?)
    }

    // 取消发送接口
    pub async fn send_cancel(
        &self,
        body: &SenderMailBodyModel,
        message: &[&SenderMailMessageModel],
        user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> Result<Vec<(u64, bool)>, WebAppMailerError> {
        self.mailer
            .cancal_from_message(body, message, &user_id, env_data)
            .await
            .map_err(|e| WebAppMailerError::System(e.to_string()))
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
    ) -> Result<Vec<(u64, &'t str)>, WebAppMailerError> {
        for tmp in to.iter() {
            check_email(&self.fluent, tmp).map_err(|e| WebAppMailerError::System(e.to_string()))?;
        }
        if let Some(ref cr) = reply {
            if !cr.is_empty() {
                check_email(&self.fluent, cr)
                    .map_err(|e| WebAppMailerError::System(e.to_string()))?;
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
            .map_err(|e| WebAppMailerError::System(e.to_string()))
            .map(|e| e.1)
    }
    // APP 取消发送
    pub async fn app_send_cancel(
        &self,
        app: &AppsModel,
        id_data: &[u64],
        env_data: Option<&RequestEnv>,
    ) -> Result<(), WebAppMailerError> {
        self.mailer
            .cancal_from_message_id_vec(id_data, &app.user_id, env_data)
            .await
            .map_err(|e| WebAppMailerError::System(e.to_string()))
            .map(|_| ())
    }
}
