// use std::error::Error;

use std::sync::Arc;
#[macro_use]
mod macros;

pub use access::*;
pub use cache::*;
pub use check::*;
pub use data::*;
use logger::*;
use lsys_core::cache::{LocalCache, LocalCacheConfig};
use lsys_core::RemoteNotify;
use lsys_logger::dao::ChangeLogger;
pub use res::*;
pub use role::*;
use sqlx::{MySql, Pool};
pub use tags::*;

mod access;
mod cache;
mod check;
mod data;
mod res;
// mod res_tpl;
mod logger;
mod result;
mod role;
mod tags;
pub use result::*;

pub const PRIORITY_MAX: i8 = 100;
pub const PRIORITY_MIN: i8 = 0;

pub struct Rbac {
    pub res: Arc<RbacRes>,
    pub role: Arc<RbacRole>,
    pub access: Arc<RbacAccess>,
    pub data: Arc<RbacData>,
    pub(crate) role_relation_cache: Arc<LocalCache<String, Option<RoleDetailRow>>>,
    pub(crate) role_access_cache: Arc<LocalCache<String, Option<RoleAccessRow>>>,
    pub(crate) res_key_cache: Arc<LocalCache<ResKey, Option<RbacResData>>>,
}

impl Rbac {
    pub fn new(
        db: Pool<MySql>,
        remote_notify: Arc<RemoteNotify>,
        system_role: Option<Box<dyn SystemRoleCheckData>>,
        use_cache: bool,
        logger: Arc<ChangeLogger>,
    ) -> Self {
        let tags = Arc::from(RbacTags::new(db.clone(), logger.clone()));
        let res_key_cache = Arc::from(LocalCache::new(
            remote_notify.clone(),
            LocalCacheConfig::new("key-res"),
        ));
        let role_relation_cache = Arc::from(LocalCache::new(
            remote_notify.clone(),
            LocalCacheConfig::new("role-relation"),
        ));
        let role_access_cache = Arc::from(LocalCache::new(
            remote_notify,
            LocalCacheConfig::new("role-access"),
        ));

        let role = Arc::from(RbacRole::new(
            db.clone(),
            // fluent.clone(),
            tags.clone(),
            role_relation_cache.clone(),
            role_access_cache.clone(),
            logger.clone(),
        ));
        let res = Arc::from(RbacRes::new(
            db,
            // fluent.clone(),
            tags.clone(),
            role.clone(),
            res_key_cache.clone(),
            logger,
        ));
        let access = Arc::from(RbacAccess::new(
            // fluent,
            res.clone(),
            role.clone(),
            system_role,
            use_cache,
        ));
        let data = Arc::from(RbacData::new(res.clone(), role.clone(), tags));
        Rbac {
            res,
            role,
            access,
            data,
            res_key_cache,
            role_relation_cache,
            role_access_cache,
        }
    }
}
