use serde::Deserialize;

use lsys_setting::dao::{SettingDecode, SettingEncode, SettingJson, SettingKey, SettingResult};
use serde::Serialize;

#[derive(Deserialize, Serialize, Clone, Default)]
pub struct SiteConfig {
    pub site_tips: String,
}

impl SettingKey for SiteConfig {
    fn key<'t>() -> &'t str {
        "site-config"
    }
}
impl SettingDecode for SiteConfig {
    fn decode(data: &str) -> SettingResult<Self> {
        SettingJson::decode(data)
    }
}
impl SettingEncode for SiteConfig {
    fn encode(&self) -> String {
        SettingJson::encode(self)
    }
}
impl SettingJson<'_> for SiteConfig {}
