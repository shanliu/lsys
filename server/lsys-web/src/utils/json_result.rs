use lsys_core::{fluent_message, ConfigError, FluentBundle, FluentMessage, ValidCodeError};
use lsys_docs::dao::GitDocError;
use lsys_logger::dao::LoggerError;
use lsys_notify::dao::NotifyError;
use lsys_rbac::dao::rbac::UserRbacError;
use lsys_sender::dao::SenderError;
use lsys_setting::dao::SettingError;
use lsys_user::dao::{account::UserAccountError, auth::UserAuthError};
use serde_json::{json, Value};

use std::{collections::HashMap, num::ParseIntError};

use crate::dao::{WebAppMailerError, WebAppSmserError};
use lsys_app::dao::AppsError;

pub type JsonResult<T> = Result<T, JsonData>;

#[derive(Debug, Clone)]
pub struct JsonData {
    code: String,
    sub_code: String,
    message: String,
    data: Option<Value>,
}
impl Default for JsonData {
    fn default() -> Self {
        JsonData {
            code: "200".to_string(),
            sub_code: "ok".to_string(),
            message: "ok".to_string(),
            data: None,
        }
    }
}

pub trait JsonDataFluent {
    fn fluent_from(&self, fluent: &FluentBundle) -> JsonData;
}

impl JsonData {
    pub fn fluent_from<T: JsonDataFluent>(fluent: &FluentBundle, data: T) -> JsonData {
        data.fluent_from(fluent)
    }
    pub fn data(value: Value) -> Self {
        JsonData::default().set_message("ok").set_data(value)
    }
    pub fn error<T: std::error::Error>(error: T) -> Self {
        JsonData::message_error(format!("error:{}", error))
    }
    pub fn message_error<T: ToString>(msg: T) -> Self {
        JsonData::message(msg).set_code(500).set_sub_code("system")
    }
    pub fn message<T: ToString>(msg: T) -> Self {
        JsonData::default().set_message(msg)
    }
    pub fn set_data(mut self, value: Value) -> Self {
        self.data = Some(value);
        self
    }
    #[allow(dead_code)]
    pub fn set_total_data<T: ToString>(mut self, value: Value, total: T) -> Self {
        self.data = Some(json!({
            "total":total.to_string(),
            "data":value,
        }));
        self
    }
    pub fn set_code<T: ToString>(mut self, code: T) -> Self {
        self.code = code.to_string();
        self
    }
    pub fn set_sub_code<T: ToString>(mut self, sub_code: T) -> Self {
        self.sub_code = sub_code.to_string();
        self
    }
    pub fn set_message<T: ToString>(mut self, msg: T) -> Self {
        self.message = msg.to_string();
        self
    }
    pub fn to_value(&self) -> Value {
        if self.data.is_none() {
            json!({
                "result": {
                    "code": self.code,
                    "state":self.sub_code,
                    "message": self.message,
                },
            })
        } else {
            json!({
                "result": {
                    "code": self.code,
                    "state":self.sub_code,
                    "message": self.message,
                },
                "response": self.data
            })
        }
    }
}

impl JsonDataFluent for FluentMessage {
    fn fluent_from(&self, fluent: &FluentBundle) -> JsonData {
        JsonData::default()
            .set_sub_code("system")
            .set_message(fluent.format_message(self))
    }
}

impl JsonDataFluent for UserAccountError {
    fn fluent_from(&self, fluent: &FluentBundle) -> JsonData {
        let out = JsonData::default().set_sub_code("valid_code");
        match self {
            UserAccountError::Sqlx(err) => err.fluent_from(fluent),
            UserAccountError::System(err) => out.set_message(fluent.format_message(err)),
            UserAccountError::Status((_, err)) => out.set_message(fluent.format_message(err)),
            UserAccountError::Redis(err) => err.fluent_from(fluent),
            UserAccountError::RedisPool(err) => err.fluent_from(fluent),
            UserAccountError::ValidCode(err) => err.fluent_from(fluent),
            UserAccountError::Param(err) => out.set_message(fluent.format_message(err)),
            UserAccountError::Setting(err) => err.fluent_from(fluent),
        }
    }
}

impl JsonDataFluent for ValidCodeError {
    fn fluent_from(&self, fluent: &FluentBundle) -> JsonData {
        let out = JsonData::default().set_sub_code("valid_code");
        match self {
            ValidCodeError::DelayTimeout(err) => out
                .set_data(json!({
                    "type":err.prefix
                }))
                .set_message(fluent.format_message(&err.message)),
            ValidCodeError::NotMatch(err) => out
                .set_data(json!({
                    "type":err.prefix
                }))
                .set_message(fluent.format_message(&err.message)),
            ValidCodeError::Utf8Err(err) => out.set_message(fluent.format_message(err)),
            // ValidCodeError::Create(err) => out.set_message(fluent.format_message(&err.into())),
            ValidCodeError::Redis(err) => err.fluent_from(fluent),
            ValidCodeError::RedisPool(err) => err.fluent_from(fluent),
            ValidCodeError::Tag(err) => out.set_message(fluent.format_message(err)),
        }
    }
}

