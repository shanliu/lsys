// use std::error::Error;
// use std::fmt::{Display, Formatter};

use std::time::SystemTimeError;

use deadpool_redis::PoolError;
use lsys_access::dao::AccessError;
use lsys_core::{fluent_message, FluentMessage, IntoFluentMessage, ValidCodeError};

use lsys_setting::dao::SettingError;
use redis::RedisError;

use crate::dao::UserAuthError;
#[derive(Debug)]
pub enum AccountError {
    Sqlx(sqlx::Error),
    System(FluentMessage),
    Status((u64, FluentMessage)),
    Redis(RedisError),
    RedisPool(PoolError),
    SerdeJson(serde_json::Error),
    ValidCode(ValidCodeError),
    Setting(SettingError),
    Param(FluentMessage),
    AuthStatusError((u64, FluentMessage)),
    UserAuthError(UserAuthError),
    AccessError(AccessError),
    PasswordNotMatch((u64, FluentMessage)),
    PasswordNotSet((u64, FluentMessage)),
    UserNotFind(FluentMessage),
}

impl IntoFluentMessage for AccountError {
    fn to_fluent_message(&self) -> FluentMessage {
        match self {
            Self::Sqlx(err) => fluent_message!("sqlx-error", err),
            Self::System(err) => err.to_owned(),
            Self::Status((_, err)) => err.to_owned(),
            Self::Redis(err) => fluent_message!("redis-error", err),
            Self::RedisPool(err) => fluent_message!("redis-error", err),
            Self::ValidCode(err) => err.to_fluent_message(),
            Self::Setting(err) => err.to_fluent_message(),
            Self::Param(err) => err.to_owned(),
            Self::AccessError(err) => err.to_fluent_message(),
            Self::AuthStatusError(err) => err.1.to_owned(),
            Self::PasswordNotMatch(err) => err.1.to_owned(),
            Self::PasswordNotSet(err) => err.1.to_owned(),
            Self::UserAuthError(err) => err.to_fluent_message(),
            Self::UserNotFind(err) => err.to_owned(),
            Self::SerdeJson(err) => fluent_message!("serde-json-error", err),
        }
    }
}

// impl Display for AccountError {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{:?}", self)
//     }
// }
// impl Error for AccountError {}

impl AccountError {
    pub fn is_not_found(&self) -> bool {
        matches!(self, AccountError::Sqlx(sqlx::Error::RowNotFound))
    }
}

pub type AccountResult<T> = Result<T, AccountError>;
impl From<serde_json::Error> for AccountError {
    fn from(err: serde_json::Error) -> Self {
        AccountError::SerdeJson(err)
    }
}
impl From<AccessError> for AccountError {
    fn from(err: AccessError) -> Self {
        AccountError::AccessError(err)
    }
}
impl From<UserAuthError> for AccountError {
    fn from(err: UserAuthError) -> Self {
        AccountError::UserAuthError(err)
    }
}
impl From<sqlx::Error> for AccountError {
    fn from(err: sqlx::Error) -> Self {
        AccountError::Sqlx(err)
    }
}
impl From<RedisError> for AccountError {
    fn from(err: RedisError) -> Self {
        AccountError::Redis(err)
    }
}
impl From<PoolError> for AccountError {
    fn from(err: PoolError) -> Self {
        AccountError::RedisPool(err)
    }
}
impl From<SystemTimeError> for AccountError {
    fn from(err: SystemTimeError) -> Self {
        AccountError::System(fluent_message!("time-error", err))
    }
}

impl From<ValidCodeError> for AccountError {
    fn from(err: ValidCodeError) -> Self {
        AccountError::ValidCode(err)
    }
}
impl From<SettingError> for AccountError {
    fn from(err: SettingError) -> Self {
        AccountError::Setting(err)
    }
}
