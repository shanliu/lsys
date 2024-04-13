// use std::error::Error;

use std::sync::Arc;
#[macro_use]
mod macros;

pub use access::*;
pub use cache::*;
pub use check::*;
pub use data::*;
use logger::*;
use lsys_core::cache:: LocalCacheConfig;
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




pub struct RbacConfig{
    pub role_cache:RbacRoleConfig,
    pub res_cache:LocalCacheConfig,
}

impl RbacConfig {
    pub fn new(use_cache:bool) -> Self {
        Self {
            role_cache:RbacRoleConfig::new(use_cache),
            res_cache:LocalCacheConfig::new("rbac-res",if use_cache{None}else{Some(0)},None),
        }
    }
}

pub struct Rbac {
    pub res: Arc<RbacRes>,
    pub role: Arc<RbacRole>,
    pub access: Arc<RbacAccess>,
    pub data: Arc<RbacData>,
    // pub(crate) role_relation_cache: Arc<LocalCache<String, Option<RoleDetailRow>>>,
    // pub(crate) role_access_cache: Arc<LocalCache<String, Option<RoleAccessRow>>>,
    // pub(crate) res_key_cache: Arc<LocalCache<ResKey, Option<RbacResData>>>,
}

impl Rbac {
    pub fn new(
        db: Pool<MySql>,
        system_role: Option<Box<dyn SystemRoleCheckData>>,
        remote_notify: Arc<RemoteNotify>,
        config:RbacConfig,
        logger: Arc<ChangeLogger>,
    ) -> Self {
      
        let tags = Arc::from(RbacTags::new(db.clone(), logger.clone()));
        let role = Arc::from(RbacRole::new(
            db.clone(),
            // fluent.clone(),
            tags.clone(),
            remote_notify.clone(),
            config.role_cache,
            logger.clone(),
        ));
        let res = Arc::from(RbacRes::new(
            db,
            // fluent.clone(),
            tags.clone(),
            role.clone(),
            remote_notify.clone(),
            config.res_cache,
            logger,
        ));
        let access = Arc::from(RbacAccess::new(
            // fluent,
            res.clone(),
            role.clone(),
            system_role,
        ));
        let data = Arc::from(RbacData::new(res.clone(), role.clone(), tags));
        Rbac {
            res,
            role,
            access,
            data,
            // res_key_cache,
            // role_relation_cache,
            // role_access_cache,
        }
    }
}
