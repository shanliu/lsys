// 定制错误的输出消息
// 多语言格式化错误信息输出实现

use lsys_access::dao::AccessError;
use lsys_app::dao::AppError;
use lsys_app_sender::dao::SenderError;
use lsys_core::AppCoreError;
use lsys_core::FluentBundleError;
use lsys_core::RemoteNotifyError;
use lsys_core::ValidError;
use lsys_core::{fluent_message, ConfigError, FluentBundle, IntoFluentMessage, ValidCodeError};
use lsys_logger::dao::LoggerError;
use lsys_rbac::dao::RbacError;
use lsys_setting::dao::SettingError;
use lsys_user::dao::AccountError;
use lsys_user::dao::UserAuthError;
use std::num::ParseIntError;

#[cfg(feature = "docs")]
use lsys_docs::dao::GitDocError;

pub trait FluentFormat {
    fn fluent_format(&self, fluent: &FluentBundle) -> String;
}

//crate error
impl FluentFormat for sqlx::Error {
    fn fluent_format(&self, fluent: &FluentBundle) -> String {
        match self {
            sqlx::Error::RowNotFound => {
                fluent.format_message(&fluent_message!("db-not-found", self))
            }
            _ => fluent.format_message(&fluent_message!("db-error", self)),
        }
    }
}
macro_rules! crate_error_fluent_string {
    ($crate_error:ty,$key:literal) => {
        impl FluentFormat for $crate_error {
            fn fluent_format(&self, fluent: &FluentBundle) -> String {
                fluent.format_message(&fluent_message!($key, self))
            }
        }
    };
}

crate_error_fluent_string!(std::env::VarError, "env-var-error");
crate_error_fluent_string!(config::ConfigError, "config-error");
crate_error_fluent_string!(std::io::Error, "io-error");
crate_error_fluent_string!(tera::Error, "tera-error");
crate_error_fluent_string!(redis::RedisError, "redis-error");
crate_error_fluent_string!(deadpool_redis::PoolError, "redis-error");
crate_error_fluent_string!(deadpool_redis::CreatePoolError, "redis-error");
crate_error_fluent_string!(serde_json::Error, "serde-json-error");
crate_error_fluent_string!(ParseIntError, "parse-error");
crate_error_fluent_string!(std::string::FromUtf8Error, "utf8-parse-error");
crate_error_fluent_string!(dotenv::Error, "dotenv-error");

#[cfg(feature = "docs")]
crate_error_fluent_string!(git2::Error, "git-error");

impl FluentFormat for lsys_lib_area::AreaError {
    fn fluent_format(&self, fluent: &FluentBundle) -> String {
        match self {
            lsys_lib_area::AreaError::DB(err) => {
                fluent.format_message(&fluent_message!("lsys-lib-area-db", err))
            }
            lsys_lib_area::AreaError::System(err) => {
                fluent.format_message(&fluent_message!("lsys-lib-area-error", err))
            }
            lsys_lib_area::AreaError::NotFind(_) => {
                fluent.format_message(&fluent_message!("area-not-found"))
            }
            lsys_lib_area::AreaError::Store(err) => {
                fluent.format_message(&fluent_message!("area-store-error", err))
            }
            lsys_lib_area::AreaError::Tantivy(err) => {
                fluent.format_message(&fluent_message!("area-tantivy-error", err))
            }
        }
    }
}
// 内部错误
impl FluentFormat for FluentBundleError {
    fn fluent_format(&self, fluent: &FluentBundle) -> String {
        match self {
            FluentBundleError::Io(error) => error.fluent_format(fluent),
            _ => fluent.format_message(&self.to_fluent_message()),
        }
    }
}
impl FluentFormat for RemoteNotifyError {
    fn fluent_format(&self, fluent: &FluentBundle) -> String {
        match self {
            RemoteNotifyError::RedisPool(pool_error) => pool_error.fluent_format(fluent),
            RemoteNotifyError::Redis(redis_error) => redis_error.fluent_format(fluent),
            _ => fluent.format_message(&self.to_fluent_message()),
        }
    }
}

