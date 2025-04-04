use std::{
    // fmt::{Display, Formatter},
    string::FromUtf8Error,
};

use deadpool_redis::PoolError;
use redis::RedisError;

use crate::{fluent_message, FluentMessage, IntoFluentMessage};
#[derive(Debug)]
//不匹配错误
pub struct ValidCodeCheckError {
    pub message: FluentMessage,
    pub prefix: String,
}
#[derive(Debug)]
pub enum ValidCodeError {
    Utf8Err(FromUtf8Error),
    Redis(RedisError),
    RedisPool(PoolError),
    Tag(FluentMessage),
    DelayTimeout(ValidCodeCheckError),
    NotMatch(ValidCodeCheckError),
}

impl IntoFluentMessage for ValidCodeError {
    fn to_fluent_message(&self) -> FluentMessage {
        match self {
            ValidCodeError::Utf8Err(err) => fluent_message!("utf-parse-error", err),
            ValidCodeError::Redis(err) => fluent_message!("redis-error", err),
            ValidCodeError::RedisPool(err) => fluent_message!("redis-error", err),
            ValidCodeError::Tag(err) => err.to_owned(),
            ValidCodeError::DelayTimeout(err) => err.message.clone(),
            ValidCodeError::NotMatch(err) => err.message.clone(),
        }
    }
}

impl From<FromUtf8Error> for ValidCodeError {
    fn from(err: FromUtf8Error) -> Self {
        ValidCodeError::Utf8Err(err)
    }
}
impl From<RedisError> for ValidCodeError {
    fn from(err: RedisError) -> Self {
        ValidCodeError::Redis(err)
    }
}
impl From<PoolError> for ValidCodeError {
    fn from(err: PoolError) -> Self {
        ValidCodeError::RedisPool(err)
    }
}
