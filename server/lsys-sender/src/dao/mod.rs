mod mailer;
mod smser;
mod task;
use std::{
    error::Error,
    fmt::{Display, Formatter},
};

use deadpool_redis::PoolError;
use lsys_setting::dao::SettingError;
pub use mailer::*;
pub use smser::*;

#[derive(Debug)]
pub enum SenderError {
    Sqlx(sqlx::Error),
    Redis(String),
    Exec(String),
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

pub type SenderResult<T> = Result<T, SenderError>;
