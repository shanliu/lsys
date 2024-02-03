use lsys_core::{
    fluent_message, ConfigError, FluentBundle, FluentMessage, ValidCodeCheckError, ValidCodeError,
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

impl FluentFormat for UserAccountError {
    fn fluent_format(&self, fluent: &FluentBundle) -> String {
        match self {
            UserAccountError::System(err) => fluent.format_message(err),
            UserAccountError::Status((_, err)) => fluent.format_message(err),
            UserAccountError::Param(err) => fluent.format_message(err),
            UserAccountError::Redis(err) => err.fluent_format(fluent),
            UserAccountError::RedisPool(err) => err.fluent_format(fluent),
            UserAccountError::ValidCode(err) => err.fluent_format(fluent),
            UserAccountError::Sqlx(err) => err.fluent_format(fluent),
            UserAccountError::Setting(err) => err.fluent_format(fluent),
        }
    }
}

impl FluentFormat for FluentMessage {
    fn fluent_format(&self, fluent: &FluentBundle) -> String {
        fluent.format_message(self)
    }
}

impl FluentFormat for ValidCodeError {
    fn fluent_format(&self, fluent: &FluentBundle) -> String {
        match self {
            ValidCodeError::Utf8Err(err) => fluent.format_message(err),
            ValidCodeError::Tag(err) => fluent.format_message(err),
            ValidCodeError::DelayTimeout(err) => err.fluent_format(fluent),
            ValidCodeError::NotMatch(err) => err.fluent_format(fluent),
            ValidCodeError::Redis(err) => err.fluent_format(fluent),
            ValidCodeError::RedisPool(err) => err.fluent_format(fluent),
        }
    }
}

impl FluentFormat for UserAuthError {
    fn fluent_format(&self, fluent: &FluentBundle) -> String {
        match self {
            UserAuthError::PasswordNotMatch((_, err)) => fluent.format_message(err),
            UserAuthError::PasswordNotSet((_, err)) => fluent.format_message(err),
            UserAuthError::StatusError((_, err)) => fluent.format_message(err),
            UserAuthError::UserNotFind(err) => fluent.format_message(err),
            UserAuthError::NotLogin(err) => fluent.format_message(err),
            UserAuthError::System(err) => fluent.format_message(err),
            UserAuthError::CheckUserLock((_, err)) => fluent.format_message(err),
            UserAuthError::TokenParse(err) => fluent.format_message(err),
            UserAuthError::CheckCaptchaNeed(err) => fluent.format_message(err),
            UserAuthError::Sqlx(err) => err.fluent_format(fluent),
            UserAuthError::UserAccount(err) => err.fluent_format(fluent),
            UserAuthError::ValidCode(err) => err.fluent_format(fluent),
            UserAuthError::Redis(err) => err.fluent_format(fluent),
            UserAuthError::RedisPool(err) => err.fluent_format(fluent),
        }
    }
}

impl FluentFormat for UserRbacError {
    fn fluent_format(&self, fluent: &FluentBundle) -> String {
        match self {
            UserRbacError::Sqlx(err) => err.fluent_format(fluent),
            UserRbacError::NotLogin(err) => fluent.format_message(err),
            UserRbacError::Check(_) => fluent.format_message(&fluent_message!("rbac-check-fail")),
            UserRbacError::System(err) => fluent.format_message(err),
        }
    }
}

impl FluentFormat for SettingError {
    fn fluent_format(&self, fluent: &FluentBundle) -> String {
        match self {
            SettingError::Sqlx(err) => err.fluent_format(fluent),
            SettingError::SerdeJson(err) => err.fluent_format(fluent),
        }
    }
}

impl FluentFormat for SenderError {
    fn fluent_format(&self, fluent: &FluentBundle) -> String {
        match self {
            SenderError::Sqlx(err) => err.fluent_format(fluent),
            SenderError::Redis(err) => err.fluent_format(fluent),
            SenderError::RedisPool(err) => err.fluent_format(fluent),
            SenderError::Tera(err) => err.fluent_format(fluent),
            SenderError::System(err) => fluent.format_message(err),
            SenderError::Setting(err) => err.fluent_format(fluent),
        }
    }
}
impl FluentFormat for ValidCodeCheckError {
    fn fluent_format(&self, fluent: &FluentBundle) -> String {
        fluent.format_message(&self.message)
    }
}

impl FluentFormat for AppsError {
    fn fluent_format(&self, fluent: &FluentBundle) -> String {
        match self {
            AppsError::Sqlx(err) => err.fluent_format(fluent),
            AppsError::System(err) => fluent.format_message(err),
            AppsError::Redis(err) => err.fluent_format(fluent),
            AppsError::RedisPool(err) => err.fluent_format(fluent),
            AppsError::ScopeNotFind(err) => fluent.format_message(err),
            AppsError::UserAccount(err) => err.fluent_format(fluent),
            AppsError::SerdeJson(err) => err.fluent_format(fluent),
        }
    }
}

impl FluentFormat for ConfigError {
    fn fluent_format(&self, fluent: &FluentBundle) -> String {
        match self {
            ConfigError::Io(err) => err.fluent_format(fluent),
            ConfigError::Config(err) => err.fluent_format(fluent),
        }
    }
}

impl FluentFormat for NotifyError {
    fn fluent_format(&self, fluent: &FluentBundle) -> String {
        match self {
            NotifyError::Sqlx(err) => err.fluent_format(fluent),
            NotifyError::Redis(err) => err.fluent_format(fluent),
            NotifyError::RedisPool(err) => err.fluent_format(fluent),
            NotifyError::System(err) => fluent.format_message(err),
        }
    }
}

impl FluentFormat for GitDocError {
    fn fluent_format(&self, fluent: &FluentBundle) -> String {
        match self {
            GitDocError::Sqlx(err) => err.fluent_format(fluent),
            GitDocError::System(err) => fluent.format_message(err),
            GitDocError::Remote(err) => fluent.format_message(err),
            GitDocError::Git(err) => err.fluent_format(fluent),
        }
    }
}

impl FluentFormat for LoggerError {
    fn fluent_format(&self, fluent: &FluentBundle) -> String {
        match self {
            LoggerError::Sqlx(err) => err.fluent_format(fluent),
        }
    }
}

impl FluentFormat for area_db::AreaError {
    fn fluent_format(&self, fluent: &FluentBundle) -> String {
        match self {
            area_db::AreaError::DB(err) => fluent.format_message(&err.into()),
            area_db::AreaError::System(err) => fluent.format_message(&err.into()),
            area_db::AreaError::NotFind(_) => fluent.format_message(&"not find area record".into()),
            area_db::AreaError::Store(e) => {
                fluent.format_message(&format!("index area data fail:{}", e).into())
            }
            area_db::AreaError::Tantivy(e) => {
                fluent.format_message(&format!("tantivy area data fail:{}", e).into())
            }
        }
    }
}

//lib error

impl FluentFormat for sqlx::Error {
    fn fluent_format(&self, fluent: &FluentBundle) -> String {
        match self {
            sqlx::Error::RowNotFound => {
                fluent.format_message(&fluent_message!("system-not-found", self))
            }
            _ => fluent.format_message(&self.to_string().into()),
        }
    }
}
macro_rules! crate_error_fluent {
    ($crate_error:ty) => {
        impl FluentFormat for $crate_error {
            fn fluent_format(&self, fluent: &FluentBundle) -> String {
                fluent.format_message(&self.to_string().into())
            }
        }
    };
}
crate_error_fluent!(config::ConfigError);
crate_error_fluent!(std::io::Error);
crate_error_fluent!(tera::Error);
crate_error_fluent!(lsys_docs::gitError);
crate_error_fluent!(redis::RedisError);
crate_error_fluent!(deadpool_redis::PoolError);
crate_error_fluent!(serde_json::Error);
crate_error_fluent!(ParseIntError);
