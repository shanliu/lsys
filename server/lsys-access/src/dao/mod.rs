mod auth;
mod cache;
mod result;
mod session;
mod user;
use std::sync::Arc;

pub use auth::*;
pub use cache::*;
use lsys_core::{cache::LocalCacheConfig, RemoteNotify};
pub use result::*;
pub use session::*;
use sqlx::{MySql, Pool};
pub use user::*;

pub struct AccessDao {
    //   pub redis: deadpool_redis::Pool,
    // 权限相关
    pub auth: Arc<AccessAuth>,
    pub user: Arc<AccessUser>,
}

pub struct AccessConfig {
    pub auth_cache: LocalCacheConfig,
    pub user_cache: LocalCacheConfig,
}

impl AccessConfig {
    pub fn new(use_cache: bool) -> Self {
        Self {
            auth_cache: LocalCacheConfig::new("auth", if use_cache { None } else { Some(0) }, None),
            user_cache: LocalCacheConfig::new("user", if use_cache { None } else { Some(0) }, None),
        }
    }
}

impl AccessDao {
    pub fn new(
        db: Pool<MySql>,
        remote_notify: Arc<RemoteNotify>,
        config: AccessConfig,
    ) -> AccessDao {
        let user = Arc::from(AccessUser::new(
            db.clone(),
            remote_notify.clone(),
            config.auth_cache,
        ));
        let auth = Arc::from(AccessAuth::new(
            db,
            user.clone(),
            remote_notify,
            config.user_cache,
        ));
        Self { auth, user }
    }
}
