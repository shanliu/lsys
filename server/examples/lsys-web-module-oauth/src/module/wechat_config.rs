use serde::Deserialize;

use lsys_setting::dao::{SettingDecode, SettingEncode, SettingJson, SettingKey, SettingResult};
use serde::Serialize;

//config

#[derive(Deserialize, Serialize, Clone, Default)]
pub struct WeChatConfig {
    pub app_id: String,
    pub app_secret: String,
}

impl SettingKey for WeChatConfig {
    fn key<'t>() -> &'t str {
        "oauth-wechat"
    }
}

impl SettingDecode for WeChatConfig {
    fn decode(data: &str) -> SettingResult<Self> {
        SettingJson::decode(data)
    }
}
impl SettingEncode for WeChatConfig {
    fn encode(&self) -> String {
        SettingJson::encode(self)
    }
}
impl SettingJson<'_> for WeChatConfig {}
