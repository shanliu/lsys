use lsys_core::{fluent_message, FluentMessage, IntoFluentMessage, ValidError};
#[derive(Debug)]
pub enum SettingError {
    Sqlx(sqlx::Error),
    SerdeJson(serde_json::Error),
    Vaild(ValidError),
}

impl IntoFluentMessage for SettingError {
    fn to_fluent_message(&self) -> FluentMessage {
        match self {
            Self::Sqlx(err) => fluent_message!("sqlx-error", err),
            Self::SerdeJson(err) => fluent_message!("serde-json-error", err),
            Self::Vaild(e) => e.to_fluent_message(),
        }
    }
}

impl From<sqlx::Error> for SettingError {
    fn from(err: sqlx::Error) -> Self {
        SettingError::Sqlx(err)
    }
}
impl From<ValidError> for SettingError {
    fn from(err: ValidError) -> Self {
        SettingError::Vaild(err)
    }
}
pub type SettingResult<T> = Result<T, SettingError>;

pub trait NotFoundResult {
    fn notfound_default(self) -> Self;
}

impl<T: Default> NotFoundResult for SettingResult<T> {
    fn notfound_default(self) -> Self {
        match self {
            Ok(s) => Ok(s),
            Err(SettingError::Sqlx(sqlx::Error::RowNotFound)) => Ok(T::default()),
            Err(e) => Err(e),
        }
    }
}
