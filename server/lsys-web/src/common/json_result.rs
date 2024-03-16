use lsys_app_notify::dao::NotifyError;
use lsys_app_sender::dao::SenderError;
use lsys_core::{ConfigError, FluentBundle, FluentMessage, ValidCodeError};

use lsys_logger::dao::LoggerError;
use lsys_rbac::dao::rbac::UserRbacError;
use lsys_setting::dao::SettingError;
use lsys_user::dao::{account::UserAccountError, auth::UserAuthError};
use serde_json::{json, Value};

use std::{collections::HashMap, num::ParseIntError};

use crate::FluentFormat;
use lsys_app::dao::AppsError;

pub trait FluentJsonData {
    fn set_data(&self, json_data: JsonData, fluent: &FluentBundle) -> JsonData;
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
    pub fn fluent_from<T: FluentJsonData + FluentFormat>(
        fluent: &FluentBundle,
        data: T,
    ) -> JsonData {
        data.set_data(
            JsonData::default().set_message(data.fluent_format(fluent)),
            fluent,
        )
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
    fn set_data(&self, json_data: JsonData, _: &FluentBundle) -> JsonData {
        json_data
    }
}

impl FluentJsonData for UserAccountError {
    fn set_data(&self, json_data: JsonData, fluent: &FluentBundle) -> JsonData {
        let json_data = json_data.set_code(500).set_sub_code("user_account");
        match self {
            UserAccountError::Param(_) => json_data,
            UserAccountError::System(_) => json_data,
            UserAccountError::Status(_) => json_data,
            UserAccountError::Redis(err) => err.set_data(json_data, fluent),
            UserAccountError::RedisPool(err) => err.set_data(json_data, fluent),
            UserAccountError::ValidCode(err) => err.set_data(json_data, fluent),
            UserAccountError::Sqlx(err) => err.set_data(json_data, fluent),
            UserAccountError::Setting(err) => err.set_data(json_data, fluent),
        }
    }
}

impl FluentJsonData for ValidCodeError {
    fn set_data(&self, json_data: JsonData, fluent: &FluentBundle) -> JsonData {
        let json_data = json_data.set_code(500);
        match self {
            ValidCodeError::DelayTimeout(err) => {
                json_data.set_sub_code("valid_code").set_data(json!({
                    "type":err.prefix
                }))
            }
            ValidCodeError::NotMatch(err) => json_data.set_sub_code("valid_code").set_data(json!({
                "type":err.prefix
            })),
            ValidCodeError::Utf8Err(_) => json_data.set_sub_code("valid_code_err"),
            ValidCodeError::Tag(_) => json_data.set_sub_code("valid_code_err"),
            // ValidCodeError::Create(err) => json_data.set_message(fluent.format_message(&err.into())),
            ValidCodeError::Redis(err) => err.set_data(json_data, fluent),
            ValidCodeError::RedisPool(err) => err.set_data(json_data, fluent),
        }
    }
}

impl FluentJsonData for UserAuthError {
    fn set_data(&self, json_data: JsonData, fluent: &FluentBundle) -> JsonData {
        let json_data = json_data.set_code(500);
        match self {
            UserAuthError::PasswordNotMatch((_, _)) => json_data.set_sub_code("password_wrong"),
            UserAuthError::PasswordNotSet(_) => json_data.set_sub_code("password_empty"),
            UserAuthError::StatusError(_) => json_data.set_sub_code("status_wrong"),
            UserAuthError::UserNotFind(_) => json_data.set_sub_code("not_find"),
            UserAuthError::NotLogin(_) => json_data.set_sub_code("not_login"),
            UserAuthError::System(_) => json_data.set_sub_code("auth"),
            UserAuthError::CheckCaptchaNeed(_) => json_data.set_sub_code("need_captcha"),
            UserAuthError::CheckUserLock(_) => json_data.set_sub_code("user_lock"),
            UserAuthError::TokenParse(_) => json_data.set_sub_code("token_wrong"),
            UserAuthError::Sqlx(err) => err.set_data(json_data, fluent),
            UserAuthError::UserAccount(err) => err.set_data(json_data, fluent),
            UserAuthError::ValidCode(err) => err.set_data(json_data, fluent),
            UserAuthError::Redis(err) => err.set_data(json_data, fluent),
            UserAuthError::RedisPool(err) => err.set_data(json_data, fluent),
            UserAuthError::SerdeJson(err) => err.set_data(json_data, fluent),
            UserAuthError::Utf8Err(err) => err.set_data(json_data, fluent),
        }
    }
}

impl FluentJsonData for UserRbacError {
    fn set_data(&self, json_data: JsonData, fluent: &FluentBundle) -> JsonData {
        let json_data = json_data.set_sub_code("rbac").set_code(500);
        match self {
            UserRbacError::Sqlx(err) => err.set_data(json_data, fluent),
            UserRbacError::NotLogin(_) => json_data.set_code(403).set_sub_code("not_login"),
            UserRbacError::Check(err) => {
                let mut hash = HashMap::<&String, Vec<String>>::new();
                for (k, v) in err {
                    hash.entry(k).or_default().push(fluent.format_message(v));
                }
                json_data
                    .set_code(403)
                    .set_sub_code("check_fail")
                    .set_data(json!( {
                        "check_detail":hash,
                    }))
            }
            UserRbacError::System(_) => json_data,
        }
    }
}

impl FluentJsonData for SettingError {
    fn set_data(&self, json_data: JsonData, fluent: &FluentBundle) -> JsonData {
        let json_data = json_data.set_code(500).set_sub_code("setting");
        match self {
            SettingError::Sqlx(err) => err.set_data(json_data, fluent),
            SettingError::SerdeJson(err) => err.set_data(json_data, fluent),
        }
    }
}

impl FluentJsonData for SenderError {
    fn set_data(&self, json_data: JsonData, fluent: &FluentBundle) -> JsonData {
        let json_data = json_data.set_code(500).set_sub_code("sender");
        match self {
            SenderError::Sqlx(err) => err.set_data(json_data, fluent),
            SenderError::Redis(err) => err.set_data(json_data, fluent),
            SenderError::RedisPool(err) => err.set_data(json_data, fluent),
            SenderError::Tera(err) => err.set_data(json_data, fluent),
            SenderError::Setting(err) => err.set_data(json_data, fluent),
            SenderError::System(_) => json_data,
        }
    }
}

impl FluentJsonData for AppsError {
    fn set_data(&self, json_data: JsonData, fluent: &FluentBundle) -> JsonData {
        let json_data = json_data.set_code(500).set_sub_code("apps");
        match self {
            AppsError::ScopeNotFind(_) => json_data.set_sub_code("app-bad-scope"),
            AppsError::System(_) => json_data,
            AppsError::Sqlx(err) => err.set_data(json_data, fluent),
            AppsError::Redis(err) => err.set_data(json_data, fluent),
            AppsError::RedisPool(err) => err.set_data(json_data, fluent),
            AppsError::UserAccount(err) => err.set_data(json_data, fluent),
            AppsError::SerdeJson(err) => err.set_data(json_data, fluent),
        }
    }
}

impl FluentJsonData for ConfigError {
    fn set_data(&self, json_data: JsonData, fluent: &FluentBundle) -> JsonData {
        let json_data = json_data.set_code(500).set_sub_code("config");
        match self {
            ConfigError::Io(err) => err.set_data(json_data, fluent),
            ConfigError::Config(err) => err.set_data(json_data, fluent),
        }
    }
}

impl FluentJsonData for NotifyError {
    fn set_data(&self, json_data: JsonData, fluent: &FluentBundle) -> JsonData {
        match self {
            NotifyError::Sqlx(err) => err.set_data(json_data, fluent),
            NotifyError::Redis(err) => err.set_data(json_data, fluent),
            NotifyError::RedisPool(err) => err.set_data(json_data, fluent),
            NotifyError::System(_) => json_data.set_code(500).set_sub_code("notify"),
        }
    }
}

impl FluentJsonData for LoggerError {
    fn set_data(&self, json_data: JsonData, fluent: &FluentBundle) -> JsonData {
        match self {
            LoggerError::Sqlx(err) => err.set_data(json_data, fluent),
        }
    }
}

impl FluentJsonData for lsys_lib_area::AreaError {
    fn set_data(&self, json_data: JsonData, _: &FluentBundle) -> JsonData {
        match self {
            lsys_lib_area::AreaError::DB(_) => {
                json_data.set_code(500).set_sub_code("lsys_lib_area")
            }
            lsys_lib_area::AreaError::System(_) => json_data.set_code(500).set_sub_code("area"),
            lsys_lib_area::AreaError::NotFind(_) => json_data.set_sub_code("not_found"),
            lsys_lib_area::AreaError::Store(_) => json_data.set_sub_code("area_store"),
            lsys_lib_area::AreaError::Tantivy(_) => json_data.set_sub_code("area_tantivy"),
        }
    }
}

//lib error

impl FluentJsonData for sqlx::Error {
    fn set_data(&self, json_data: JsonData, _: &FluentBundle) -> JsonData {
        match self {
            sqlx::Error::RowNotFound => json_data.set_sub_code("not_found").set_code(404),
            _ => json_data.set_code(500).set_sub_code("sqlx"),
        }
    }
}
macro_rules! crate_error_fluent {
    ($crate_error:ty,$code:literal) => {
        impl FluentJsonData for $crate_error {
            fn set_data(&self, json_data: JsonData, _: &FluentBundle) -> JsonData {
                json_data.set_code(500).set_sub_code($code)
            }
        }
    };
}
crate_error_fluent!(config::ConfigError, "config");
crate_error_fluent!(std::io::Error, "io");
crate_error_fluent!(tera::Error, "tera");
crate_error_fluent!(redis::RedisError, "redis");
crate_error_fluent!(deadpool_redis::PoolError, "redis");
crate_error_fluent!(serde_json::Error, "serde");
crate_error_fluent!(ParseIntError, "parse");
crate_error_fluent!(std::string::FromUtf8Error, "parse");

#[cfg(feature = "docs")]
use lsys_docs::dao::GitDocError;
#[cfg(feature = "docs")]
crate_error_fluent!(lsys_docs::GitError, "git");
#[cfg(feature = "docs")]
impl FluentJsonData for GitDocError {
    fn set_data(&self, json_data: JsonData, fluent: &FluentBundle) -> JsonData {
        let json_data = json_data.set_code(500).set_sub_code("doc");
        match self {
            GitDocError::Sqlx(err) => err.set_data(json_data, fluent),
            GitDocError::Git(err) => err.set_data(json_data, fluent),
            GitDocError::System(_) => json_data,
            GitDocError::Remote(_) => json_data,
        }
    }
}
