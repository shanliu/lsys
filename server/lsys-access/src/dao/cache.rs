use std::{str::FromStr, sync::Arc};

use async_trait::async_trait;
use lsys_core::{
    cache::{LocalCache, LocalCacheClearItem},
    IntoFluentMessage,
};

use crate::model::{SessionModel, UserModel};

use super::{AccessAuthSessionCacheKey, AccessDao, AccessUserAppUserKey};

pub enum AccessLocalCacheClear {
    AuthSession(Arc<LocalCache<AccessAuthSessionCacheKey, SessionModel>>),
    AuthSessionData(Arc<LocalCache<AccessAuthSessionCacheKey, Vec<(String, String)>>>),
    AuthUser(Arc<LocalCache<u64, UserModel>>),
    AuthAppUserData(Arc<LocalCache<AccessUserAppUserKey, u64>>),
}

impl AccessLocalCacheClear {
    pub fn new_clears(access: &AccessDao) -> Vec<Self> {
        vec![
            Self::AuthUser(access.user.user_cache.clone()),
            Self::AuthAppUserData(access.user.app_user_data.clone()),
            Self::AuthSession(access.auth.session_cache.clone()),
            Self::AuthSessionData(access.auth.session_data_cache.clone()),
        ]
    }
}

#[async_trait]
impl LocalCacheClearItem<'_> for AccessLocalCacheClear {
    fn cache_name(&self) -> &str {
        match self {
            Self::AuthUser(cache) => cache.config().cache_name,
            Self::AuthAppUserData(cache) => cache.config().cache_name,
            Self::AuthSession(cache) => cache.config().cache_name,
            Self::AuthSessionData(cache) => cache.config().cache_name,
        }
    }
    async fn clear_from_message(&self, msg: &str) -> Result<(), String> {
        match self {
            Self::AuthUser(cache) => {
                cache
                    .del(&msg.parse::<u64>().map_err(|e| e.to_string())?)
                    .await
            }
            Self::AuthAppUserData(cache) => {
                cache
                    .del(
                        &AccessUserAppUserKey::from_str(msg)
                            .map_err(|e| e.to_fluent_message().default_format())?,
                    )
                    .await
            }
            Self::AuthSession(cache) => {
                cache
                    .del(
                        &AccessAuthSessionCacheKey::from_str(msg)
                            .map_err(|e| e.to_fluent_message().default_format())?,
                    )
                    .await
            }
            Self::AuthSessionData(cache) => {
                cache
                    .del(
                        &AccessAuthSessionCacheKey::from_str(msg)
                            .map_err(|e| e.to_fluent_message().default_format())?,
                    )
                    .await
            }
        };
        Ok(())
    }
}
