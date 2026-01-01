use lsys_web::lsys_core::AppCoreError;

use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum AppError {
    AppCore(AppCoreError),
    Rustls(rustls::Error),
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for AppError {}
impl From<AppCoreError> for AppError {
    fn from(err: AppCoreError) -> Self {
        AppError::AppCore(err)
    }
}
impl From<rustls::Error> for AppError {
    fn from(err: rustls::Error) -> Self {
        AppError::Rustls(err)
    }
}
