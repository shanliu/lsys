use lsys_access::dao::AccessError;
use lsys_app_notify::dao::NotifyError;
use lsys_app_sender::dao::SenderError;
use lsys_core::{ConfigError, FluentBundle, FluentMessage, ValidCodeError};

use lsys_logger::dao::LoggerError;
use lsys_rbac::dao::RbacError;
use lsys_setting::dao::SettingError;
use lsys_user::dao::{AccountError, UserAuthError};
use serde_json::{json, Value};

use std::{collections::HashMap, num::ParseIntError};

use lsys_app::dao::AppError;

pub trait FluentJsonData: FluentFormat + 'static {
    fn to_json_data(&self, fluent: &FluentBundle) -> JsonData;
}

pub enum JsonError {
    Error(Box<dyn FluentJsonData>),
    Message(FluentMessage),
    JsonData(JsonData, FluentMessage),
}

impl<T: FluentJsonData> From<T> for JsonError {
    fn from(value: T) -> Self {
        Self::Error(Box::new(value))
    }
}
impl JsonError {
    pub fn to_json_data(&self, fluent: &FluentBundle) -> JsonData {
        match self {
            JsonError::Error(fluent_error_json_data) => fluent_error_json_data
                .to_json_data(fluent)
                .set_message(fluent_error_json_data.fluent_format(fluent)),
            JsonError::Message(_) => JsonData::default().set_code(500).set_sub_code("system"),
            JsonError::JsonData(json_data, message) => json_data
                .to_owned()
                .set_message(fluent.format_message(message)),
        }
    }
}

pub type JsonResult<T> = Result<T, JsonError>;

#[derive(Debug, Clone)]
pub struct JsonData {
    pub(crate) code: String,
    pub(crate) sub_code: String,
    pub(crate) message: String,
    pub(crate) data: Option<Value>,
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

impl FluentJsonData for AccountError {
    fn to_json_data(&self, fluent: &FluentBundle) -> JsonData {
        let json_data = JsonData::default()
            .set_code(500)
            .set_sub_code("user_account");
        match self {
            AccountError::Param(_) => json_data,
            AccountError::System(_) => json_data,
            AccountError::Status(_) => json_data,
            AccountError::Redis(err) => err.to_json_data(fluent),
            AccountError::RedisPool(err) => err.to_json_data(fluent),
            AccountError::ValidCode(err) => err.to_json_data(fluent),
            AccountError::Sqlx(err) => err.to_json_data(fluent),
            AccountError::Setting(err) => err.to_json_data(fluent),
            AccountError::SerdeJson(err) => err.to_json_data(fluent),

            AccountError::UserAuthError(err) => err.to_json_data(fluent),
            AccountError::AccessError(err) => err.to_json_data(fluent),

            AccountError::PasswordNotMatch((_, _)) => json_data.set_sub_code("password_wrong"),
            AccountError::PasswordNotSet(_) => json_data.set_sub_code("password_empty"),
            AccountError::AuthStatusError(_) => json_data.set_sub_code("status_wrong"),
            AccountError::UserNotFind(_) => json_data.set_sub_code("not_find"),
        }
    }
}

impl FluentJsonData for ValidCodeError {
    fn to_json_data(&self, fluent: &FluentBundle) -> JsonData {
        let json_data = JsonData::default().set_code(500);
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
            ValidCodeError::Redis(err) => err.to_json_data(fluent),
            ValidCodeError::RedisPool(err) => err.to_json_data(fluent),
        }
    }
}

impl FluentJsonData for UserAuthError {
    fn to_json_data(&self, fluent: &FluentBundle) -> JsonData {
        let json_data = JsonData::default().set_code(500);
        match self {
            UserAuthError::NotLogin(_) => json_data.set_sub_code("not_login"),
            UserAuthError::System(_) => json_data.set_sub_code("auth"),
            UserAuthError::CheckCaptchaNeed(_) => json_data.set_sub_code("need_captcha"),
            UserAuthError::CheckUserLock(_) => json_data.set_sub_code("user_lock"),
            UserAuthError::TokenParse(_) => json_data.set_sub_code("token_wrong"),
            UserAuthError::Sqlx(err) => err.to_json_data(fluent),
            UserAuthError::ValidCode(err) => err.to_json_data(fluent),
            UserAuthError::Redis(err) => err.to_json_data(fluent),
            UserAuthError::RedisPool(err) => err.to_json_data(fluent),
            UserAuthError::Utf8Err(err) => err.to_json_data(fluent),
            UserAuthError::AccessError(err) => err.to_json_data(fluent),
        }
    }
}