impl JsonDataFluent for UserAuthError {
    fn fluent_from(&self, fluent: &FluentBundle) -> JsonData {
        let out = JsonData::default();
        match self {
            UserAuthError::PasswordNotMatch((_, err)) => out
                .set_sub_code("password_wrong")
                .set_message(fluent.format_message(err)),
            UserAuthError::PasswordNotSet((_, err)) => out
                .set_sub_code("password_empty")
                .set_message(fluent.format_message(err)),
            UserAuthError::StatusError((_, err)) => out
                .set_sub_code("status_wrong")
                .set_message(fluent.format_message(err)),
            UserAuthError::UserNotFind(err) => out
                .set_sub_code("not_find")
                .set_message(fluent.format_message(err)),
            UserAuthError::NotLogin(err) => out
                .set_sub_code("not_login")
                .set_message(fluent.format_message(err)),
            UserAuthError::Sqlx(err) => err.fluent_from(fluent),
            UserAuthError::UserAccount(err) => err.fluent_from(fluent),
            UserAuthError::ValidCode(err) => err.fluent_from(fluent),
            UserAuthError::System(err) => out
                .set_code(500)
                .set_sub_code("system")
                .set_message(fluent.format_message(err)),
            UserAuthError::CheckCaptchaNeed(err) => out
                .set_sub_code("need_captcha")
                .set_message(fluent.format_message(err)),
            UserAuthError::Redis(err) => err.fluent_from(fluent),
            UserAuthError::RedisPool(err) => err.fluent_from(fluent),
            UserAuthError::CheckUserLock((_, err)) => out
                .set_sub_code("user_lock")
                .set_message(fluent.format_message(err)),
            UserAuthError::TokenParse(err) => out
                .set_sub_code("token_wrong")
                .set_message(fluent.format_message(err)),
        }
    }
}

impl JsonDataFluent for UserRbacError {
    fn fluent_from(&self, fluent: &FluentBundle) -> JsonData {
        let out = JsonData::default().set_sub_code("rbac");
        match self {
            UserRbacError::Sqlx(err) => err.fluent_from(fluent),
            UserRbacError::NotLogin(err) => out.set_sub_code("not_login").set_message(err),
            UserRbacError::Check(err) => {
                let mut hash = HashMap::<&String, Vec<String>>::new();
                for (k, v) in err {
                    hash.entry(k).or_default().push(fluent.format_message(v));
                }
                out.set_sub_code("check_fail")
                    .set_data(json!( {
                        "check_detail":hash,
                    }))
                    .set_message(fluent.format_message(&fluent_message!("rbac-check-fail")))
            }
            UserRbacError::System(err) => out.set_message(err),
        }
    }
}

impl JsonDataFluent for SettingError {
    fn fluent_from(&self, fluent: &FluentBundle) -> JsonData {
        match self {
            SettingError::Sqlx(err) => err.fluent_from(fluent),
            SettingError::SerdeJson(err) => err.fluent_from(fluent),
        }
    }
}

impl JsonDataFluent for SenderError {
    fn fluent_from(&self, fluent: &FluentBundle) -> JsonData {
        let out = JsonData::default().set_code(500).set_sub_code("sender");
        match self {
            SenderError::Sqlx(err) => err.fluent_from(fluent),
            SenderError::Redis(err) => err.fluent_from(fluent),
            SenderError::RedisPool(err) => err.fluent_from(fluent),
            SenderError::Tera(err) => err.fluent_from(fluent),
            SenderError::System(err) => out.set_message(fluent.format_message(err)),
            SenderError::Setting(err) => err.fluent_from(fluent),
        }
    }
}

impl JsonDataFluent for WebAppSmserError {
    fn fluent_from(&self, fluent: &FluentBundle) -> JsonData {
        let out = JsonData::default().set_code(500).set_sub_code("sms");
        match self {
            WebAppSmserError::Config(err) => {
                out.set_message(fluent.format_message(&err.to_string().into()))
            }
            WebAppSmserError::System(err) => out.set_message(fluent.format_message(err)),
            WebAppSmserError::Sender(err) => err.fluent_from(fluent),
        }
    }
}
impl JsonDataFluent for WebAppMailerError {
    fn fluent_from(&self, fluent: &FluentBundle) -> JsonData {
        let out = JsonData::default().set_code(500).set_sub_code("sms");
        match self {
            WebAppMailerError::Config(err) => {
                out.set_message(fluent.format_message(&err.to_string().into()))
            }
            WebAppMailerError::System(err) => out.set_message(fluent.format_message(err)),
            WebAppMailerError::Sender(err) => err.fluent_from(fluent),
            WebAppMailerError::Tera(err) => err.fluent_from(fluent),
        }
    }
}

