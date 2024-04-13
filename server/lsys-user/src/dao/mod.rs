use std::sync::Arc;

pub mod account;
pub mod auth;

use crate::dao::auth::UserAuth;

use lsys_core::{AppCoreError, RemoteNotify};

use lsys_logger::dao::ChangeLogger;
use lsys_setting::dao::SingleSetting;
use sqlx::{MySql, Pool};

use self::account::{UserAccount, UserAccountConfig};
use self::auth::{UserAuthConfig, UserAuthStore};


pub struct UserConfig{
    pub account:UserAccountConfig,
    pub oauth:UserAuthConfig,
}


impl UserConfig {
    pub fn new(use_cache:bool) -> Self {
        Self {
            account:UserAccountConfig::new(use_cache),
            oauth:UserAuthConfig::new(use_cache),
        }
    }
}




pub struct UserDao<T: UserAuthStore> {
    //内部依赖
    pub db: Pool<MySql>,
    pub redis: deadpool_redis::Pool,
    // 授权相关
    pub user_auth: Arc<UserAuth<T>>,
    // 账号相关
    pub user_account: Arc<UserAccount>,
}

impl<T: UserAuthStore + Send + Sync> UserDao<T> {
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        db: Pool<MySql>,
        redis: deadpool_redis::Pool,
        setting: Arc<SingleSetting>,
        logger: Arc<ChangeLogger>,
        remote_notify: Arc<RemoteNotify>,
        store: T,
        config: UserConfig,
    ) -> Result<UserDao<T>, AppCoreError> {
        let user_account = Arc::from(UserAccount::new(
            db.clone(),
            redis.clone(),
            setting,
            remote_notify.clone(),
            config.account,
            logger,
        ));
        let user_auth = Arc::from(UserAuth::new(
            db.clone(),
            redis.clone(),
            remote_notify.clone(),
            user_account.clone(),
            store,
            config.oauth,
        ));
        Ok(UserDao {
            user_auth,
            user_account,
            db,
            redis,
        })
    }
}
