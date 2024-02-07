use crate::{fluent_message, FluentMessage, IntoFluentMessage};
use deadpool_redis::PoolError;
use std::fmt::Debug;

#[derive(Debug)]
pub enum WaitNotifyError {
    System(FluentMessage),
    Redis(redis::RedisError),
    RedisPool(PoolError),
    TimeOut,
}

impl IntoFluentMessage for WaitNotifyError {
    fn to_fluent_message(&self) -> FluentMessage {
        match self {
            WaitNotifyError::System(err) => err.to_owned(),
            WaitNotifyError::Redis(err) => fluent_message!("redis-error", err),
            WaitNotifyError::RedisPool(err) => fluent_message!("redis-error", err),
            WaitNotifyError::TimeOut => fluent_message!("wait-notify-timeout"),
        }
    }
}

// impl Display for WaitNotifyError {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{:?}", self)
//     }
// }

pub type WaitNotifyResult = Result<bool, String>;
