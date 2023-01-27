use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::Arc;
#[macro_use]
mod macros;

pub use access::*;
pub use cache::*;
pub use data::*;
use lsys_core::cache::{LocalCache, LocalCacheConfig};
use lsys_core::FluentMessage;
use redis::aio::ConnectionManager;
pub use res::*;
pub use role::*;
use sqlx::{MySql, Pool};
pub use tags::*;
use tokio::sync::Mutex;

mod access;
mod cache;
mod data;
mod res;
mod role;
mod tags;

pub const PRIORITY_MAX: i8 = 100;
pub const PRIORITY_MIN: i8 = 0;

#[derive(Debug)]
pub enum UserRbacError {
    NotLogin(String),
    Sqlx(sqlx::Error),
    System(String),
    Check(Vec<(String, String)>),
    // Access(String),
}
impl Display for UserRbacError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl Error for UserRbacError {}

pub type UserRbacResult<T> = Result<T, UserRbacError>;

impl From<sqlx::Error> for UserRbacError {
    fn from(err: sqlx::Error) -> Self {
        UserRbacError::Sqlx(err)
    }
}

pub struct Rbac {
    pub res: Arc<RbacRes>,
    pub role: Arc<RbacRole>,
    pub access: Arc<RbacAccess>,
    pub data: Arc<RbacData>,
    pub role_relation_cache: Arc<LocalCache<String, Option<RoleDetailRow>>>,
    pub role_access_cache: Arc<LocalCache<String, Option<RoleAccessRow>>>,
    pub res_key_cache: Arc<LocalCache<ResKey, Option<RbacResData>>>,
}

impl Rbac {
    pub fn new(
        fluent: Arc<FluentMessage>,
        db: Pool<MySql>,
        redis: Arc<Mutex<ConnectionManager>>,
        system_role: Option<Box<dyn SystemRoleCheckData>>,
        use_cache: bool,
    ) -> Self {
        let tags = Arc::from(RbacTags::new(db.clone()));
        let res_key_cache = Arc::from(LocalCache::new(
            redis.clone(),
            LocalCacheConfig::new("key-res"),
        ));
        let role_relation_cache = Arc::from(LocalCache::new(
            redis.clone(),
            LocalCacheConfig::new("role-relation"),
        ));
        let role_access_cache =
            Arc::from(LocalCache::new(redis, LocalCacheConfig::new("role-access")));

        let role = Arc::from(RbacRole::new(
            db.clone(),
            fluent.clone(),
            tags.clone(),
            role_relation_cache.clone(),
            role_access_cache.clone(),
        ));
        let res = Arc::from(RbacRes::new(
            db,
            fluent.clone(),
            tags.clone(),
            role.clone(),
            res_key_cache.clone(),
        ));

        let access = Arc::from(RbacAccess::new(
            fluent,
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
