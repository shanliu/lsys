use crate::module::WeChatConfig;
use lsys_user::dao::auth::{SessionData, SessionTokenData, UserSession};
use lsys_web::{
    dao::RequestAuthDao,
    handler::api::setting::{setting_get, setting_set},
    JsonData, JsonResult,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct WechatSetConfigParam {
    pub app_id: String,
    pub app_secret: String,
}
impl From<WechatSetConfigParam> for WeChatConfig {
    fn from(value: WechatSetConfigParam) -> Self {
        WeChatConfig {
            app_id: value.app_id,
            app_secret: value.app_secret,
        }
    }
}
pub async fn wechat_set_config<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: WechatSetConfigParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    setting_set::<WechatSetConfigParam, WeChatConfig, T, D, S>(param, req_dao).await
}

pub async fn wechat_get_config<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    setting_get::<WeChatConfig, T, D, S>(req_dao).await
}
