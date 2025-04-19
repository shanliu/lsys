use std::sync::Arc;

use async_trait::async_trait;
use lsys_core::{
    cache::{LocalCache, LocalCacheClearItem},
    IntoFluentMessage,
};

use crate::model::{AppModel, AppOAuthClientModel};

use super::{AppDao, AppOAuthServerScopeData, AppSecretCacheKey, AppSecretRecrod};

pub enum AppLocalCacheClear {
    AppId(Arc<LocalCache<u64, AppModel>>),
    AppClientId(Arc<LocalCache<String, Option<u64>>>),
    AppSecret(Arc<LocalCache<AppSecretCacheKey, Vec<AppSecretRecrod>>>),
    AppFeature(Arc<LocalCache<u64, Vec<(String, bool)>>>),
    OAuthClient(Arc<LocalCache<u64, AppOAuthClientModel>>),
    OAuthServerScope(Arc<LocalCache<u64, Vec<AppOAuthServerScopeData>>>),
}

impl AppLocalCacheClear {
    pub fn new_clears(app: &AppDao) -> Vec<Self> {
        vec![
            Self::AppId(app.app.id_cache.clone()),
            Self::AppClientId(app.app.client_id_cache.clone()),
            Self::AppSecret(app.app_secret.secret_cache.clone()),
            Self::AppFeature(app.app.feature_cache.clone()),
            Self::OAuthClient(app.oauth_client.oauth_client_cache.clone()),
            Self::OAuthServerScope(app.oauth_server.oauth_server_scope_cache.clone()),
        ]
    }
}

#[async_trait]
impl LocalCacheClearItem for AppLocalCacheClear {
    fn cache_name(&self) -> &str {
        match self {
            Self::AppId(cache) => cache.config().cache_name,
            Self::AppClientId(cache) => cache.config().cache_name,
            Self::AppSecret(cache) => cache.config().cache_name,
            Self::AppFeature(cache) => cache.config().cache_name,
            Self::OAuthClient(cache) => cache.config().cache_name,
            Self::OAuthServerScope(cache) => cache.config().cache_name,
        }
    }
    async fn clear_from_message(&self, msg: &str) -> Result<(), String> {
        match self {
            Self::AppId(cache) => {
                cache
                    .del(&msg.parse::<u64>().map_err(|e| e.to_string())?)
                    .await
            }
            Self::AppSecret(cache) => {
                cache
                    .del(
                        &msg.parse::<AppSecretCacheKey>()
                            .map_err(|e| e.to_fluent_message().default_format())?,
                    )
                    .await
            }
            Self::AppClientId(cache) => cache.del(&msg.to_string()).await,
            Self::AppFeature(cache) => {
                cache
                    .del(&msg.parse::<u64>().map_err(|e| e.to_string())?)
                    .await
            }
            Self::OAuthClient(cache) => {
                cache
                    .del(&msg.parse::<u64>().map_err(|e| e.to_string())?)
                    .await
            }
            Self::OAuthServerScope(cache) => {
                cache
                    .del(&msg.parse::<u64>().map_err(|e| e.to_string())?)
                    .await
            }
        };
        Ok(())
    }
}
