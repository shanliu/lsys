use std::{
    error::Error,
    fmt::{Display, Formatter},
};

use deadpool_redis::PoolError;
use lsys_core::FluentMessage;
use lsys_setting::dao::SettingError;

//公共结构定义
#[derive(Debug)]
pub enum SenderError {
    Sqlx(sqlx::Error),
    Redis(redis::RedisError),
    RedisPool(PoolError),
    Tera(tera::Error),
    System(FluentMessage),
    Setting(SettingError),
}
impl Display for SenderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for SenderError {}

impl From<sqlx::Error> for SenderError {
    fn from(err: sqlx::Error) -> Self {
        SenderError::Sqlx(err)
    }
}
impl From<redis::RedisError> for SenderError {
    fn from(err: redis::RedisError) -> Self {
        SenderError::Redis(err)
    }
}
impl From<PoolError> for SenderError {
    fn from(err: PoolError) -> Self {
        SenderError::RedisPool(err)
    }
}
impl From<SettingError> for SenderError {
    fn from(err: SettingError) -> Self {
        SenderError::Setting(err)
    }
}
impl From<tera::Error> for SenderError {
    fn from(err: tera::Error) -> Self {
        SenderError::Tera(err)
    }
}

pub type SenderResult<T> = Result<T, SenderError>;