impl FluentJsonData for AccessError {
    fn to_json_data(&self, fluent: &FluentBundle) -> JsonData {
        let json_data = JsonData::default().set_sub_code("access").set_code(500);
        match self {
            AccessError::Sqlx(err) => err.to_json_data(fluent),
            AccessError::Redis(err) => err.to_json_data(fluent),
            AccessError::RedisPool(err) => err.to_json_data(fluent),
            AccessError::NotLogin => json_data.set_code(403).set_sub_code("not_login"),
            AccessError::IsLogout => json_data.set_code(403).set_sub_code("not_login"),
            AccessError::System(_) => json_data,
            AccessError::SerdeJson(err) => err.to_json_data(fluent),
            AccessError::BadAccount(_) => json_data.set_code(400).set_sub_code("bad_account"),
        }
    }
}
impl FluentJsonData for RbacError {
    fn to_json_data(&self, fluent: &FluentBundle) -> JsonData {
        let json_data = JsonData::default().set_sub_code("rbac").set_code(500);
        match self {
            RbacError::Sqlx(err) => err.to_json_data(fluent),
            RbacError::Check(err) => {
                let mut hash = HashMap::<String, Vec<String>>::new();
                for tmp in err {
                    let key = if tmp.res_data.is_empty() {
                        tmp.res_type.to_string()
                    } else {
                        format!("{}-{}", tmp.res_type, tmp.res_data)
                    };
                    hash.entry(key)
                        .or_default()
                        .push(fluent.format_message(&tmp.msg));
                }
                json_data
                    .set_code(403)
                    .set_sub_code("check_fail")
                    .set_data(json!( {
                        "check_detail":hash,
                    }))
            }
            RbacError::System(_) => json_data,
        }
    }
}

impl FluentJsonData for SettingError {
    fn to_json_data(&self, fluent: &FluentBundle) -> JsonData {
        match self {
            SettingError::Sqlx(err) => err.to_json_data(fluent),
            SettingError::SerdeJson(err) => err.to_json_data(fluent),
        }
        .set_code(500)
        .set_sub_code("setting")
    }
}

impl FluentJsonData for SenderError {
    fn to_json_data(&self, fluent: &FluentBundle) -> JsonData {
        let json_data = JsonData::default().set_code(500).set_sub_code("sender");
        match self {
            SenderError::Sqlx(err) => err.to_json_data(fluent),
            SenderError::Redis(err) => err.to_json_data(fluent),
            SenderError::RedisPool(err) => err.to_json_data(fluent),
            SenderError::Tera(err) => err.to_json_data(fluent),
            SenderError::Setting(err) => err.to_json_data(fluent),
            SenderError::System(_) => json_data,
        }
    }
}

impl FluentJsonData for AppError {
    fn to_json_data(&self, fluent: &FluentBundle) -> JsonData {
        let json_data = JsonData::default().set_code(500).set_sub_code("apps");
        match self {
            AppError::ScopeBad(_) => json_data.set_sub_code("app-bad-scope"),
            AppError::System(_) => json_data,
            AppError::Sqlx(err) => err.to_json_data(fluent),
            AppError::Redis(err) => err.to_json_data(fluent),
            AppError::RedisPool(err) => err.to_json_data(fluent),
            AppError::SerdeJson(err) => err.to_json_data(fluent),
            AppError::Access(err) => err.to_json_data(fluent),
            AppError::AppNotFound(_) => json_data.set_sub_code("app-not-found"),
            AppError::AppBadStatus => json_data.set_sub_code("app-bad-status"),
            AppError::AppBadFeature(_, _) => json_data.set_sub_code("app-bad-feature"),
            AppError::AppOAuthClientBadConfig(_) => json_data,
        }
    }
}

