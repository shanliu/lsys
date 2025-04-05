use lsys_access::dao::AccessError;
use lsys_app_notify::dao::NotifyError;
use lsys_app_sender::dao::SenderError;
use lsys_core::{fluent_message, ConfigError, FluentBundle, IntoFluentMessage, ValidCodeError};

use lsys_logger::dao::LoggerError;
use lsys_rbac::dao::RbacError;
use lsys_setting::dao::SettingError;
use lsys_user::dao::AccountError;
use lsys_user::dao::UserAuthError;

use std::num::ParseIntError;

use lsys_app::dao::AppError;

pub trait FluentFormat {
    fn fluent_format(&self, fluent: &FluentBundle) -> String;
}

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
macro_rules! self_error_fluent_string {
    ($self_error:ty) => {
        impl FluentFormat for $self_error {
            fn fluent_format(&self, fluent: &FluentBundle) -> String {
                fluent.format_message(&self.to_fluent_message())
            }
        }
    };
}
self_error_fluent_string!(AccountError);
self_error_fluent_string!(ValidCodeError);
self_error_fluent_string!(AccessError);
self_error_fluent_string!(UserAuthError);
self_error_fluent_string!(RbacError);
self_error_fluent_string!(SettingError);
self_error_fluent_string!(SenderError);
self_error_fluent_string!(AppError);
self_error_fluent_string!(ConfigError);
self_error_fluent_string!(NotifyError);
self_error_fluent_string!(LoggerError);

//crate error

macro_rules! crate_error_fluent_string {
    ($crate_error:ty,$key:literal) => {
        impl FluentFormat for $crate_error {
            fn fluent_format(&self, fluent: &FluentBundle) -> String {
                fluent.format_message(&fluent_message!($key, self))
            }
        }
    };
}

crate_error_fluent_string!(config::ConfigError, "config-error");
crate_error_fluent_string!(std::io::Error, "io-error");
crate_error_fluent_string!(tera::Error, "tera-error");
crate_error_fluent_string!(redis::RedisError, "redis-error");
crate_error_fluent_string!(deadpool_redis::PoolError, "redis-error");
crate_error_fluent_string!(serde_json::Error, "serde-json-error");
crate_error_fluent_string!(ParseIntError, "parse-error");
crate_error_fluent_string!(std::string::FromUtf8Error, "utf8-parse-error");

#[cfg(feature = "docs")]
use lsys_docs::dao::GitDocError;

#[cfg(feature = "docs")]
crate_error_fluent_string!(lsys_docs::GitError, "git-error");
#[cfg(feature = "docs")]
self_error_fluent_string!(GitDocError);

impl FluentFormat for lsys_lib_area::AreaError {
    fn fluent_format(&self, fluent: &FluentBundle) -> String {
        match self {
            lsys_lib_area::AreaError::DB(err) => {
                fluent.format_message(&fluent_message!("lsys-lib-area-error", err))
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

#[cfg(feature = "barcode")]
impl FluentFormat for lsys_app_barcode::dao::BarCodeError {
    fn fluent_format(&self, fluent: &FluentBundle) -> String {
        match self {
            lsys_app_barcode::dao::BarCodeError::System(err) => fluent.format_message(err),
            lsys_app_barcode::dao::BarCodeError::DB(err) => {
                fluent.format_message(&fluent_message!("lsys-lib-area-error", err))
            }
            lsys_app_barcode::dao::BarCodeError::RXing(err) => {
                fluent.format_message(&fluent_message!("lsys-rxing-error", err))
            }
            lsys_app_barcode::dao::BarCodeError::Io(err) => {
                fluent.format_message(&fluent_message!("io-error", err))
            }
            lsys_app_barcode::dao::BarCodeError::Image(err) => {
                fluent.format_message(&fluent_message!("barcode-image-error", err))
            }
        }
    }
}