impl FluentFormat for AppCoreError {
    fn fluent_format(&self, fluent: &FluentBundle) -> String {
        match self {
            AppCoreError::Sqlx(error) => error.fluent_format(fluent),
            AppCoreError::Env(var_error) => var_error.fluent_format(fluent),
            AppCoreError::Tera(error) => error.fluent_format(fluent),
            AppCoreError::Io(error) => error.fluent_format(fluent),
            AppCoreError::Redis(redis_error) => redis_error.fluent_format(fluent),
            AppCoreError::RedisCreatePool(create_pool_error) => {
                create_pool_error.fluent_format(fluent)
            }
            AppCoreError::RedisPool(pool_error) => pool_error.fluent_format(fluent),
            AppCoreError::Dotenv(error) => error.fluent_format(fluent),
            AppCoreError::Config(config_error) => config_error.fluent_format(fluent),
            AppCoreError::Fluent(fluent_bundle_error) => fluent_bundle_error.fluent_format(fluent),
            AppCoreError::RemoteNotify(remote_notify_error) => {
                remote_notify_error.fluent_format(fluent)
            }
            _ => fluent.format_message(&self.to_fluent_message()),
        }
    }
}

impl FluentFormat for AccountError {
    fn fluent_format(&self, fluent: &FluentBundle) -> String {
        match self {
            AccountError::Sqlx(error) => error.fluent_format(fluent),
            AccountError::System(fluent_message) => fluent.format_message(fluent_message),
            AccountError::Status(e) => fluent.format_message(&e.1),
            AccountError::Redis(redis_error) => redis_error.fluent_format(fluent),
            AccountError::RedisPool(pool_error) => pool_error.fluent_format(fluent),
            AccountError::SerdeJson(error) => error.fluent_format(fluent),
            AccountError::ValidCode(valid_code_error) => valid_code_error.fluent_format(fluent),
            AccountError::Setting(setting_error) => setting_error.fluent_format(fluent),
            AccountError::Param(fluent_message) => fluent.format_message(fluent_message),
            AccountError::AuthStatusError(e) => fluent.format_message(&e.1),
            AccountError::UserAuthError(user_auth_error) => user_auth_error.fluent_format(fluent),
            AccountError::AccessError(access_error) => access_error.fluent_format(fluent),
            AccountError::PasswordNotMatch(e) => fluent.format_message(&e.1),
            AccountError::PasswordNotSet(e) => fluent.format_message(&e.1),
            AccountError::UserNotFind(fluent_message) => fluent.format_message(fluent_message),
            AccountError::Vaild(valid_error) => valid_error.fluent_format(fluent),
        }
    }
}

impl FluentFormat for ValidCodeError {
    fn fluent_format(&self, fluent: &FluentBundle) -> String {
        match self {
            ValidCodeError::Utf8Err(e) => e.fluent_format(fluent),
            ValidCodeError::Redis(redis_error) => redis_error.fluent_format(fluent),
            ValidCodeError::RedisPool(pool_error) => pool_error.fluent_format(fluent),
            ValidCodeError::Tag(fluent_message) => fluent.format_message(fluent_message),
            ValidCodeError::NotMatch(valid_code_check_error) => {
                fluent.format_message(&valid_code_check_error.message)
            }
            ValidCodeError::Valid(valid_error) => valid_error.fluent_format(fluent),
        }
    }
}

impl FluentFormat for AccessError {
    fn fluent_format(&self, fluent: &FluentBundle) -> String {
        match self {
            AccessError::Sqlx(error) => error.fluent_format(fluent),
            AccessError::Redis(redis_error) => redis_error.fluent_format(fluent),
            AccessError::RedisPool(pool_error) => pool_error.fluent_format(fluent),
            AccessError::System(fluent_message) => fluent.format_message(fluent_message),
            AccessError::SerdeJson(error) => error.fluent_format(fluent),
            AccessError::BadAccount(fluent_message) => fluent.format_message(fluent_message),
            AccessError::Vaild(valid_error) => valid_error.fluent_format(fluent),
            _ => fluent.format_message(&self.to_fluent_message()),
        }
    }
}

impl FluentFormat for UserAuthError {
    fn fluent_format(&self, fluent: &FluentBundle) -> String {
        match self {
            UserAuthError::TokenParse(fluent_message) => fluent.format_message(fluent_message),
            UserAuthError::Sqlx(error) => error.fluent_format(fluent),
            UserAuthError::Redis(error) => error.fluent_format(fluent),
            UserAuthError::RedisPool(error) => error.fluent_format(fluent),
            UserAuthError::AccessError(error) => error.fluent_format(fluent),
            UserAuthError::ValidCode(error) => error.fluent_format(fluent),
            UserAuthError::NotLogin(fluent_message) => fluent.format_message(fluent_message),
            UserAuthError::System(fluent_message) => fluent.format_message(fluent_message),
            UserAuthError::CheckUserLock(e) => fluent.format_message(&e.1),
            UserAuthError::CheckCaptchaNeed(fluent_message) => {
                fluent.format_message(fluent_message)
            }
            UserAuthError::Utf8Err(error) => error.fluent_format(fluent),
            UserAuthError::Vaild(error) => error.fluent_format(fluent),
        }
    }
}

