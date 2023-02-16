use std::sync::Arc;

use config::ConfigError;
use lettre::message::{header, Mailbox, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::AsyncTransport;
use lettre::{AsyncSmtpTransport, Message, Tokio1Executor};
use lsys_core::{AppCore, FluentMessage};
use lsys_user::dao::account::{check_email, UserAccountError};
use tera::{Context, Tera};

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

impl From<WebAppMailerError> for UserAccountError {
    fn from(err: WebAppMailerError) -> Self {
        UserAccountError::System(err.to_string())
    }
}
pub struct WebAppMailer {
    app_core: Arc<AppCore>,
    tera: Arc<Tera>,
    fluent: Arc<FluentMessage>,
}

macro_rules! err_result {
    ($msg:expr) => {
        WebAppMailerError::System($msg)
    };
}

impl WebAppMailer {
    pub fn new(app_core: Arc<AppCore>, tera: Arc<Tera>, fluent: Arc<FluentMessage>) -> Self {
        Self {
            app_core,
            tera,
            fluent,
        }
    }
    async fn send(&self, email: Message) -> Result<(), WebAppMailerError> {
        let host = self.app_core.config.get_string("smtp_host")?;
        let name = self
            .app_core
            .config
            .get_string("smtp_user")
            .unwrap_or_else(|_| String::from(""));
        let pass = self.app_core.config.get_string("smtp_password")?;

        let mut mailer_builder = AsyncSmtpTransport::<Tokio1Executor>::relay(host.as_str())
            .map_err(|e| err_result!(e.to_string() + "[smtp_relay]"))?;
        if !name.is_empty() || !pass.is_empty() {
            let creds = Credentials::new(name, pass);
            mailer_builder = mailer_builder.credentials(creds)
        }
        let mailer: AsyncSmtpTransport<Tokio1Executor> = mailer_builder.build();
        mailer
            .send(email)
            .await
            .map_err(|e| err_result!(e.to_string() + "[smtp_send]"))?;
        Ok(())
    }
    pub async fn send_valid_code(
        &self,
        to: &str,
        code: &str,
        ttl: &usize,
    ) -> Result<(), WebAppMailerError> {
        let mut context = Context::new();
        context.insert("code", code);
        context.insert("ttl", ttl);
        self.tpl_send(to, "valid_code", &context).await
    }
    async fn tpl_send(
        &self,
        to: &str,
        tpl_type: &str,
        context: &Context,
    ) -> Result<(), WebAppMailerError> {
        check_email(&self.fluent, to).map_err(|e| WebAppMailerError::System(e.to_string()))?;
        let tomail = to
            .parse::<Mailbox>()
            .map_err(|e| err_result!(format!("to mail error:{}", e)))?;
        let notifys = self.app_core.config.get_table("smtp_notify")?;
        let notify = notifys
            .get(tpl_type)
            .ok_or_else(|| err_result!(format!("not find{} tpl [smtp_notify]", tpl_type)))?
            .to_owned()
            .into_table()?;
        let subject = if let Some(sub) = notify.get("subject") {
            if let Ok(subt) = sub.to_owned().into_string() {
                subt
            } else {
                format!("notify {}", tpl_type)
            }
        } else {
            format!("notify {}", tpl_type)
        };
        let account = if let Some(sub) = notify.get("send_mail") {
            if let Ok(subt) = sub.to_owned().into_string() {
                subt.parse::<Mailbox>()
                    .map_err(|e| err_result!(e.to_string() + " on send_mail"))?
            } else {
                return Err(err_result!(format!(
                    "send_mail a string on {} - send_mail",
                    tpl_type
                )));
            }
        } else {
            return Err(err_result!(format!(
                "send_mail not find on {} - send_mail",
                tpl_type
            )));
        };

        let reply = if let Some(sub) = notify.get("reply_mail") {
            if let Ok(subt) = sub.to_owned().into_string() {
                subt.parse::<Mailbox>().unwrap_or_else(|_| account.clone())
            } else {
                account.clone()
            }
        } else {
            account.clone()
        };

        let template_name = notify
            .get("tpl")
            .ok_or_else(|| err_result!(format!("not find tpl on {}", tpl_type)))?
            .to_owned()
            .into_string()
            .map_err(|e| err_result!(e.to_string() + "[mail_notify_tpl]"))?;
        let body = self.tera.render(&template_name, context)?;
        let email = Message::builder()
            .from(account)
            .reply_to(reply)
            .to(tomail)
            .subject(&subject)
            .multipart(
                MultiPart::alternative() // This is composed of two parts.
                    .singlepart(
                        SinglePart::builder()
                            .header(header::ContentType::TEXT_HTML)
                            .body(body),
                    ),
            )
            .map_err(|e| err_result!(e.to_string() + "[build_mail]"))?;
        self.send(email).await
    }
}
