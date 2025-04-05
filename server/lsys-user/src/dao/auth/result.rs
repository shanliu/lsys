//统一错误

use deadpool_redis::PoolError;
use lsys_access::dao::AccessError;

use lsys_core::{fluent_message, FluentMessage, IntoFluentMessage, ValidCodeError};

use redis::RedisError;

use std::string::FromUtf8Error;

use std::time::SystemTimeError;

#[derive(Debug)]
pub enum UserAuthError {
    TokenParse(FluentMessage),
    Sqlx(sqlx::Error),
    Redis(RedisError),
    RedisPool(PoolError),
    AccessError(AccessError),
    ValidCode(ValidCodeError),

    NotLogin(FluentMessage),
    System(FluentMessage),

    CheckUserLock((u64, FluentMessage)),
    CheckCaptchaNeed(FluentMessage),
    Utf8Err(FromUtf8Error),
}

impl IntoFluentMessage for UserAuthError {
    fn to_fluent_message(&self) -> FluentMessage {
        match self {
            UserAuthError::TokenParse(err) => err.to_owned(),
            UserAuthError::Sqlx(err) => fluent_message!("sqlx-error", err),
            UserAuthError::Redis(err) => fluent_message!("redis-error", err),
            UserAuthError::RedisPool(err) => fluent_message!("redis-error", err),
            UserAuthError::ValidCode(err) => err.to_fluent_message(),
            UserAuthError::NotLogin(err) => err.to_owned(),
            UserAuthError::AccessError(err) => err.to_fluent_message(),
            UserAuthError::System(err) => err.to_owned(),
            UserAuthError::CheckUserLock(err) => err.1.to_owned(),
            UserAuthError::CheckCaptchaNeed(err) => err.to_owned(),
            UserAuthError::Utf8Err(err) => fluent_message!("utf-parse-error", err),
        }
    }
}

pub type UserAuthResult<T> = Result<T, UserAuthError>;

impl From<sqlx::Error> for UserAuthError {
    fn from(err: sqlx::Error) -> Self {
        UserAuthError::Sqlx(err)
    }
}
impl From<SystemTimeError> for UserAuthError {
    fn from(err: SystemTimeError) -> Self {
        UserAuthError::System(fluent_message!("time-error", err))
    }
}
impl From<RedisError> for UserAuthError {
    fn from(err: RedisError) -> Self {
        UserAuthError::Redis(err)
    }
}
impl From<PoolError> for UserAuthError {
    fn from(err: PoolError) -> Self {
        UserAuthError::RedisPool(err)
    }
}

impl From<FromUtf8Error> for UserAuthError {
    fn from(err: FromUtf8Error) -> Self {
        UserAuthError::Utf8Err(err)
    }
}
impl From<ValidCodeError> for UserAuthError {
    fn from(err: ValidCodeError) -> Self {
        UserAuthError::ValidCode(err)
    }
}
impl From<AccessError> for UserAuthError {
    fn from(err: AccessError) -> Self {
        UserAuthError::AccessError(err)
    }
}
