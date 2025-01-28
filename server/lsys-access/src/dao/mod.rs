mod auth;
mod cache;
mod oauth;
mod result;
mod session;
mod user;
use std::sync::Arc;

pub use auth::*;
pub use user::*;
pub use cache::*;
use lsys_core::{cache::LocalCacheConfig, RemoteNotify};
pub use oauth::*;
pub use result::*;
pub use session::*;
use sqlx::{MySql, Pool};

pub struct AccessDao {
    //   pub redis: deadpool_redis::Pool,
    // 权限相关
    pub auth: Arc<AccessAuth>,
    pub oauth: Arc<AccessOAuth>,
    pub user:Arc<AccessUser>,
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
        redis: deadpool_redis::Pool,
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
        let oauth = Arc::from(AccessOAuth::new(
            auth.clone(),
            redis,
        ));
        Self { auth, oauth,user }
    }
}
