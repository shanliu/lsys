mod auth;
mod cache;
mod result;
mod session;
mod user;
use std::sync::Arc;

pub use auth::*;
pub use cache::*;
use lsys_core::cache::LocalCacheConfig;
use lsys_core::remote_notify::RemoteNotify;
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

/// 按 login_type 分别控制会话数量上限
pub struct SessionLimitConfig {
    per_type: std::collections::HashMap<String, u32>,
}

impl SessionLimitConfig {
    pub fn new(per_type: std::collections::HashMap<String, u32>) -> Self {
        Self { per_type }
    }

    /// 返回指定 login_type 的上限，0 表示不限制
    pub fn limit_for(&self, login_type: &str) -> u32 {
        self.per_type.get(login_type).copied().unwrap_or(0)
    }
}

pub struct AccessConfig {
    pub auth_cache: LocalCacheConfig,
    pub user_cache: LocalCacheConfig,
    pub session_limit: SessionLimitConfig,
}

impl AccessConfig {
    pub fn new(use_cache: bool, session_limit: SessionLimitConfig) -> Self {
        Self {
            auth_cache: LocalCacheConfig::new("auth", if use_cache { None } else { Some(0) }, None),
            user_cache: LocalCacheConfig::new("user", if use_cache { None } else { Some(0) }, None),
            session_limit,
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
            config.session_limit,
        ));
        Self { auth, user }
    }
}
