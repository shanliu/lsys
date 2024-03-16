//公共结构定义

use lsys_core::{fluent_message, FluentMessage, IntoFluentMessage};

#[derive(Debug)]
pub enum BarCodeError {
    System(FluentMessage),
    Sqlx(sqlx::Error),
}

pub type BarCodeResult<T> = Result<T, BarCodeError>;

impl IntoFluentMessage for BarCodeError {
    fn to_fluent_message(&self) -> FluentMessage {
        match self {
            BarCodeError::System(err) => err.to_owned(),
            BarCodeError::Sqlx(e) => fluent_message!("sqlx-error", e),
        }
    }
}

impl From<sqlx::Error> for BarCodeError {
    fn from(err: sqlx::Error) -> Self {
        BarCodeError::Sqlx(err)
    }
}
