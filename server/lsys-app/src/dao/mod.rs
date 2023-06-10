use lsys_core::{AppCore, AppCoreError, FluentMessage, RemoteNotify};
use lsys_user::dao::account::UserAccount;
use std::{
    error::Error,
    fmt::{Display, Formatter},
    time::SystemTimeError,
};

use deadpool_redis::PoolError;

use lsys_user::dao::account::UserAccountError;

use redis::RedisError;

use self::app::{Apps, AppsOauth};
use lsys_logger::dao::ChangeLogger;
use sqlx::{MySql, Pool};
use std::sync::Arc;
pub mod app;
pub mod session;

#[derive(Debug)]
pub enum AppsError {
    Sqlx(sqlx::Error),
    System(String),
    Redis(String),
    ScopeNotFind(String),
}
impl Display for AppsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl Error for AppsError {}

impl From<sqlx::Error> for AppsError {
    fn from(err: sqlx::Error) -> Self {
        AppsError::Sqlx(err)
    }
}
impl From<RedisError> for AppsError {
    fn from(err: RedisError) -> Self {
        AppsError::Redis(err.to_string())
    }
}
impl From<PoolError> for AppsError {
    fn from(err: PoolError) -> Self {
        AppsError::Redis(err.to_string())
    }
}
impl From<SystemTimeError> for AppsError {
    fn from(err: SystemTimeError) -> Self {
        AppsError::System(err.to_string())
    }
}
impl From<serde_json::Error> for AppsError {
    fn from(err: serde_json::Error) -> Self {
        AppsError::System(format!("{:?}", err))
    }
}
impl From<UserAccountError> for AppsError {
    fn from(err: UserAccountError) -> Self {
        AppsError::System(format!("user error {:?}", err))
    }
}

pub type AppsResult<T> = Result<T, AppsError>;

pub struct AppDao {
    //内部依赖
    pub app: Arc<Apps>,
    pub app_oauth: Arc<AppsOauth>,
    pub db: Pool<MySql>,
    pub redis: deadpool_redis::Pool,
    pub(crate) fluent: Arc<FluentMessage>,
}

impl AppDao {
    pub async fn new(
        user_account: Arc<UserAccount>,
        app_core: Arc<AppCore>,
        db: Pool<MySql>,
        redis: deadpool_redis::Pool,
        remote_notify: Arc<RemoteNotify>,
        logger: Arc<ChangeLogger>,
        time_out: u64,
    ) -> Result<AppDao, AppCoreError> {
        let app_locale_dir = app_core.app_dir.join("locale/lsys-app");
        let fluent = Arc::new(if app_locale_dir.exists() {
            app_core.create_fluent(app_locale_dir).await?
        } else {
            let cargo_dir = env!("CARGO_MANIFEST_DIR");
            app_core
                .create_fluent(cargo_dir.to_owned() + "/locale")
                .await?
        });

        let app = Arc::from(Apps::new(
            app_core.clone(),
            db.clone(),
            remote_notify.clone(),
            fluent.clone(),
            logger,
        ));
        let app_oauth = Arc::from(AppsOauth::new(
            app.clone(),
            user_account,
            db.clone(),
            redis.clone(),
            fluent.clone(),
            remote_notify,
            time_out,
        ));
        Ok(AppDao {
            app,
            app_oauth,
            db,
            fluent,
            redis,
        })
    }
}
