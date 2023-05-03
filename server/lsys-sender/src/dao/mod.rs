mod app_config_reader;
mod logger;
mod mailer;
mod message_cancel;
mod message_logs;
mod message_reader;
mod message_tpls;
mod sender_config;
mod smser;
mod task_sender;
use std::{
    error::Error,
    fmt::{Display, Formatter},
};

pub use app_config_reader::*;
use deadpool_redis::PoolError;

use lsys_setting::dao::SettingError;
pub use mailer::*;
pub use message_cancel::*;
pub use message_logs::*;
pub use message_reader::*;
pub use message_tpls::*;
pub use sender_config::*;
pub use smser::*;
pub use task_sender::*;

#[derive(Debug)]
pub enum SenderError {
    Sqlx(sqlx::Error),
    Redis(String),
    Tpl(tera::Error),
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
impl From<tera::Error> for SenderError {
    fn from(err: tera::Error) -> Self {
        SenderError::Tpl(err)
    }
}

pub type SenderResult<T> = Result<T, SenderError>;
