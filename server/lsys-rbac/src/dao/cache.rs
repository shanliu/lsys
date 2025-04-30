//验证结果缓存
use async_trait::async_trait;
use lsys_core::{
    cache::{LocalCache, LocalCacheClearItem},
    IntoFluentMessage,
};
use std::{str::FromStr, sync::Arc};

use crate::model::{RbacOpModel, RbacResModel};

use super::{op::OpCacheKey, res::ResCacheKey, role::AccessRoleRow, RbacDao};

//RBAC 授权缓存
pub enum RbacLocalCacheClear {
    OpCacheKey(Arc<LocalCache<OpCacheKey, Option<RbacOpModel>>>),
    ResCacheKey(Arc<LocalCache<ResCacheKey, Option<RbacResModel>>>),
    RbacRoleCache(Arc<LocalCache<String, Vec<AccessRoleRow>>>),
}

impl RbacLocalCacheClear {
    pub fn new_clears(rbac: &RbacDao) -> Vec<Self> {
        vec![
            RbacLocalCacheClear::OpCacheKey(rbac.op.cache_op_data.clone()),
            RbacLocalCacheClear::ResCacheKey(rbac.res.cache_res_data.clone()),
            RbacLocalCacheClear::RbacRoleCache(rbac.role.cache_access.clone()),
        ]
    }
}

#[async_trait]
impl LocalCacheClearItem<'_> for RbacLocalCacheClear {
    fn cache_name(&self) -> &str {
        match self {
            RbacLocalCacheClear::OpCacheKey(cache) => cache.config().cache_name,
            RbacLocalCacheClear::ResCacheKey(cache) => cache.config().cache_name,
            RbacLocalCacheClear::RbacRoleCache(cache) => cache.config().cache_name,
        }
    }
    async fn clear_from_message(&self, msg: &str) -> Result<(), String> {
        match self {
            RbacLocalCacheClear::OpCacheKey(cache) => {
                cache
                    .del(
                        &OpCacheKey::from_str(msg)
                            .map_err(|e| e.to_fluent_message().default_format())?,
                    )
                    .await
            }
            RbacLocalCacheClear::ResCacheKey(cache) => {
                cache
                    .del(
                        &ResCacheKey::from_str(msg)
                            .map_err(|e| e.to_fluent_message().default_format())?,
                    )
                    .await
            }
            RbacLocalCacheClear::RbacRoleCache(cache) => cache.del(&msg.to_owned()).await,
        };
        Ok(())
    }
}
