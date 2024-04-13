use lsys_core::{fluent_message, FluentMessage, IntoFluentMessage};
use std::{
    // error::Error,
    // fmt::{Display, Formatter},
    time::SystemTimeError,
};

use deadpool_redis::PoolError;

use lsys_user::dao::account::UserAccountError;

use redis::RedisError;

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
impl AppsError {
    pub fn app_not_found(&self)->bool{
        matches!(self, Self::Sqlx(sqlx::Error::RowNotFound))
    }
}
impl IntoFluentMessage for AppsError {
    fn to_fluent_message(&self) -> FluentMessage {
        match self {
            AppsError::System(e) => e.to_owned(),
            AppsError::ScopeNotFind(e) => e.to_owned(),
            AppsError::Sqlx(e) => {
                fluent_message!("sqlx-error", e)
            }
            AppsError::Redis(e) => {
                fluent_message!("redis-error", e)
            }
            AppsError::RedisPool(e) => {
                fluent_message!("redis-error", e)
            }
            AppsError::UserAccount(e) => e.to_fluent_message(),
            AppsError::SerdeJson(e) => {
                fluent_message!("serde-json-error", e)
            }
        }
    }
}

// impl Display for AppsError {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{:?}", self)
//     }
// }

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