impl FluentJsonData for ConfigError {
    fn to_json_data(&self, fluent: &FluentBundle) -> JsonData {
        match self {
            ConfigError::Io(err) => err.to_json_data(fluent),
            ConfigError::Config(err) => err.to_json_data(fluent),
        }
        .set_code(500)
        .set_sub_code("config")
    }
}

impl FluentJsonData for NotifyError {
    fn to_json_data(&self, fluent: &FluentBundle) -> JsonData {
        match self {
            NotifyError::Sqlx(err) => err.to_json_data(fluent),
            NotifyError::Redis(err) => err.to_json_data(fluent),
            NotifyError::RedisPool(err) => err.to_json_data(fluent),
            NotifyError::System(_) => JsonData::default().set_code(500).set_sub_code("notify"),
        }
    }
}

impl FluentJsonData for LoggerError {
    fn to_json_data(&self, fluent: &FluentBundle) -> JsonData {
        match self {
            LoggerError::Sqlx(err) => err.to_json_data(fluent),
        }
    }
}
#[cfg(feature = "area")]
impl FluentJsonData for lsys_lib_area::AreaError {
    fn to_json_data(&self, _: &FluentBundle) -> JsonData {
        match self {
            lsys_lib_area::AreaError::DB(_) => JsonData::default()
                .set_code(500)
                .set_sub_code("lsys_lib_area"),
            lsys_lib_area::AreaError::System(_) => {
                JsonData::default().set_code(500).set_sub_code("area")
            }
            lsys_lib_area::AreaError::NotFind(_) => JsonData::default().set_sub_code("not_found"),
            lsys_lib_area::AreaError::Store(_) => JsonData::default().set_sub_code("area_store"),
            lsys_lib_area::AreaError::Tantivy(_) => {
                JsonData::default().set_sub_code("area_tantivy")
            }
        }
    }
}

#[cfg(feature = "barcode")]
impl FluentJsonData for lsys_app_barcode::dao::BarCodeError {
    fn to_json_data(&self, fluent: &FluentBundle) -> JsonData {
        match self {
            lsys_app_barcode::dao::BarCodeError::System(_) => JsonData::default()
                .set_code(500)
                .set_sub_code("app_barcode"),
            lsys_app_barcode::dao::BarCodeError::DB(err) => err.to_json_data(fluent),
            lsys_app_barcode::dao::BarCodeError::RXing(_) => {
                JsonData::default().set_code(500).set_sub_code("rxing")
            }
            lsys_app_barcode::dao::BarCodeError::Io(err) => err.to_json_data(fluent),
            lsys_app_barcode::dao::BarCodeError::Image(_) => {
                JsonData::default().set_code(500).set_sub_code("image")
            }
        }
    }
}

//lib error

impl FluentJsonData for sqlx::Error {
    fn to_json_data(&self, _: &FluentBundle) -> JsonData {
        match self {
            sqlx::Error::RowNotFound => JsonData::default().set_sub_code("not_found").set_code(404),
            _ => JsonData::default().set_code(500).set_sub_code("sqlx"),
        }
    }
}
macro_rules! crate_error_fluent {
    ($crate_error:ty,$code:literal) => {
        impl FluentJsonData for $crate_error {
            fn to_json_data(&self, _: &FluentBundle) -> JsonData {
                JsonData::default().set_code(500).set_sub_code($code)
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

use super::FluentFormat;

#[cfg(feature = "docs")]
crate_error_fluent!(lsys_docs::GitError, "git");
#[cfg(feature = "docs")]
impl FluentJsonData for GitDocError {
    fn to_json_data(&self, fluent: &FluentBundle) -> JsonData {
        let json_data = JsonData::default().set_code(500).set_sub_code("doc");
        match self {
            GitDocError::Sqlx(err) => err.to_json_data(fluent),
            GitDocError::Git(err) => err.to_json_data(fluent),
            GitDocError::System(_) => json_data,
            GitDocError::Remote(_) => json_data,
        }
    }
}
