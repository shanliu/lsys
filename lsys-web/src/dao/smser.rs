use std::sync::Arc;

use config::ConfigError;
use lsys_core::FluentMessage;
use lsys_sender::dao::Smser;
use lsys_user::dao::account::{check_mobile, UserAccountError};
use tera::Context;

pub enum WebAppSmserError {
    Config(ConfigError),
    System(String),
    Tera(tera::Error),
}
impl ToString for WebAppSmserError {
    fn to_string(&self) -> String {
        match self {
            WebAppSmserError::Config(err) => {
                format!("config error:{}", err)
            }
            WebAppSmserError::System(err) => {
                format!("error:{}", err)
            }
            WebAppSmserError::Tera(err) => {
                format!("tpl error:{}", err)
            }
        }
    }
}
impl From<ConfigError> for WebAppSmserError {
    fn from(err: ConfigError) -> Self {
        WebAppSmserError::Config(err)
    }
}

impl From<tera::Error> for WebAppSmserError {
    fn from(err: tera::Error) -> Self {
        WebAppSmserError::Tera(err)
    }
}

impl From<WebAppSmserError> for UserAccountError {
    fn from(err: WebAppSmserError) -> Self {
        UserAccountError::System(err.to_string())
    }
}
pub struct WebAppSmser {
    smser: Arc<Smser>,
    fluent: Arc<FluentMessage>,
}

impl WebAppSmser {
    pub fn new(smser: Arc<Smser>, fluent: Arc<FluentMessage>) -> Self {
        Self { smser, fluent }
    }
    pub async fn send(
        &self,
        tpl_type: &str,
        area: &str,
        mobile: &str,
        body: &str,
    ) -> Result<(), WebAppSmserError> {
        check_mobile(&self.fluent, area, mobile)
            .map_err(|e| WebAppSmserError::System(e.to_string()))?;
        self.smser
            .send(area, mobile, tpl_type, body)
            .await
            .map_err(WebAppSmserError::System)
            .map(|_| ())
    }
    pub async fn send_valid_code(
        &self,
        area: &str,
        mobile: &str,
        code: &str,
        ttl: &usize,
    ) -> Result<(), WebAppSmserError> {
        let mut context = Context::new();
        context.insert("code", code);
        context.insert("time", &ttl);
        self.tpl_send(area, mobile, "valid_code", context).await
    }
    pub async fn tpl_send(
        &self,
        area: &str,
        mobile: &str,
        tpl_type: &str,
        context: Context,
    ) -> Result<(), WebAppSmserError> {
        // # [sms_notify.valid_code]
        // # tpl="sms/valid_code.txt"
        // # sms_tpl="valid_code"
        // let notifys = self.app_core.config.get_table("sms_notify")?;
        // let notify = notifys
        //     .get(tpl_type)
        //     .ok_or_else(|| {
        //         err_result!(format!("not find {} notify config [sms_notify]", tpl_type))
        //     })?
        //     .to_owned()
        //     .into_table()
        //     .map_err(|e| err_result!(e.to_string() + "[sms_notify_parse]"))?;
        // let template_name = notify
        //     .get("tpl")
        //     .ok_or_else(|| err_result!(format!("not find tpl on {}", tpl_type)))?
        //     .to_owned()
        //     .into_string()
        //     .map_err(|e| err_result!(e.to_string() + "[sms_notify_tpl]"))?;
        // let body = self.tera.render(&template_name, context)?;
        self.send(tpl_type, area, mobile, &context.into_json().to_string())
            .await
    }
}
