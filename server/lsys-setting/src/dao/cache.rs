//验证结果缓存
use async_trait::async_trait;
use lsys_core::cache::{LocalCache, LocalCacheClearItem};
use std::sync::Arc;

use crate::model::SettingModel;

use super::SettingDao;

//RBAC 授权缓存
pub enum SettingLocalCacheClear {
    MultipleSetting(Arc<LocalCache<String, Vec<SettingModel>>>),
    SingleSetting(Arc<LocalCache<String, SettingModel>>),
}

impl SettingLocalCacheClear {
    pub fn new_clears(setting: &SettingDao) -> Vec<Self> {
        vec![
            SettingLocalCacheClear::MultipleSetting(setting.multiple.cache.clone()),
            SettingLocalCacheClear::SingleSetting(setting.single.cache.clone()),
        ]
    }
}

#[async_trait]
impl LocalCacheClearItem for SettingLocalCacheClear {
    fn cache_name(&self) -> &str {
        match self {
            SettingLocalCacheClear::MultipleSetting(cache) => cache.config().cache_name,
            SettingLocalCacheClear::SingleSetting(cache) => cache.config().cache_name,
        }
    }
    async fn clear_from_message(&self, msg: &str) -> Result<(), String> {
        match self {
            SettingLocalCacheClear::MultipleSetting(cache) => cache.del(&msg.to_string()).await,
            SettingLocalCacheClear::SingleSetting(cache) => cache.del(&msg.to_string()).await,
        };
        Ok(())
    }
}
