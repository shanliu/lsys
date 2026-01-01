//站点配置
use lsys_access::dao::SessionBody;
use lsys_core::RequestEnv;
use lsys_user::dao::AccountPasswordConfig;
use serde::Deserialize;

use lsys_setting::dao::{
    SettingDecode, SettingEncode, SettingJson, SettingKey, SettingResult, SingleSettingData,
};
use serde::Serialize;

use crate::common::JsonResult;

use super::WebSetting;

#[derive(Deserialize, Serialize, Default)]
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

pub struct SiteConfigData<'t> {
    pub site_tips: &'t str,
    pub password_timeout: u64,
    pub disable_old_password: bool,
}
impl WebSetting {
    pub async fn save_site_setting_data(
        &self,
        session_body: &SessionBody,
        param: &SiteConfigData<'_>,
        env_data: Option<&RequestEnv>,
    ) -> JsonResult<()> {
        let mut transaction = self.db.begin().await?;
        if let Err(e) = self
            .setting_dao
            .single
            .save::<AccountPasswordConfig>(
                None,
                &SingleSettingData {
                    name: AccountPasswordConfig::key(),
                    data: &AccountPasswordConfig {
                        timeout: param.password_timeout,
                        disable_old_password: param.disable_old_password,
                    },
                },
                session_body.user_id(),
                Some(&mut transaction),
                env_data,
            )
            .await
        {
            transaction.rollback().await?;
            return Err(e.into());
        };
        if let Err(e) = self
            .setting_dao
            .single
            .save::<SiteConfig>(
                None,
                &SingleSettingData {
                    name: SiteConfig::key(),
                    data: &SiteConfig {
                        site_tips: param.site_tips.to_string(),
                    },
                },
                session_body.user_id(),
                Some(&mut transaction),
                env_data,
            )
            .await
        {
            transaction.rollback().await?;
            return Err(e.into());
        };
        transaction.commit().await?;
        Ok(())
    }
}
