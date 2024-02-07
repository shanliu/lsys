// use std::error::Error;
// use std::fmt::{Display, Formatter};

use std::time::SystemTimeError;

use deadpool_redis::PoolError;
use lsys_core::{fluent_message, FluentMessage, IntoFluentMessage, ValidCodeError};

use lsys_setting::dao::SettingError;
use redis::RedisError;
#[derive(Debug)]
pub enum UserAccountError {
    Sqlx(sqlx::Error),
    System(FluentMessage),
    Status((u64, FluentMessage)),
    Redis(RedisError),
    RedisPool(PoolError),
    ValidCode(ValidCodeError),
    Setting(SettingError),
    Param(FluentMessage),
}

impl IntoFluentMessage for UserAccountError {
    fn to_fluent_message(&self) -> FluentMessage {
        match self {
            UserAccountError::Sqlx(err) => fluent_message!("sqlx-error", err),
            UserAccountError::System(err) => err.to_owned(),
            UserAccountError::Status((_, err)) => err.to_owned(),
            UserAccountError::Redis(err) => fluent_message!("redis-error", err),
            UserAccountError::RedisPool(err) => fluent_message!("redis-error", err),
            UserAccountError::ValidCode(err) => err.to_fluent_message(),
            UserAccountError::Setting(err) => err.to_fluent_message(),
            UserAccountError::Param(err) => err.to_owned(),
        }
    }
}

// impl Display for UserAccountError {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{:?}", self)
//     }
// }
// impl Error for UserAccountError {}

impl UserAccountError {
    pub fn is_not_found(&self) -> bool {
        matches!(self, UserAccountError::Sqlx(sqlx::Error::RowNotFound))
    }
}

pub type UserAccountResult<T> = Result<T, UserAccountError>;

impl From<sqlx::Error> for UserAccountError {
    fn from(err: sqlx::Error) -> Self {
        UserAccountError::Sqlx(err)
    }
}
impl From<RedisError> for UserAccountError {
    fn from(err: RedisError) -> Self {
        UserAccountError::Redis(err)
    }
}
impl From<PoolError> for UserAccountError {
    fn from(err: PoolError) -> Self {
        UserAccountError::RedisPool(err)
    }
}
impl From<SystemTimeError> for UserAccountError {
    fn from(err: SystemTimeError) -> Self {
        UserAccountError::System(fluent_message!("time-error", err))
    }
}

impl From<ValidCodeError> for UserAccountError {
    fn from(err: ValidCodeError) -> Self {
        UserAccountError::ValidCode(err)
    }
}
impl From<SettingError> for UserAccountError {
    fn from(err: SettingError) -> Self {
        UserAccountError::Setting(err)
    }
}
