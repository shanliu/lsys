use std::sync::Arc;

pub mod account;
pub mod auth;

use crate::dao::auth::UserAuth;
use lsys_core::{AppCore, AppCoreError, FluentMessage};

use lsys_setting::dao::SingleSetting;
use sqlx::{MySql, Pool};

use self::account::UserAccount;
use self::auth::{UserAuthConfig, UserAuthStore};

pub struct UserDao<T: UserAuthStore> {
    //内部依赖
    pub fluent: Arc<FluentMessage>,
    pub db: Pool<MySql>,
    pub redis: deadpool_redis::Pool,
    // 授权相关
    pub user_auth: Arc<UserAuth<T>>,
    // 账号相关
    pub user_account: Arc<UserAccount>,
}

impl<T: UserAuthStore + Send + Sync> UserDao<T> {
    pub async fn new(
        app_core: Arc<AppCore>,
        db: Pool<MySql>,
        redis: deadpool_redis::Pool,
        setting: Arc<SingleSetting>,
        store: T,
        config: Option<UserAuthConfig>,
    ) -> Result<UserDao<T>, AppCoreError> {
        let app_locale_dir = app_core.app_dir.join("locale/lsys-user");
        let fluent = Arc::new(if app_locale_dir.exists() {
            app_core.create_fluent(app_locale_dir).await?
        } else {
            let cargo_dir = env!("CARGO_MANIFEST_DIR");
            app_core
                .create_fluent(cargo_dir.to_owned() + "/locale")
                .await?
        });

        let user_account = Arc::from(UserAccount::new(
            db.clone(),
            redis.clone(),
            fluent.clone(),
            setting,
        ));
        let user_auth = Arc::from(UserAuth::new(
            db.clone(),
            redis.clone(),
            fluent.clone(),
            user_account.clone(),
            store,
            config,
        ));
        Ok(UserDao {
            user_auth,
            user_account,
            fluent,
            db,
            redis,
        })
    }
}
