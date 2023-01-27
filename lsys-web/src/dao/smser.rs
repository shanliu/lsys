use std::sync::Arc;

use config::ConfigError;
use lsys_core::{AppCore, FluentMessage};
use lsys_user::dao::account::{check_mobile, UserAccountError};
use tera::{Context, Tera};
use tracing::debug;

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

macro_rules! err_result {
    ($msg:expr) => {
        WebAppSmserError::System($msg)
    };
}
impl From<WebAppSmserError> for UserAccountError {
    fn from(err: WebAppSmserError) -> Self {
        UserAccountError::System(err.to_string())
    }
}
pub struct WebAppSmser {
    app_core: Arc<AppCore>,
    // tera: Arc<Tera>,
    fluent: Arc<FluentMessage>,
}

impl WebAppSmser {
    pub fn new(app_core: Arc<AppCore>, _: Arc<Tera>, fluent: Arc<FluentMessage>) -> Self {
        Self {
            app_core,
            // tera,
            fluent,
        }
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
        let aliconfig = self.app_core.config.get_table("alisms")?;
        let sms_config = aliconfig
            .get(tpl_type)
            .ok_or_else(|| {
                err_result!(format!("not find {} notify config [ali_config]", tpl_type))
            })?
            .to_owned()
            .into_table()
            .map_err(|e| err_result!(e.to_string() + "[ali_config]"))?;
        let tpl = sms_config
            .get("sms_tpl")
            .ok_or_else(|| err_result!(format!("not find {} notify tpl [ali_tpls]", tpl_type)))?
            .to_owned()
            .into_string()
            .map_err(|e| err_result!(e.to_string() + "[ali_tpls]"))?;
        let key = sms_config
            .get("access_key_id")
            .ok_or_else(|| err_result!(format!("not find {} notify tpl [ali_key]", tpl_type)))?
            .to_owned()
            .into_string()
            .map_err(|e| err_result!(e.to_string() + "[ali_key]"))?;
        let secret = sms_config
            .get("access_key_secret")
            .ok_or_else(|| err_result!(format!("not find {} notify tpl [ali_secret]", tpl_type)))?
            .to_owned()
            .into_string()
            .map_err(|e| err_result!(e.to_string() + "[ali_secret]"))?;
        let name = sms_config
            .get("sign_name")
            .ok_or_else(|| {
                err_result!(format!(
                    "not find {} notify tpl [ali_sms_sign_name]",
                    tpl_type
                ))
            })?
            .to_owned()
            .into_string()
            .map_err(|e| err_result!(e.to_string() + "[ali_sms_sign_name]"))?;
        use sms::aliyun::Aliyun;
        let aliyun = Aliyun::new(key.as_str(), secret.as_str());
        let resp = aliyun
            .send_sms(mobile, name.as_str(), tpl.as_str(), body)
            .await
            .map_err(|e| WebAppSmserError::System(e.to_string()))?;
        debug!("aliyun sms resp :{:?}", resp);
        if resp.get("Code").map(|e| e == "OK").unwrap_or(false) {
            return Ok(());
        }
        Err(err_result!(format!(
            "aliyun error:{:?} ",
            resp.get("Message")
        )))
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
