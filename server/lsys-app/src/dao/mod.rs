use lsys_core::{cache:: LocalCacheConfig, AppCoreError, RemoteNotify};
use lsys_user::dao::account::UserAccount;

use self::app::{Apps, AppsOauth,  SubApps};
use lsys_logger::dao::ChangeLogger;
use sqlx::{MySql, Pool};
use std::sync::Arc;
pub mod app;

mod result;
pub mod session;
pub use result::*;

pub struct AppDao {
    //内部依赖
    pub app: Arc<Apps>,
    pub sub_app: Arc<SubApps>,
    pub app_oauth: Arc<AppsOauth>,
    pub db: Pool<MySql>,
    pub redis: deadpool_redis::Pool,
}

pub struct AppConfig{
    pub app_cache:LocalCacheConfig,
    pub sub_app_cache:LocalCacheConfig,
    pub oauth_cache:LocalCacheConfig,
}

impl  AppConfig {
    pub fn new(use_cache:bool) -> Self {
        Self { 
            sub_app_cache:LocalCacheConfig::new("sub-app",if use_cache{None}else{Some(0)},None), 
            app_cache: LocalCacheConfig::new("app",if use_cache{None}else{Some(0)},None),
            oauth_cache:LocalCacheConfig::new("oauth",if use_cache{None}else{Some(0)},None),
        }
    }
}


impl AppDao {
    pub async fn new(
        user_account: Arc<UserAccount>,
        db: Pool<MySql>,
        redis: deadpool_redis::Pool,
        remote_notify: Arc<RemoteNotify>,
        config: AppConfig,
        logger: Arc<ChangeLogger>,
        time_out: u64,
    ) -> Result<AppDao, AppCoreError> {
        let sub_app = Arc::from(SubApps::new(
            db.clone(),
            remote_notify.clone(),
            config.sub_app_cache,
            logger.clone(),
        ));
        let app = Arc::from(Apps::new(
            db.clone(),
            sub_app.clone(),
            remote_notify.clone(),
            config.app_cache,
            logger,
        ));
        let app_oauth = Arc::from(AppsOauth::new(
            app.clone(),
            user_account,
            db.clone(),
            redis.clone(),
            time_out,
            remote_notify.clone(),
            config.oauth_cache,
        ));
        Ok(AppDao {
            app,
            sub_app,
            app_oauth,
            db,
            // fluent,
            redis,
        })
    }
}