impl FluentFormat for RbacError {
    fn fluent_format(&self, fluent: &FluentBundle) -> String {
        match self {
            RbacError::Sqlx(error) => error.fluent_format(fluent),
            RbacError::System(fluent_message) => fluent.format_message(fluent_message),
            RbacError::Vaild(error) => error.fluent_format(fluent),
            _ => fluent.format_message(&self.to_fluent_message()),
        }
    }
}
impl FluentFormat for SettingError {
    fn fluent_format(&self, fluent: &FluentBundle) -> String {
        match self {
            SettingError::Sqlx(error) => error.fluent_format(fluent),
            SettingError::SerdeJson(error) => error.fluent_format(fluent),
            SettingError::Vaild(error) => error.fluent_format(fluent),
        }
    }
}
impl FluentFormat for SenderError {
    fn fluent_format(&self, fluent: &FluentBundle) -> String {
        match self {
            SenderError::Sqlx(error) => error.fluent_format(fluent),
            SenderError::Redis(error) => error.fluent_format(fluent),
            SenderError::RedisPool(error) => error.fluent_format(fluent),
            SenderError::Tera(error) => error.fluent_format(fluent),
            SenderError::System(fluent_message) => fluent.format_message(fluent_message),
            SenderError::Setting(error) => error.fluent_format(fluent),
            SenderError::Vaild(error) => error.fluent_format(fluent),
            SenderError::AppCore(error) => error.fluent_format(fluent),
        }
    }
}
impl FluentFormat for AppError {
    fn fluent_format(&self, fluent: &FluentBundle) -> String {
        match self {
            AppError::Sqlx(error) => error.fluent_format(fluent),
            AppError::System(fluent_message) => fluent.format_message(fluent_message),
            AppError::Redis(error) => error.fluent_format(fluent),
            AppError::RedisPool(error) => error.fluent_format(fluent),
            AppError::Access(error) => error.fluent_format(fluent),
            AppError::SerdeJson(error) => error.fluent_format(fluent),
            AppError::AppCore(error) => error.fluent_format(fluent),
            AppError::Vaild(error) => error.fluent_format(fluent),
            _ => fluent.format_message(&self.to_fluent_message()),
        }
    }
}
impl FluentFormat for ConfigError {
    fn fluent_format(&self, fluent: &FluentBundle) -> String {
        match self {
            ConfigError::Io(error) => error.fluent_format(fluent),
            ConfigError::Config(error) => error.fluent_format(fluent),
        }
    }
}
impl FluentFormat for LoggerError {
    fn fluent_format(&self, fluent: &FluentBundle) -> String {
        match self {
            LoggerError::Sqlx(error) => error.fluent_format(fluent),
        }
    }
}
impl FluentFormat for ValidError {
    fn fluent_format(&self, fluent: &FluentBundle) -> String {
        fluent.format_message(&self.to_fluent_message())
    }
}

#[cfg(feature = "docs")]
impl FluentFormat for GitDocError {
    fn fluent_format(&self, fluent: &FluentBundle) -> String {
        match self {
            GitDocError::Sqlx(error) => error.fluent_format(fluent),
            GitDocError::Git(error) => error.fluent_format(fluent),
            GitDocError::System(fluent_message) => fluent.format_message(fluent_message),
            GitDocError::Remote(fluent_message) => fluent.format_message(fluent_message),
            GitDocError::Vaild(valid_error) => valid_error.fluent_format(fluent),
        }
    }
}

#[cfg(feature = "barcode")]
impl FluentFormat for lsys_app_barcode::dao::BarCodeError {
    fn fluent_format(&self, fluent: &FluentBundle) -> String {
        match self {
            lsys_app_barcode::dao::BarCodeError::System(err) => fluent.format_message(err),
            lsys_app_barcode::dao::BarCodeError::DB(err) => err.fluent_format(fluent),
            lsys_app_barcode::dao::BarCodeError::Io(err) => err.fluent_format(fluent),
            lsys_app_barcode::dao::BarCodeError::Vaild(err) => err.fluent_format(fluent),
            _ => fluent.format_message(&self.to_fluent_message()),
        }
    }
}
#[cfg(feature = "barcode")]
crate_error_fluent_string!(base64::DecodeError, "base64-error");
