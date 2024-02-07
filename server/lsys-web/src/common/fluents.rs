use lsys_core::{
    fluent_message, ConfigError, FluentBundle, FluentMessage, IntoFluentMessage, ValidCodeError,
};
use lsys_docs::dao::GitDocError;
use lsys_logger::dao::LoggerError;
use lsys_notify::dao::NotifyError;
use lsys_rbac::dao::rbac::UserRbacError;
use lsys_sender::dao::SenderError;
use lsys_setting::dao::SettingError;
use lsys_user::dao::{account::UserAccountError, auth::UserAuthError};

use std::num::ParseIntError;

use lsys_app::dao::AppsError;

pub trait FluentFormat {
    fn fluent_format(&self, fluent: &FluentBundle) -> String;
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
self_error_fluent_string!(UserAccountError);
self_error_fluent_string!(ValidCodeError);
self_error_fluent_string!(UserAuthError);
self_error_fluent_string!(UserRbacError);
self_error_fluent_string!(SettingError);
self_error_fluent_string!(SenderError);
self_error_fluent_string!(AppsError);
self_error_fluent_string!(ConfigError);
self_error_fluent_string!(NotifyError);
self_error_fluent_string!(GitDocError);
self_error_fluent_string!(LoggerError);

impl FluentFormat for FluentMessage {
    fn fluent_format(&self, fluent: &FluentBundle) -> String {
        fluent.format_message(self)
    }
}

impl FluentFormat for area_db::AreaError {
    fn fluent_format(&self, fluent: &FluentBundle) -> String {
        match self {
            area_db::AreaError::DB(err) => {
                fluent.format_message(&fluent_message!("area-db-error", err))
            }
            area_db::AreaError::System(err) => {
                fluent.format_message(&fluent_message!("area-error", err))
            }
            area_db::AreaError::NotFind(_) => {
                fluent.format_message(&fluent_message!("area-not-found"))
            }
            area_db::AreaError::Store(err) => {
                fluent.format_message(&fluent_message!("area-store-error", err))
            }
            area_db::AreaError::Tantivy(err) => {
                fluent.format_message(&fluent_message!("area-tantivy-error", err))
            }
        }
    }
}

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

crate_error_fluent_string!(config::ConfigError, "config-error");
crate_error_fluent_string!(std::io::Error, "io-error");
crate_error_fluent_string!(tera::Error, "tera-error");
crate_error_fluent_string!(lsys_docs::GitError, "git-error");
crate_error_fluent_string!(redis::RedisError, "redis-error");
crate_error_fluent_string!(deadpool_redis::PoolError, "redis-error");
crate_error_fluent_string!(serde_json::Error, "serde-json-error");
crate_error_fluent_string!(ParseIntError, "parse-error");
crate_error_fluent_string!(std::string::FromUtf8Error, "utf8-parse-error");
