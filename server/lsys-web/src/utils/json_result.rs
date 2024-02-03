use lsys_core::{ConfigError, FluentBundle, FluentMessage, ValidCodeError};
use lsys_docs::dao::GitDocError;
use lsys_logger::dao::LoggerError;
use lsys_notify::dao::NotifyError;
use lsys_rbac::dao::rbac::UserRbacError;
use lsys_sender::dao::SenderError;
use lsys_setting::dao::SettingError;
use lsys_user::dao::{account::UserAccountError, auth::UserAuthError};
use serde_json::{json, Value};

use std::{collections::HashMap, num::ParseIntError};

use crate::FluentFormat;
use lsys_app::dao::AppsError;

pub trait FluentJsonData {
    fn json_data(&self, fluent: &FluentBundle) -> JsonData;
}

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
impl JsonData {
    pub fn fluent_from<T: FluentJsonData>(fluent: &FluentBundle, data: T) -> JsonData {
        data.json_data(fluent)
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

impl FluentJsonData for FluentMessage {
    fn json_data(&self, fluent: &FluentBundle) -> JsonData {
        JsonData::default()
            .set_sub_code("system")
            .set_message(self.fluent_format(fluent))
    }
}

impl FluentJsonData for UserAccountError {
    fn json_data(&self, fluent: &FluentBundle) -> JsonData {
        let out = JsonData::default().set_sub_code("valid_code");
        match self {
            UserAccountError::Param(_) => out.set_message(self.fluent_format(fluent)),
            UserAccountError::System(_) => out.set_message(self.fluent_format(fluent)),
            UserAccountError::Status(_) => out.set_message(self.fluent_format(fluent)),
            UserAccountError::Redis(err) => err.json_data(fluent),
            UserAccountError::RedisPool(err) => err.json_data(fluent),
            UserAccountError::ValidCode(err) => err.json_data(fluent),
            UserAccountError::Sqlx(err) => err.json_data(fluent),
            UserAccountError::Setting(err) => err.json_data(fluent),
        }
    }
}

impl FluentJsonData for ValidCodeError {
    fn json_data(&self, fluent: &FluentBundle) -> JsonData {
        let out = JsonData::default().set_sub_code("valid_code");
        match self {
            ValidCodeError::DelayTimeout(err) => out
                .set_data(json!({
                    "type":err.prefix
                }))
                .set_message(self.fluent_format(fluent)),
            ValidCodeError::NotMatch(err) => out
                .set_data(json!({
                    "type":err.prefix
                }))
                .set_message(self.fluent_format(fluent)),
            ValidCodeError::Utf8Err(_) => out.set_message(self.fluent_format(fluent)),
            ValidCodeError::Tag(_) => out.set_message(self.fluent_format(fluent)),
            // ValidCodeError::Create(err) => out.set_message(fluent.format_message(&err.into())),
            ValidCodeError::Redis(err) => err.json_data(fluent),
            ValidCodeError::RedisPool(err) => err.json_data(fluent),
        }
    }
}

impl FluentJsonData for UserAuthError {
    fn json_data(&self, fluent: &FluentBundle) -> JsonData {
        let out = JsonData::default();
        match self {
            UserAuthError::PasswordNotMatch((_, _)) => out
                .set_sub_code("password_wrong")
                .set_message(self.fluent_format(fluent)),
            UserAuthError::PasswordNotSet(_) => out
                .set_sub_code("password_empty")
                .set_message(self.fluent_format(fluent)),
            UserAuthError::StatusError(_) => out
                .set_sub_code("status_wrong")
                .set_message(self.fluent_format(fluent)),
            UserAuthError::UserNotFind(_) => out
                .set_sub_code("not_find")
                .set_message(self.fluent_format(fluent)),
            UserAuthError::NotLogin(_) => out
                .set_sub_code("not_login")
                .set_message(self.fluent_format(fluent)),
            UserAuthError::System(_) => out
                .set_code(500)
                .set_sub_code("system")
                .set_message(self.fluent_format(fluent)),
            UserAuthError::CheckCaptchaNeed(_) => out
                .set_sub_code("need_captcha")
                .set_message(self.fluent_format(fluent)),
            UserAuthError::CheckUserLock(_) => out
                .set_sub_code("user_lock")
                .set_message(self.fluent_format(fluent)),
            UserAuthError::TokenParse(_) => out
                .set_sub_code("token_wrong")
                .set_message(self.fluent_format(fluent)),
            UserAuthError::Sqlx(err) => err.json_data(fluent),
            UserAuthError::UserAccount(err) => err.json_data(fluent),
            UserAuthError::ValidCode(err) => err.json_data(fluent),
            UserAuthError::Redis(err) => err.json_data(fluent),
            UserAuthError::RedisPool(err) => err.json_data(fluent),
        }
    }
}

impl FluentJsonData for UserRbacError {
    fn json_data(&self, fluent: &FluentBundle) -> JsonData {
        let out = JsonData::default().set_sub_code("rbac");
        match self {
            UserRbacError::Sqlx(err) => err.json_data(fluent),
            UserRbacError::NotLogin(_) => out
                .set_sub_code("not_login")
                .set_message(self.fluent_format(fluent)),
            UserRbacError::Check(err) => {
                let mut hash = HashMap::<&String, Vec<String>>::new();
                for (k, v) in err {
                    hash.entry(k).or_default().push(fluent.format_message(v));
                }
                out.set_sub_code("check_fail")
                    .set_data(json!( {
                        "check_detail":hash,
                    }))
                    .set_message(self.fluent_format(fluent))
            }
            UserRbacError::System(_) => out.set_message(self.fluent_format(fluent)),
        }
    }
}

impl FluentJsonData for SettingError {
    fn json_data(&self, fluent: &FluentBundle) -> JsonData {
        match self {
            SettingError::Sqlx(err) => err.json_data(fluent),
            SettingError::SerdeJson(err) => err.json_data(fluent),
        }
    }
}

impl FluentJsonData for SenderError {
    fn json_data(&self, fluent: &FluentBundle) -> JsonData {
        let out = JsonData::default().set_code(500).set_sub_code("sender");
        match self {
            SenderError::Sqlx(err) => err.json_data(fluent),
            SenderError::Redis(err) => err.json_data(fluent),
            SenderError::RedisPool(err) => err.json_data(fluent),
            SenderError::Tera(err) => err.json_data(fluent),
            SenderError::Setting(err) => err.json_data(fluent),
            SenderError::System(_) => out.set_message(self.fluent_format(fluent)),
        }
    }
}

impl FluentJsonData for AppsError {
    fn json_data(&self, fluent: &FluentBundle) -> JsonData {
        let out = JsonData::default().set_code(500).set_sub_code("app");
        match self {
            AppsError::ScopeNotFind(_) => out
                .set_sub_code("app-bad-scope")
                .set_message(self.fluent_format(fluent)),
            AppsError::System(_) => out.set_message(self.fluent_format(fluent)),
            AppsError::Sqlx(err) => err.json_data(fluent),
            AppsError::Redis(err) => err.json_data(fluent),
            AppsError::RedisPool(err) => err.json_data(fluent),
            AppsError::UserAccount(err) => err.json_data(fluent),
            AppsError::SerdeJson(err) => err.json_data(fluent),
        }
    }
}

impl FluentJsonData for ConfigError {
    fn json_data(&self, fluent: &FluentBundle) -> JsonData {
        match self {
            ConfigError::Io(err) => err.json_data(fluent),
            ConfigError::Config(err) => err.json_data(fluent),
        }
    }
}

impl FluentJsonData for NotifyError {
    fn json_data(&self, fluent: &FluentBundle) -> JsonData {
        match self {
            NotifyError::Sqlx(err) => err.json_data(fluent),
            NotifyError::Redis(err) => err.json_data(fluent),
            NotifyError::RedisPool(err) => err.json_data(fluent),
            NotifyError::System(_) => JsonData::default()
                .set_code(500)
                .set_sub_code("notify")
                .set_message(self.fluent_format(fluent)),
        }
    }
}

impl FluentJsonData for GitDocError {
    fn json_data(&self, fluent: &FluentBundle) -> JsonData {
        match self {
            GitDocError::Sqlx(err) => err.json_data(fluent),
            GitDocError::Git(err) => err.json_data(fluent),
            GitDocError::System(_) => JsonData::default()
                .set_sub_code("doc")
                .set_code(200)
                .set_message(self.fluent_format(fluent)),
            GitDocError::Remote(_) => JsonData::default()
                .set_sub_code("doc")
                .set_code(200)
                .set_message(self.fluent_format(fluent)),
        }
    }
}

impl FluentJsonData for LoggerError {
    fn json_data(&self, fluent: &FluentBundle) -> JsonData {
        match self {
            LoggerError::Sqlx(err) => err.json_data(fluent),
        }
    }
}

impl FluentJsonData for area_db::AreaError {
    fn json_data(&self, fluent: &FluentBundle) -> JsonData {
        match self {
            area_db::AreaError::DB(_) => JsonData::default()
                .set_code(500)
                .set_sub_code("area")
                .set_message(self.fluent_format(fluent)),
            area_db::AreaError::System(_) => JsonData::default()
                .set_code(500)
                .set_sub_code("area")
                .set_message(self.fluent_format(fluent)),
            area_db::AreaError::NotFind(_) => JsonData::default()
                .set_sub_code("not_found")
                .set_message(self.fluent_format(fluent)),
            area_db::AreaError::Store(_) => JsonData::default()
                .set_sub_code("area")
                .set_message(self.fluent_format(fluent)),
            area_db::AreaError::Tantivy(_) => JsonData::default()
                .set_sub_code("tantivy")
                .set_message(self.fluent_format(fluent)),
        }
    }
}

//lib error

impl FluentJsonData for sqlx::Error {
    fn json_data(&self, fluent: &FluentBundle) -> JsonData {
        let out = JsonData::default().set_code(500);
        match self {
            sqlx::Error::RowNotFound => out.set_sub_code("not_found").set_code(404),
            _ => out.set_sub_code("sqlx"),
        }
        .set_message(self.fluent_format(fluent))
    }
}
macro_rules! crate_error_fluent {
    ($crate_error:ty,$code:literal) => {
        impl FluentJsonData for $crate_error {
            fn json_data(&self, fluent: &FluentBundle) -> JsonData {
                JsonData::default()
                    .set_code(500)
                    .set_sub_code($code)
                    .set_message(self.fluent_format(fluent))
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
