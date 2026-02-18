use lsys_core::{fluent_message, FluentMessage, IntoFluentMessage};

/// 文件错误类型
#[derive(Debug)]
pub enum FileError {
    Sqlx(sqlx::Error),
    Io(std::io::Error),
    System(FluentMessage),
    Param(FluentMessage),
    Http(String),
    InvalidStatusCode(u16),
    RedirectLimitExceeded,
    InvalidChunkData(String),
}

impl std::fmt::Display for FileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileError::Sqlx(e) => write!(f, "sqlx error: {}", e),
            FileError::Io(e) => write!(f, "io error: {}", e),
            FileError::System(e) => write!(f, "system error: {:?}", e),
            FileError::Param(e) => write!(f, "param error: {:?}", e),
            FileError::Http(e) => write!(f, "HTTP error: {}", e),
            FileError::InvalidStatusCode(code) => write!(f, "Invalid status code: {}", code),
            FileError::RedirectLimitExceeded => write!(f, "Redirect limit exceeded"),
            FileError::InvalidChunkData(e) => write!(f, "Invalid chunk data: {}", e),
        }
    }
}

impl IntoFluentMessage for FileError {
    fn to_fluent_message(&self) -> FluentMessage {
        match self {
            FileError::Sqlx(e) => fluent_message!("sqlx-error", e),
            FileError::Io(e) => fluent_message!("file-io-error", e),
            FileError::System(e) => e.clone(),
            FileError::Param(e) => e.clone(),
            FileError::Http(e) => fluent_message!("file-error", e),
            FileError::InvalidStatusCode(code) => {
                fluent_message!("file-error", &format!("Invalid status code: {}", code))
            }
            FileError::RedirectLimitExceeded => {
                fluent_message!("file-error", "Redirect limit exceeded")
            }
            FileError::InvalidChunkData(e) => fluent_message!("file-error", e),
        }
    }
}

impl From<sqlx::Error> for FileError {
    fn from(err: sqlx::Error) -> Self {
        FileError::Sqlx(err)
    }
}

impl From<std::io::Error> for FileError {
    fn from(err: std::io::Error) -> Self {
        FileError::Io(err)
    }
}

impl From<std::time::SystemTimeError> for FileError {
    fn from(err: std::time::SystemTimeError) -> Self {
        FileError::System(fluent_message!("time-error", err))
    }
}

pub type FileResult<T> = Result<T, FileError>;
