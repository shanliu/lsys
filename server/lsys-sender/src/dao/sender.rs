use std::{
    error::Error,
    fmt::{Display, Formatter},
};

use deadpool_redis::PoolError;
use lsys_setting::dao::SettingError;

//公共结构定义
#[derive(Debug)]
pub enum SenderError {
    Sqlx(sqlx::Error),
    Redis(String),
    Tpl(tera::Error),
    System(String),
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
        SenderError::Redis(err.to_string())
    }
}
impl From<PoolError> for SenderError {
    fn from(err: PoolError) -> Self {
        SenderError::Redis(err.to_string())
    }
}
impl From<SettingError> for SenderError {
    fn from(err: SettingError) -> Self {
        match err {
            SettingError::Sqlx(e) => SenderError::Sqlx(e),
            SettingError::System(e) => SenderError::System(e),
        }
    }
}
impl From<tera::Error> for SenderError {
    fn from(err: tera::Error) -> Self {
        SenderError::Tpl(err)
    }
}

pub type SenderResult<T> = Result<T, SenderError>;
