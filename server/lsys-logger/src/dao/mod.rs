use std::{
    error::Error,
    fmt::{Display, Formatter},
};
mod change_log;
pub use change_log::*;

#[derive(Debug)]
pub enum LoggerError {
    Sqlx(sqlx::Error),
    System(String),
}
impl Display for LoggerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for LoggerError {}

impl From<sqlx::Error> for LoggerError {
    fn from(err: sqlx::Error) -> Self {
        LoggerError::Sqlx(err)
    }
}

pub type LoggerResult<T> = Result<T, LoggerError>;