impl JsonDataFluent for AppsError {
    fn fluent_from(&self, fluent: &FluentBundle) -> JsonData {
        let out = JsonData::default().set_code(500).set_sub_code("app");
        match self {
            AppsError::Sqlx(err) => err.fluent_from(fluent),
            AppsError::System(err) => out.set_message(fluent.format_message(err)),
            AppsError::Redis(err) => err.fluent_from(fluent),
            AppsError::RedisPool(err) => err.fluent_from(fluent),
            AppsError::ScopeNotFind(err) => out
                .set_sub_code("app-bad-scope")
                .set_message(fluent.format_message(err)),
            AppsError::UserAccount(err) => err.fluent_from(fluent),
            AppsError::SerdeJson(err) => err.fluent_from(fluent),
        }
    }
}

impl JsonDataFluent for ConfigError {
    fn fluent_from(&self, fluent: &FluentBundle) -> JsonData {
        match self {
            ConfigError::Io(err) => err.fluent_from(fluent),
            ConfigError::Config(err) => err.fluent_from(fluent),
        }
    }
}

impl JsonDataFluent for NotifyError {
    fn fluent_from(&self, fluent: &FluentBundle) -> JsonData {
        match self {
            NotifyError::Sqlx(err) => err.fluent_from(fluent),
            NotifyError::Redis(err) => err.fluent_from(fluent),
            NotifyError::RedisPool(err) => err.fluent_from(fluent),
            NotifyError::System(err) => JsonData::default()
                .set_code(500)
                .set_sub_code("notify")
                .set_message(fluent.format_message(err)),
        }
    }
}

impl JsonDataFluent for GitDocError {
    fn fluent_from(&self, fluent: &FluentBundle) -> JsonData {
        match self {
            GitDocError::Sqlx(err) => err.fluent_from(fluent),
            GitDocError::System(err) => JsonData::default()
                .set_sub_code("doc")
                .set_code(200)
                .set_message(fluent.format_message(err)),
            GitDocError::Remote(err) => JsonData::default()
                .set_sub_code("doc")
                .set_code(200)
                .set_message(fluent.format_message(err)),
            GitDocError::Git(err) => err.fluent_from(fluent),
        }
    }
}

impl JsonDataFluent for LoggerError {
    fn fluent_from(&self, fluent: &FluentBundle) -> JsonData {
        match self {
            LoggerError::Sqlx(err) => err.fluent_from(fluent),
        }
    }
}

impl JsonDataFluent for area_db::AreaError {
    fn fluent_from(&self, fluent: &FluentBundle) -> JsonData {
        match self {
            area_db::AreaError::DB(err) => JsonData::default()
                .set_code(500)
                .set_sub_code("area")
                .set_message(fluent.format_message(&err.into())),
            area_db::AreaError::System(err) => JsonData::default()
                .set_code(500)
                .set_sub_code("area")
                .set_message(fluent.format_message(&err.into())),
            area_db::AreaError::NotFind(_) => JsonData::default()
                .set_sub_code("not_found")
                .set_message(fluent.format_message(&"not find area record".into())),
            area_db::AreaError::Store(e) => JsonData::default()
                .set_sub_code("area")
                .set_message(fluent.format_message(&format!("index area data fail:{}", e).into())),
            area_db::AreaError::Tantivy(e) => {
                JsonData::default().set_sub_code("tantivy").set_message(
                    fluent.format_message(&format!("tantivy area data fail:{}", e).into()),
                )
            }
        }
    }
}

//lib error

impl JsonDataFluent for sqlx::Error {
    fn fluent_from(&self, fluent: &FluentBundle) -> JsonData {
        let out = JsonData::default().set_code(500);
        match self {
            sqlx::Error::RowNotFound => out
                .set_sub_code("not_found")
                .set_code(404)
                .set_message(fluent.format_message(&fluent_message!("system-not-found", self))),
            _ => {
                let msg = fluent.format_message(&self.to_string().into());
                out.set_sub_code("sqlx").set_message(msg)
            }
        }
    }
}
macro_rules! crate_error_fluent {
    ($crate_error:ty,$code:literal) => {
        impl JsonDataFluent for $crate_error {
            fn fluent_from(&self, fluent: &FluentBundle) -> JsonData {
                JsonData::default()
                    .set_code(500)
                    .set_sub_code($code)
                    .set_message(fluent.format_message(&self.to_string().into()))
            }
        }
    };
}
crate_error_fluent!(config::ConfigError, "config");
crate_error_fluent!(std::io::Error, "io");
crate_error_fluent!(tera::Error, "tera");
crate_error_fluent!(lsys_docs::gitError, "git");
crate_error_fluent!(redis::RedisError, "redis");
crate_error_fluent!(deadpool_redis::PoolError, "redis");
crate_error_fluent!(serde_json::Error, "serde");
crate_error_fluent!(ParseIntError, "parse");
