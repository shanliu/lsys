use std::{str::FromStr, sync::Arc};

use async_trait::async_trait;
use lsys_core::{
    cache::{LocalCache, LocalCacheClearItem},
    IntoFluentMessage,
};

use super::{Rbac, RbacResData, ResKey, RoleAccessRow, RoleDetailRow};

//RBAC 授权缓存
pub enum RbacLocalCacheClear {
    ResKey(Arc<LocalCache<ResKey, Option<RbacResData>>>),
    RoleRelation(Arc<LocalCache<String, Option<RoleDetailRow>>>),
    RoleAccess(Arc<LocalCache<String, Option<RoleAccessRow>>>),
}

impl RbacLocalCacheClear {
    pub fn new_clears(rbac: &Rbac) -> Vec<Self> {
        vec![
            RbacLocalCacheClear::ResKey(rbac.res_key_cache.clone()),
            RbacLocalCacheClear::RoleRelation(rbac.role_relation_cache.clone()),
            RbacLocalCacheClear::RoleAccess(rbac.role_access_cache.clone()),
        ]
    }
}

#[async_trait]
impl LocalCacheClearItem for RbacLocalCacheClear {
    fn cache_name(&self) -> &str {
        match self {
            RbacLocalCacheClear::ResKey(cache) => cache.config().cache_name,
            RbacLocalCacheClear::RoleRelation(cache) => cache.config().cache_name,
            RbacLocalCacheClear::RoleAccess(cache) => cache.config().cache_name,
        }
    }
    async fn clear_from_message(&self, msg: &str) -> Result<(), String> {
        match self {
            RbacLocalCacheClear::ResKey(cache) => {
                cache
                    .del(
                        &ResKey::from_str(msg)
                            .map_err(|e| e.to_fluent_message().default_format())?,
                    )
                    .await
            }
            RbacLocalCacheClear::RoleRelation(cache) => cache.del(&msg.to_string()).await,
            RbacLocalCacheClear::RoleAccess(cache) => cache.del(&msg.to_string()).await,
        };
        Ok(())
    }
}
