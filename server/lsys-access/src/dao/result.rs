//统一错误

use deadpool_redis::PoolError;

use lsys_core::{fluent_message, FluentMessage, IntoFluentMessage, ValidError};

use redis::RedisError;

use std::time::SystemTimeError;

#[derive(Debug)]
pub enum AccessError {
    Sqlx(sqlx::Error),
    Redis(RedisError),
    RedisPool(PoolError),
    NotLogin,
    IsLogout,
    LoginTokenDataExit(u64),
    System(FluentMessage),
    SerdeJson(serde_json::Error),
    BadAccount(FluentMessage),
    Vaild(ValidError),
}

impl IntoFluentMessage for AccessError {
    fn to_fluent_message(&self) -> FluentMessage {
        match self {
            AccessError::LoginTokenDataExit(id) => {
                fluent_message!("access-token-data-exits",{"id":id,})
            }
            AccessError::NotLogin => fluent_message!("access-not-login"),
            AccessError::IsLogout => fluent_message!("access-not-login"),
            AccessError::Sqlx(err) => fluent_message!("sqlx-error", err),
            AccessError::Redis(err) => fluent_message!("redis-error", err),
            AccessError::RedisPool(err) => fluent_message!("redis-error", err),
            AccessError::System(err) => err.to_owned(),
            AccessError::BadAccount(err) => err.to_owned(),
            AccessError::SerdeJson(err) => fluent_message!("serde-json-error", err),
            AccessError::Vaild(e) => e.to_fluent_message(),
        }
    }
}

pub type AccessResult<T> = Result<T, AccessError>;
impl From<sqlx::Error> for AccessError {
    fn from(err: sqlx::Error) -> Self {
        AccessError::Sqlx(err)
    }
}
impl From<ValidError> for AccessError {
    fn from(err: ValidError) -> Self {
        AccessError::Vaild(err)
    }
}
impl From<SystemTimeError> for AccessError {
    fn from(err: SystemTimeError) -> Self {
        AccessError::System(fluent_message!("time-error", err))
    }
}
impl From<RedisError> for AccessError {
    fn from(err: RedisError) -> Self {
        AccessError::Redis(err)
    }
}
impl From<PoolError> for AccessError {
    fn from(err: PoolError) -> Self {
        AccessError::RedisPool(err)
    }
}
impl From<serde_json::Error> for AccessError {
    fn from(err: serde_json::Error) -> Self {
        AccessError::SerdeJson(err)
    }
}
