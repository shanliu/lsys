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

pub type WaitNotifyResult = Result<bool, String>; //bool 是否成功完成,String 错误消息
