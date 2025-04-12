use crate::module::WeChatConfig;
use lsys_web::{
    common::{JsonResponse, JsonResult, UserAuthQueryDao},
    handler::api::system::setting::{setting_get, setting_set},
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
pub async fn wechat_set_config(
    param: WechatSetConfigParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    setting_set::<WechatSetConfigParam, WeChatConfig>(param, req_dao).await
}

pub async fn wechat_get_config(req_dao: &UserAuthQueryDao) -> JsonResult<JsonResponse> {
    setting_get::<WeChatConfig>(req_dao).await
}
