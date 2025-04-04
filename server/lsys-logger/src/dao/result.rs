// use std::{
//     // error::Error,
//     fmt::{Display, Formatter},
// };

use lsys_core::{fluent_message, FluentMessage, IntoFluentMessage};
#[derive(Debug)]
pub enum LoggerError {
    Sqlx(sqlx::Error),
    // System(FluentMessage),
}

impl IntoFluentMessage for LoggerError {
    fn to_fluent_message(&self) -> FluentMessage {
        match self {
            LoggerError::Sqlx(e) => fluent_message!("sqlx-error", e),
        }
    }
}

impl From<sqlx::Error> for LoggerError {
    fn from(err: sqlx::Error) -> Self {
        LoggerError::Sqlx(err)
    }
}

pub type LoggerResult<T> = Result<T, LoggerError>;
