use lsys_access::dao::AccessError;
use lsys_core::{fluent_message, AppCoreError, FluentMessage, IntoFluentMessage, ValidError};
use std::{
    // error::Error,
    // fmt::{Display, Formatter},
    time::SystemTimeError,
};

use deadpool_redis::PoolError;

use redis::RedisError;

#[derive(Debug)]
pub enum AppError {
    AppNotFound(String),
    AppBadStatus,
    AppBadFeature(String, Vec<String>),
    AppOAuthClientBadConfig(String),
    AppOAuthClientBadDomain(String),
    Sqlx(sqlx::Error),
    System(FluentMessage),
    Redis(RedisError),
    RedisPool(PoolError),
    ScopeBad(Vec<String>),
    Access(AccessError),
    SerdeJson(serde_json::Error),
    AppCore(AppCoreError),
    Vaild(ValidError),
}
impl IntoFluentMessage for AppError {
    fn to_fluent_message(&self) -> FluentMessage {
        match self {
            AppError::System(e) => e.to_owned(),
            AppError::ScopeBad(data) => fluent_message!("app-bad-scope",{"scope":data.join(",")}),
            AppError::Sqlx(e) => {
                fluent_message!("sqlx-error", e)
            }
            AppError::Redis(e) => {
                fluent_message!("redis-error", e)
            }
            AppError::RedisPool(e) => {
                fluent_message!("redis-error", e)
            }
            AppError::Access(e) => e.to_fluent_message(),
            AppError::AppCore(e) => e.to_fluent_message(),
            AppError::SerdeJson(e) => {
                fluent_message!("serde-json-error", e)
            }
            AppError::AppNotFound(name) => {
                fluent_message!("app-not-found",{
                    "name":name
                })
            }
            AppError::AppBadStatus => {
                fluent_message!("app-bad-status")
            }
            AppError::AppBadFeature(name, feature) => {
                fluent_message!("app-feature-not-support",{
                    "name":name,
                    "feature":feature.join(","),
                })
            }
            AppError::AppOAuthClientBadConfig(name) => {
                fluent_message!("app-config-bad",{
                    "name":name
                })
            }
            AppError::AppOAuthClientBadDomain(name) => {
                fluent_message!("app-config-bad",{
                    "name":name
                })
            }
            AppError::Vaild(e) => e.to_fluent_message(),
        }
    }
}
impl From<ValidError> for AppError {
    fn from(err: ValidError) -> Self {
        AppError::Vaild(err)
    }
}
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Sqlx(err)
    }
}
impl From<RedisError> for AppError {
    fn from(err: RedisError) -> Self {
        AppError::Redis(err)
    }
}
impl From<PoolError> for AppError {
    fn from(err: PoolError) -> Self {
        AppError::RedisPool(err)
    }
}
impl From<SystemTimeError> for AppError {
    fn from(err: SystemTimeError) -> Self {
        AppError::System(fluent_message!("time-error", err))
    }
}
impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::SerdeJson(err)
    }
}
impl From<AccessError> for AppError {
    fn from(err: AccessError) -> Self {
        AppError::Access(err)
    }
}
impl From<AppCoreError> for AppError {
    fn from(err: AppCoreError) -> Self {
        AppError::AppCore(err)
    }
}

pub type AppResult<T> = Result<T, AppError>;
