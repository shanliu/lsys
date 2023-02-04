use std::sync::Arc;

use lsys_core::{AppCore, FluentMessage};
use lsys_user::dao::account::{check_mobile, UserAccountError};
use sms::aliyun::Aliyun;

use crate::dao::{SmserError, SmserSender};

macro_rules! err_result {
    ($msg:expr) => {
        SmserError::System($msg)
    };
}
impl From<SmserError> for UserAccountError {
    fn from(err: SmserError) -> Self {
        UserAccountError::System(err.to_string())
    }
}
pub struct ALiYunSmser {
    app_core: Arc<AppCore>,
    fluent: Arc<FluentMessage>,
}

impl SmserSender for ALiYunSmser {
    fn send(
        &self,
        tpl_type: &str,
        area: &str,
        mobile: &str,
        _body: &str,
    ) -> Result<(), SmserError> {
        check_mobile(&self.fluent, area, mobile).map_err(|e| SmserError::System(e.to_string()))?;
        let aliconfig = self.app_core.config.get_table("alisms")?;
        let sms_config = aliconfig
            .get(tpl_type)
            .ok_or_else(|| {
                err_result!(format!("not find {} notify config [ali_config]", tpl_type))
            })?
            .to_owned()
            .into_table()
            .map_err(|e| err_result!(e.to_string() + "[ali_config]"))?;
        let _tpl = sms_config
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
        let _name = sms_config
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

        let _aliyun = Aliyun::new(key.as_str(), secret.as_str());
        // let resp = aliyun
        //     .send_sms(mobile, name.as_str(), tpl.as_str(), body)
        // .await
        // .map_err(|e| SmserError::System(e.to_string()))?;
        // debug!("aliyun sms resp :{:?}", resp);
        // if resp.get("Code").map(|e| e == "OK").unwrap_or(false) {
        //     return Ok(());
        // }
        // Err(err_result!(format!(
        //     "aliyun error:{:?} ",
        //     resp.get("Message")
        // )))
        todo!()
    }
}
