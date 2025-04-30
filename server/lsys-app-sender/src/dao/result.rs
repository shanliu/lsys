use std::{
    // error::Error,
    fmt::{Display, Formatter},
};

use deadpool_redis::PoolError;
use lsys_core::{fluent_message, FluentMessage, IntoFluentMessage, ValidError};
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
    Vaild(ValidError),
}

impl IntoFluentMessage for SenderError {
    fn to_fluent_message(&self) -> FluentMessage {
        match self {
            SenderError::Sqlx(err) => fluent_message!("sqlx-error", err),
            SenderError::Redis(err) => fluent_message!("redis-error", err),
            SenderError::RedisPool(err) => fluent_message!("redis-error", err),
            SenderError::Tera(err) => fluent_message!("tera-error", err),
            SenderError::System(err) => err.to_owned(),
            SenderError::Setting(err) => err.to_fluent_message(),
            SenderError::Vaild(err) => err.to_fluent_message(),
        }
    }
}

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

impl From<ValidError> for SenderError {
    fn from(err: ValidError) -> Self {
        SenderError::Vaild(err)
    }
}

pub type SenderResult<T> = Result<T, SenderError>;

#[derive(Debug)]
pub enum SenderExecError {
    Finish(String),
    Next(String),
}

impl Display for SenderExecError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Finish(e) => write!(f, "{}", e),
            Self::Next(e) => write!(f, "{}", e),
        }
    }
}

pub enum SenderTaskStatus {
    Progress,
    Completed,
    Failed(bool), //失败是否重试
}

pub struct SenderTaskResultItem {
    pub id: u64, // SenderTaskData item id
    pub status: SenderTaskStatus,
    pub message: String,
    pub send_id: String,
}

pub type SenderTaskResult = Result<Vec<SenderTaskResultItem>, SenderExecError>;
