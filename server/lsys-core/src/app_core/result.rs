#[cfg(feature = "redis")]
use deadpool_redis::{CreatePoolError, PoolError};
#[cfg(feature = "redis")]
use redis::RedisError;

use std::env::VarError;

#[cfg(feature = "redis")]
use crate::remote_notify::RemoteNotifyError;
use crate::fluent_message;
use crate::config::ConfigError;
use crate::fluents::{FluentBundleError, FluentMessage, IntoFluentMessage};
#[derive(Debug)]
pub enum AppCoreError {
    #[cfg(feature = "db")]
    Sqlx(sqlx::Error),
    Env(VarError),
    #[cfg(feature = "tera")]
    Tera(tera::Error),
    Io(std::io::Error),
    System(String),
    Log(String),
    #[cfg(feature = "redis")]
    Redis(RedisError),
    #[cfg(feature = "redis")]
    RedisCreatePool(CreatePoolError),
    #[cfg(feature = "redis")]
    RedisPool(PoolError),
    Dotenv(dotenv::Error),
    AppDir(String),
    Config(ConfigError),
    Fluent(FluentBundleError),
    #[cfg(feature = "redis")]
    RemoteNotify(RemoteNotifyError),
}

impl IntoFluentMessage for AppCoreError {
    fn to_fluent_message(&self) -> FluentMessage {
        match self {
            #[cfg(feature = "db")]
            AppCoreError::Sqlx(err) => fluent_message!("sqlx-error", err),
            AppCoreError::Env(err) => fluent_message!("env-error", err),
            #[cfg(feature = "tera")]
            AppCoreError::Tera(err) => fluent_message!("tera-error", err),
            AppCoreError::Io(err) => fluent_message!("file-error", err),
            AppCoreError::System(err) => fluent_message!("system-error", err),
            AppCoreError::Log(err) => fluent_message!("log-error", err),
            #[cfg(feature = "redis")]
            AppCoreError::Redis(err) => fluent_message!("redis-error", err),
            #[cfg(feature = "redis")]
            AppCoreError::RedisPool(err) => fluent_message!("redis-error", err),
            #[cfg(feature = "redis")]
            AppCoreError::RedisCreatePool(err) => fluent_message!("redis-error", err),
            AppCoreError::Dotenv(err) => fluent_message!("env-error", err),
            AppCoreError::AppDir(err) => fluent_message!("file-error", err),
            AppCoreError::Config(err) => err.to_fluent_message(),
            #[cfg(feature = "redis")]
            AppCoreError::RemoteNotify(err) => err.to_fluent_message(),
            AppCoreError::Fluent(err) => err.to_fluent_message(),
        }
    }
}
#[cfg(feature = "db")]
impl From<sqlx::Error> for AppCoreError {
    fn from(err: sqlx::Error) -> Self {
        AppCoreError::Sqlx(err)
    }
}
#[cfg(feature = "redis")]
impl From<PoolError> for AppCoreError {
    fn from(err: PoolError) -> Self {
        AppCoreError::RedisPool(err)
    }
}
#[cfg(feature = "redis")]
impl From<CreatePoolError> for AppCoreError {
    fn from(err: CreatePoolError) -> Self {
        AppCoreError::RedisCreatePool(err)
    }
}
#[cfg(feature = "redis")]
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
#[cfg(feature = "tera")]
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
#[cfg(feature = "redis")]
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
