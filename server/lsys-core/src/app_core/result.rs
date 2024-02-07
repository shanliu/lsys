// use config::Config;
use deadpool_redis::CreatePoolError;

use redis::RedisError;

use std::env::VarError;
// use std::error::Error;
// use std::fmt::{Display, Formatter};

use crate::{fluent_message, AppCore, FluentBundleError, FluentMessage, RemoteNotifyError};
use crate::{ConfigError, IntoFluentMessage};
#[derive(Debug)]
pub enum AppCoreError {
    Sqlx(sqlx::Error),
    Env(VarError),
    Tera(tera::Error),
    Io(std::io::Error),
    System(String),
    Log(String),
    Redis(RedisError),
    RedisPool(CreatePoolError),
    Dotenv(dotenv::Error),
    AppDir(String),
    Config(ConfigError),
    Fluent(FluentBundleError),
    RemoteNotify(RemoteNotifyError),
}

impl IntoFluentMessage for AppCoreError {
    fn to_fluent_message(&self) -> FluentMessage {
        match self {
            AppCoreError::Sqlx(err) => fluent_message!("sqlx-error", err),
            AppCoreError::Env(err) => fluent_message!("env-error", err),
            AppCoreError::Tera(err) => fluent_message!("tera-error", err),
            AppCoreError::Io(err) => fluent_message!("file-error", err),
            AppCoreError::System(err) => fluent_message!("app-error", err),
            AppCoreError::Log(err) => fluent_message!("log-error", err),
            AppCoreError::Redis(err) => fluent_message!("redis-error", err),
            AppCoreError::RedisPool(err) => fluent_message!("redis-error", err),
            AppCoreError::Dotenv(err) => fluent_message!("env-error", err),
            AppCoreError::AppDir(err) => fluent_message!("file-error", err),
            AppCoreError::Config(err) => err.to_fluent_message(),
            AppCoreError::RemoteNotify(err) => err.to_fluent_message(),
            AppCoreError::Fluent(err) => err.to_fluent_message(),
        }
    }
}

// impl Display for AppCoreError {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{:?}", self)
//     }
// }
// impl Error for AppCoreError {}
impl From<sqlx::Error> for AppCoreError {
    fn from(err: sqlx::Error) -> Self {
        AppCoreError::Sqlx(err)
    }
}
impl From<CreatePoolError> for AppCoreError {
    fn from(err: CreatePoolError) -> Self {
        AppCoreError::RedisPool(err)
    }
}
impl From<RemoteNotifyError> for AppCoreError {
    fn from(err: RemoteNotifyError) -> Self {
        AppCoreError::RemoteNotify(err)
    }
}

impl From<VarError> for AppCoreError {
    fn from(err: VarError) -> Self {
        AppCoreError::Env(err)
    }
}
impl From<tera::Error> for AppCoreError {
    fn from(err: tera::Error) -> Self {
        AppCoreError::Tera(err)
    }
}
impl From<std::io::Error> for AppCoreError {
    fn from(err: std::io::Error) -> Self {
        AppCoreError::Io(err)
    }
}
impl From<RedisError> for AppCoreError {
    fn from(err: RedisError) -> Self {
        AppCoreError::Redis(err)
    }
}
impl From<dotenv::Error> for AppCoreError {
    fn from(err: dotenv::Error) -> Self {
        AppCoreError::Dotenv(err)
    }
}
// impl From<core::convert::Infallible> for AppCoreError {
//     fn from(err: core::convert::Infallible) -> Self {
//         AppCoreError::AppDir(err.to_string())
//     }
// }
impl From<ConfigError> for AppCoreError {
    fn from(err: ConfigError) -> Self {
        AppCoreError::Config(err)
    }
}
impl From<config::ConfigError> for AppCoreError {
    fn from(err: config::ConfigError) -> Self {
        AppCoreError::Config(ConfigError::Config(err))
    }
}
impl From<FluentBundleError> for AppCoreError {
    fn from(err: FluentBundleError) -> Self {
        AppCoreError::Fluent(err)
    }
}

pub type AppCoreResult = Result<AppCore, AppCoreError>;
