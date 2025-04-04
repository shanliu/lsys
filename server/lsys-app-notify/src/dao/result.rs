//公共结构定义

use deadpool_redis::PoolError;

use lsys_core::{fluent_message, FluentMessage, IntoFluentMessage};

#[derive(Debug)]
pub enum NotifyError {
    Sqlx(sqlx::Error),
    Redis(redis::RedisError),
    RedisPool(PoolError),
    System(FluentMessage),
}

impl IntoFluentMessage for NotifyError {
    fn to_fluent_message(&self) -> FluentMessage {
        match self {
            NotifyError::Sqlx(e) => fluent_message!("sqlx-error", e),
            NotifyError::Redis(err) => fluent_message!("redis-error", err),
            NotifyError::RedisPool(err) => fluent_message!("redis-error", err),
            NotifyError::System(err) => err.to_owned(),
        }
    }
}

impl From<sqlx::Error> for NotifyError {
    fn from(err: sqlx::Error) -> Self {
        NotifyError::Sqlx(err)
    }
}
impl From<redis::RedisError> for NotifyError {
    fn from(err: redis::RedisError) -> Self {
        NotifyError::Redis(err)
    }
}
impl From<PoolError> for NotifyError {
    fn from(err: PoolError) -> Self {
        NotifyError::RedisPool(err)
    }
}

pub type NotifyResult<T> = Result<T, NotifyError>;
