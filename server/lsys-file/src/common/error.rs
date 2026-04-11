use lsys_core::fluent_message;
use lsys_core::fluents::{FluentMessage, IntoFluentMessage};

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
    /// 等待下载完成超时
    DownloadTimeout(u64, u64),
    /// 下载失败
    DownloadFailed(u64, String),
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
            FileError::DownloadTimeout(timeout, file_user_id) => write!(
                f,
                "Download timeout after {}s, file_user_id={}",
                timeout, file_user_id
            ),
            FileError::DownloadFailed(file_user_id, msg) => write!(
                f,
                "Download failed, file_user_id={}, msg={}",
                file_user_id, msg
            ),
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
            FileError::DownloadTimeout(timeout, file_user_id) => {
                fluent_message!(
                    "file-download-timeout",
                    &format!(
                        "Download timeout after {}s, file_user_id={}",
                        timeout, file_user_id
                    )
                )
            }
            FileError::DownloadFailed(file_user_id, msg) => {
                fluent_message!(
                    "file-download-failed",
                    &format!(
                        "Download failed, file_user_id={}, msg={}",
                        file_user_id, msg
                    )
                )
            }
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

impl From<lsys_setting::dao::SettingError> for FileError {
    fn from(err: lsys_setting::dao::SettingError) -> Self {
        FileError::System(err.to_fluent_message())
    }
}

impl From<lsys_core::valid_param::ValidError> for FileError {
    fn from(err: lsys_core::valid_param::ValidError) -> Self {
        FileError::Param(err.to_fluent_message())
    }
}

pub type FileResult<T> = Result<T, FileError>;
