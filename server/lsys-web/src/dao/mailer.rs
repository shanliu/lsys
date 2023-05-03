use config::ConfigError;
use lsys_app::model::AppsModel;
use lsys_core::{AppCore, FluentMessage, RequestEnv};
use lsys_logger::dao::ChangeLogger;
use lsys_sender::{
    dao::{
        MailSender, MailTaskAcquisitionRecord, MailTaskRecord, SenderError, SmtpSender,
        SmtpSenderTask,
    },
    model::SenderMailMessageModel,
};
use lsys_setting::dao::MultipleSetting;
use lsys_user::dao::account::{check_email, UserAccountError};
use sqlx::{MySql, Pool};
use std::sync::Arc;
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
    app_core: Arc<AppCore>,
    fluent: Arc<FluentMessage>,
    pub smtp_sender: SmtpSender,
    mailer: Arc<MailSender<MailTaskAcquisitionRecord, ()>>,
    mail_record: Arc<MailTaskRecord>,
    setting: Arc<MultipleSetting>,
    db: Pool<MySql>,
    logger: Arc<ChangeLogger>,
}

impl WebAppMailer {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        app_core: Arc<AppCore>,
        fluent: Arc<FluentMessage>,
        redis: deadpool_redis::Pool,
        db: Pool<MySql>,
        setting: Arc<MultipleSetting>,
        logger: Arc<ChangeLogger>,
        task_size: Option<usize>,
        task_timeout: usize,
        is_check: bool,
    ) -> Self {
        let mail_record = Arc::new(MailTaskRecord::new(
            db.clone(),
            app_core.clone(),
            fluent.clone(),
            logger.clone(),
        ));
        let acquisition = MailTaskAcquisitionRecord::new(mail_record.clone());
        let mailer = Arc::new(MailSender::new(
            redis,
            task_size,
            task_timeout,
            is_check,
            acquisition,
        ));
        let smtp_sender = SmtpSender::new(db.clone(), setting.clone(), logger.clone());
        Self {
            app_core,
            fluent,
            smtp_sender,
            mailer,
            mail_record,
            setting,
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
    ) -> Result<(), WebAppMailerError> {
        let mut context = Context::new();
        context.insert("code", code);
        context.insert("ttl", ttl);
        self.send("valid_code", to, &context.into_json().to_string(), env_data)
            .await
    }
    // 发送接口
    async fn send(
        &self,
        tpl_type: &str,
        to: &str,
        body: &str,
        env_data: Option<&RequestEnv>,
    ) -> Result<(), WebAppMailerError> {
        check_email(&self.fluent, to).map_err(|e| WebAppMailerError::System(e.to_string()))?;
        self.mailer
            .send(
                None,
                &[to],
                tpl_type,
                body,
                &None,
                &None,
                &None,
                &None,
                env_data,
            )
            .await
            .map_err(|e| WebAppMailerError::System(e.to_string()))
            .map(|_| ())
    }
    // 后台任务
    pub async fn task(&self) -> Result<(), WebAppMailerError> {
        Ok(self
            .mailer
            .task(
                self.app_core.clone(),
                self.mail_record.clone(),
                vec![Box::new(SmtpSenderTask::new(
                    SmtpSender::new(self.db.clone(), self.setting.clone(), self.logger.clone()),
                    self.db.clone(),
                    self.fluent.clone(),
                    self.logger.clone(),
                ))],
            )
            .await?)
    }
    // 后台任务
    pub fn mail_record(&self) -> &MailTaskRecord {
        &self.mail_record
    }
    // 取消发送接口
    pub async fn send_cancel(
        &self,
        message: &SenderMailMessageModel,
        user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> Result<(), WebAppMailerError> {
        self.mailer
            .cancal_from_message(message, &user_id, env_data)
            .await
            .map_err(|e| WebAppMailerError::System(e.to_string()))
            .map(|_| ())
    }
    // 发送接口
    #[allow(clippy::too_many_arguments)]
    pub async fn app_send(
        &self,
        app: &AppsModel,
        tpl_type: &str,
        to: &[String],
        body: &str,
        send_time: Option<u64>,
        reply: &Option<String>,
        cancel_key: &Option<String>,
        env_data: Option<&RequestEnv>,
    ) -> Result<(), WebAppMailerError> {
        for tmp in to.iter() {
            check_email(&self.fluent, tmp).map_err(|e| WebAppMailerError::System(e.to_string()))?;
        }
        if let Some(ref cr) = reply {
            if !cr.is_empty() {
                check_email(&self.fluent, cr)
                    .map_err(|e| WebAppMailerError::System(e.to_string()))?;
            }
        }
        self.mailer
            .send(
                Some(app.id),
                &to.iter().map(|e| e.as_str()).collect::<Vec<_>>(),
                tpl_type,
                body,
                &send_time,
                &Some(app.user_id),
                reply,
                cancel_key,
                env_data,
            )
            .await
            .map_err(|e| WebAppMailerError::System(e.to_string()))
            .map(|_| ())
    }
    // APP 取消发送
    pub async fn app_send_cancel(
        &self,
        app: &AppsModel,
        cancel_key: &str,
        env_data: Option<&RequestEnv>,
    ) -> Result<(), WebAppMailerError> {
        self.mailer
            .cancal_from_key(cancel_key, &app.user_id, env_data)
            .await
            .map_err(|e| WebAppMailerError::System(e.to_string()))
            .map(|_| ())
    }
}
