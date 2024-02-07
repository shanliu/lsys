// use std::error::Error;
// use std::fmt::{Display, Formatter};

use deadpool_redis::PoolError;

use redis::RedisError;

use crate::{fluent_message, FluentMessage, IntoFluentMessage};

#[derive(Debug)]
pub enum RemoteNotifyError {
    System(String),
    RedisPool(PoolError),
    Redis(RedisError),
    RemoteTimeOut,
}

impl IntoFluentMessage for RemoteNotifyError {
    fn to_fluent_message(&self) -> FluentMessage {
        match self {
            RemoteNotifyError::System(err) => fluent_message!("notify-error", err),
            RemoteNotifyError::RedisPool(err) => fluent_message!("redis-error", err),
            RemoteNotifyError::Redis(err) => fluent_message!("redis-error", err),
            RemoteNotifyError::RemoteTimeOut => fluent_message!("notify-time-out"),
        }
    }
}

// impl Display for RemoteNotifyError {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{:?}", self)
//     }
// }
// impl Error for RemoteNotifyError {}
impl From<RedisError> for RemoteNotifyError {
    fn from(err: RedisError) -> Self {
        RemoteNotifyError::Redis(err)
    }
}
impl From<PoolError> for RemoteNotifyError {
    fn from(err: PoolError) -> Self {
        RemoteNotifyError::RedisPool(err)
    }
}
