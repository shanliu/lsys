use crate::dao::{WebApp, WebResult};
use lsys_core::db::OffsetPageParam;
use lsys_setting::dao::{
    MultipleSettingData, SettingDecode, SettingEncode, SettingJson, SettingKey, SettingResult,
};
use serde::{Deserialize, Serialize};

/// setting.multiple 用于保存 WebApp 外部扩展能力的 key
///
/// - `SettingKey`: 固定为 `web-exter-feature`
/// - `SettingModel.name`: feature key（如 "sms" / "mail" / "custom_x"）
/// - `SettingModel.setting_data`: JSON（title）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WebExterFeatureSetting {
    pub title: String,
}

impl SettingKey for WebExterFeatureSetting {
    fn key<'t>() -> &'t str {
        "web-exter-feature"
    }
}
impl SettingDecode for WebExterFeatureSetting {
    fn decode(data: &str) -> SettingResult<Self> {
        SettingJson::decode(data)
    }
}
impl SettingEncode for WebExterFeatureSetting {
    fn encode(&self) -> String {
        SettingJson::encode(self)
    }
}
impl SettingJson<'_> for WebExterFeatureSetting {}

#[derive(Debug, Clone)]
pub struct WebExterFeatureSettingItem {
    pub id: u64,
    pub key: String,
    pub data: WebExterFeatureSetting,
}

impl WebApp {
    pub async fn exter_feature_add(
        &self,
        feature_key: &str,
        data: &WebExterFeatureSetting,
        change_user_id: u64,
        env_data: Option<&lsys_core::RequestEnv>,
    ) -> WebResult<u64> {
        let id = self
            .web_setting
            .setting_dao
            .multiple
            .add::<WebExterFeatureSetting>(
                None,
                &MultipleSettingData {
                    name: feature_key,
                    data,
                },
                change_user_id,
                None,
                env_data,
            )
            .await?;
        Ok(id)
    }

    pub async fn exter_feature_edit(
        &self,
        id: u64,
        feature_key: &str,
        data: &WebExterFeatureSetting,
        change_user_id: u64,
        env_data: Option<&lsys_core::RequestEnv>,
    ) -> WebResult<u64> {
        let rows = self
            .web_setting
            .setting_dao
            .multiple
            .edit::<WebExterFeatureSetting>(
                None,
                id,
                &MultipleSettingData {
                    name: feature_key,
                    data,
                },
                change_user_id,
                None,
                env_data,
            )
            .await?;
        Ok(rows)
    }

    pub async fn exter_feature_del(
        &self,
        id: u64,
        change_user_id: u64,
        env_data: Option<&lsys_core::RequestEnv>,
    ) -> WebResult<u64> {
        let rows = self
            .web_setting
            .setting_dao
            .multiple
            .del::<WebExterFeatureSetting>(None, id, change_user_id, None, env_data)
            .await?;
        Ok(rows)
    }

    /// 仅返回 Setting.multiple 中保存的扩展能力（不包含代码常量）。
    pub async fn exter_feature_list(
        &self,
        page: &OffsetPageParam,
    ) -> WebResult<Vec<WebExterFeatureSettingItem>> {
        let list = self
            .web_setting
            .setting_dao
            .multiple
            .list_data::<WebExterFeatureSetting>(None, None, page)
            .await?;

        let mut out = Vec::with_capacity(list.len());
        for item in list {
            out.push(WebExterFeatureSettingItem {
                id: item.model().id,
                key: item.model().name.clone(),
                data: std::ops::Deref::deref(&item).clone(),
            });
        }
        Ok(out)
    }
}
