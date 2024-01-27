use lsys_core::{fluent_message, AppCoreError, FluentMessage, RemoteNotify};
use lsys_user::dao::account::UserAccount;
use std::{
    error::Error,
    fmt::{Display, Formatter},
    time::SystemTimeError,
};

use deadpool_redis::PoolError;

use lsys_user::dao::account::UserAccountError;

use redis::RedisError;

use self::app::{Apps, AppsOauth, SubApps};
use lsys_logger::dao::ChangeLogger;
use sqlx::{MySql, Pool};
use std::sync::Arc;
pub mod app;
pub mod session;

#[derive(Debug)]
pub enum AppsError {
    Sqlx(sqlx::Error),
    System(FluentMessage),
    Redis(RedisError),
    RedisPool(PoolError),
    ScopeNotFind(FluentMessage),
    UserAccount(UserAccountError),
    SerdeJson(serde_json::Error),
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
        AppsError::Redis(err)
    }
}
impl From<PoolError> for AppsError {
    fn from(err: PoolError) -> Self {
        AppsError::RedisPool(err)
    }
}
impl From<SystemTimeError> for AppsError {
    fn from(err: SystemTimeError) -> Self {
        AppsError::System(fluent_message!("time-error", err))
    }
}
impl From<serde_json::Error> for AppsError {
    fn from(err: serde_json::Error) -> Self {
        AppsError::SerdeJson(err)
    }
}
impl From<UserAccountError> for AppsError {
    fn from(err: UserAccountError) -> Self {
        AppsError::UserAccount(err)
    }
}

pub type AppsResult<T> = Result<T, AppsError>;

pub struct AppDao {
    //内部依赖
    pub app: Arc<Apps>,
    pub sub_app: Arc<SubApps>,
    pub app_oauth: Arc<AppsOauth>,
    pub db: Pool<MySql>,
    pub redis: deadpool_redis::Pool,
}

impl AppDao {
    pub async fn new(
        user_account: Arc<UserAccount>,
        db: Pool<MySql>,
        redis: deadpool_redis::Pool,
        remote_notify: Arc<RemoteNotify>,
        logger: Arc<ChangeLogger>,
        time_out: u64,
    ) -> Result<AppDao, AppCoreError> {
        let sub_app = Arc::from(SubApps::new(
            db.clone(),
            remote_notify.clone(),
            logger.clone(),
        ));
        let app = Arc::from(Apps::new(
            db.clone(),
            remote_notify.clone(),
            // fluent.clone(),
            logger,
            sub_app.clone(),
        ));
        let app_oauth = Arc::from(AppsOauth::new(
            app.clone(),
            user_account,
            db.clone(),
            redis.clone(),
            // fluent.clone(),
            remote_notify,
            time_out,
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
