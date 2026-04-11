use lsys_core::{fluent_message, fluents::FluentMessage};

pub type FileManagerResult<T> = Result<T, FileManagerError>;

#[derive(Debug)]
pub enum FileManagerError {
    Database(sqlx::Error),
    Io(std::io::Error),
    Redis(deadpool_redis::PoolError),
    RedisCmd(redis::RedisError),
    Message(FluentMessage),
    File(lsys_file::dao::FileError),
    ValidError(lsys_core::valid_param::ValidError),
}

// 实现 IntoFluentMessage trait，使错误可以转换为 FluentMessage
impl lsys_core::fluents::IntoFluentMessage for FileManagerError {
    fn to_fluent_message(&self) -> FluentMessage {
        match self {
            FileManagerError::Database(e) => fluent_message!("sqlx-error", e),
            FileManagerError::Io(e) => fluent_message!("io-error", e),
            FileManagerError::Redis(e) => fluent_message!("redis-error", e),
            FileManagerError::RedisCmd(e) => fluent_message!("redis-error", e),
            FileManagerError::Message(fluent_message) => fluent_message.to_owned(),
            FileManagerError::File(file_error) => file_error.to_fluent_message(),
            FileManagerError::ValidError(valid_error) => valid_error.to_fluent_message(),
        }
    }
}

impl From<lsys_file::dao::FileError> for FileManagerError {
    fn from(e: lsys_file::dao::FileError) -> Self {
        Self::File(e)
    }
}

impl From<sqlx::Error> for FileManagerError {
    fn from(e: sqlx::Error) -> Self {
        Self::Database(e)
    }
}

impl From<std::io::Error> for FileManagerError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<deadpool_redis::PoolError> for FileManagerError {
    fn from(e: deadpool_redis::PoolError) -> Self {
        Self::Redis(e)
    }
}

impl From<redis::RedisError> for FileManagerError {
    fn from(e: redis::RedisError) -> Self {
        Self::RedisCmd(e)
    }
}

impl From<lsys_core::valid_param::ValidError> for FileManagerError {
    fn from(e: lsys_core::valid_param::ValidError) -> Self {
        Self::ValidError(e)
    }
}
