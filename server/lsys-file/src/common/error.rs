use lsys_core::dist_lock::DistLockError;
use lsys_core::fluent_message;
use lsys_core::fluents::{FluentMessage, IntoFluentMessage};

/// 文件错误类型
#[derive(Debug)]
pub enum FileError {
    Sqlx(sqlx::Error),
    Io(std::io::Error),
    Redis(redis::RedisError),
    RedisPool(deadpool_redis::PoolError),
    AppCore(lsys_core::app_core::AppCoreError),
    Setting(lsys_setting::dao::SettingError),
    Valid(lsys_core::valid_param::ValidError),
    System(FluentMessage),
    Lock(DistLockError),
    Param(FluentMessage),
    Http(String),
    InvalidStatusCode(u16),
    RedirectLimitExceeded,
    InvalidChunkData(String),
    /// 等待下载完成超时
    DownloadTimeout(u64, u64),
    /// 下载失败
    DownloadFailed(u64, String),
    /// 无效的文件加密标识符
    InvalidFileKey(String),
    /// 文件加密标识符已过期
    FileKeyExpired(String),
}

impl std::fmt::Display for FileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_fluent_message().default_format())
    }
}

impl IntoFluentMessage for FileError {
    fn to_fluent_message(&self) -> FluentMessage {
        match self {
            FileError::Sqlx(e) => fluent_message!("sqlx-error", e),
            FileError::Io(e) => fluent_message!("file-io-error", e),
            FileError::Redis(e) => fluent_message!("redis-error", e),
            FileError::RedisPool(e) => fluent_message!("redis-error", e),
            FileError::AppCore(e) => e.to_fluent_message(),
            FileError::Setting(e) => e.to_fluent_message(),
            FileError::Valid(e) => e.to_fluent_message(),
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
            FileError::DownloadTimeout(timeout, file_ref_user_id) => {
                fluent_message!(
                    "file-download-timeout",
                    &format!(
                        "Download timeout after {}s, file_ref_user_id={}",
                        timeout, file_ref_user_id
                    )
                )
            }
            FileError::DownloadFailed(file_ref_user_id, msg) => {
                fluent_message!(
                    "file-download-failed",
                    &format!(
                        "Download failed, file_ref_user_id={}, msg={}",
                        file_ref_user_id, msg
                    )
                )
            }
            FileError::Lock(dist_lock_error) => dist_lock_error.to_fluent_message(),
            FileError::InvalidFileKey(key) => {
                fluent_message!("file-error", &format!("Invalid file key: {}", key))
            }
            FileError::FileKeyExpired(key) => {
                fluent_message!("file-error", &format!("File key expired: {}", key))
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
        FileError::Setting(err)
    }
}

impl From<lsys_core::valid_param::ValidError> for FileError {
    fn from(err: lsys_core::valid_param::ValidError) -> Self {
        FileError::Valid(err)
    }
}

impl From<redis::RedisError> for FileError {
    fn from(err: redis::RedisError) -> Self {
        FileError::Redis(err)
    }
}

impl From<deadpool_redis::PoolError> for FileError {
    fn from(err: deadpool_redis::PoolError) -> Self {
        FileError::RedisPool(err)
    }
}

impl From<lsys_core::app_core::AppCoreError> for FileError {
    fn from(err: lsys_core::app_core::AppCoreError) -> Self {
        FileError::AppCore(err)
    }
}

impl From<DistLockError> for FileError {
    fn from(err: DistLockError) -> Self {
        FileError::Lock(err)
    }
}

pub type FileResult<T> = Result<T, FileError>;
