//公共结构定义

use image::ImageError;
use lsys_core::{fluent_message, FluentMessage, IntoFluentMessage, ValidError};
use rxing::Exceptions;

#[derive(Debug)]
pub enum BarCodeError {
    System(FluentMessage),
    Image(ImageError),
    DB(sqlx::Error),
    RXing(Exceptions),
    Io(std::io::Error),
    Vaild(ValidError),
}

pub type BarCodeResult<T> = Result<T, BarCodeError>;

impl IntoFluentMessage for BarCodeError {
    fn to_fluent_message(&self) -> FluentMessage {
        match self {
            BarCodeError::System(err) => err.to_owned(),
            BarCodeError::DB(e) => fluent_message!("sqlx-error", e),
            BarCodeError::RXing(e) => fluent_message!("rxing-error", e),
            BarCodeError::Io(e) => fluent_message!("io-error", e),
            BarCodeError::Image(e) => fluent_message!("barcode-image-error", e),
            BarCodeError::Vaild(e) => e.to_fluent_message(),
        }
    }
}
impl From<ValidError> for BarCodeError {
    fn from(err: ValidError) -> Self {
        BarCodeError::Vaild(err)
    }
}
impl From<sqlx::Error> for BarCodeError {
    fn from(err: sqlx::Error) -> Self {
        BarCodeError::DB(err)
    }
}
impl From<Exceptions> for BarCodeError {
    fn from(err: Exceptions) -> Self {
        BarCodeError::RXing(err)
    }
}
impl From<std::io::Error> for BarCodeError {
    fn from(err: std::io::Error) -> Self {
        BarCodeError::Io(err)
    }
}
impl From<std::time::SystemTimeError> for BarCodeError {
    fn from(err: std::time::SystemTimeError) -> Self {
        BarCodeError::System(fluent_message!("time-error", err))
    }
}
impl From<image::ImageError> for BarCodeError {
    fn from(err: image::ImageError) -> Self {
        BarCodeError::Image(err)
    }
}
