use config::ConfigError;
use lsys_core::ValidCodeError;
use lsys_rbac::dao::rbac::UserRbacError;
use lsys_user::dao::{account::UserAccountError, auth::UserAuthError};
use serde_json::{json, Value};
use std::string::FromUtf8Error;
use std::{collections::HashMap, error::Error};
use tracing::warn;

use crate::dao::access::ScopeError;
use crate::dao::{WebAppMailerError, WebAppSmserError};
use lsys_app::dao::app::AppsError;

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
    pub fn data(value: Value) -> Self {
        JsonData::default().set_data(value)
    }
    pub fn error<T: Error>(error: T) -> Self {
        JsonData::message_error(format!("err:{}", error))
    }
    pub fn message_error<T: ToString>(msg: T) -> Self {
        JsonData::message(msg).set_code(500)
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

impl From<UserAuthError> for JsonData {
    fn from(err: UserAuthError) -> Self {
        let err_str = format!("{:?}", err);
        warn!("user auth error: {}", err_str);
        let mut out = JsonData::default()
            .set_code(200)
            .set_message(err.to_string());
        match err {
            UserAuthError::PasswordNotMatch(_) => out.set_code(402).set_sub_code("password_wrong"),
            UserAuthError::PasswordNotSet(_) => out.set_code(405).set_sub_code("password_empty"),
            UserAuthError::StatusError(_) => out.set_code(405).set_sub_code("status_wrong"),
            UserAuthError::UserNotFind(_) => out.set_code(405).set_sub_code("not_find"),
            UserAuthError::NotLogin(_) => out.set_code(405).set_sub_code("not_login"),
            UserAuthError::Sqlx(_) => out.set_code(405).set_sub_code("sqlx"),
            UserAuthError::UserAccount(_) => out.set_code(405).set_sub_code("system"),
            UserAuthError::System(_) => out.set_code(405).set_sub_code("system"),
            UserAuthError::CheckCaptchaNeed(_) => out.set_code(405).set_sub_code("need_captcha"),
            UserAuthError::Redis(_) => out.set_code(405).set_sub_code("redis"),
            UserAuthError::CheckUserLock(_) => out.set_code(403).set_sub_code("user_lock"),
            UserAuthError::TokenParse(_) => out.set_code(403).set_sub_code("token_wrong"),
            UserAuthError::ValidCode(err) => {
                out = out.set_code(410).set_sub_code("valid_code");
                match err {
                    ValidCodeError::DelayTimeout(err) => out.set_data(json!({
                        "type":err.prefix
                    })),
                    ValidCodeError::NotMatch(err) => out.set_data(json!({
                        "type":err.prefix
                    })),
                    _ => out,
                }
            }
        }
    }
}

impl From<sqlx::Error> for JsonData {
    fn from(err: sqlx::Error) -> Self {
        let mut code = 500;
        let sub_code = match &err {
            sqlx::Error::RowNotFound => {
                code = 404;
                "not_found"
            }
            _err => "system",
        };
        JsonData::default()
            .set_code(code)
            .set_sub_code(sub_code)
            .set_message(err.to_string())
    }
}
impl From<ConfigError> for JsonData {
    fn from(err: ConfigError) -> Self {
        JsonData::default()
            .set_code(503)
            .set_sub_code("config")
            .set_message(err.to_string())
    }
}
impl From<UserAccountError> for JsonData {
    fn from(err: UserAccountError) -> Self {
        let out = JsonData::default()
            .set_code(200)
            .set_message(err.to_string());
        match &err {
            UserAccountError::Sqlx(sqlx::Error::RowNotFound) => {
                out.set_code(404).set_sub_code("not_found")
            }
            UserAccountError::ValidCode(err) => match err {
                ValidCodeError::DelayTimeout(err) => out.set_data(json!({
                    "type":err.prefix
                })),
                ValidCodeError::NotMatch(err) => out.set_data(json!({
                    "type":err.prefix
                })),
                _ => out,
            },
            UserAccountError::Param(_) => out.set_code(404).set_sub_code("param"),
            _err => out.set_code(404).set_sub_code("param"),
        }
    }
}

impl From<UserRbacError> for JsonData {
    fn from(err: UserRbacError) -> Self {
        let mut code = 500;
        let mut json = JsonData::default();
        let sub_code = match &err {
            UserRbacError::Sqlx(sqlx::Error::RowNotFound) => {
                code = 404;
                "not_found".to_string()
            }
            UserRbacError::NotLogin(err) => {
                code = 403;
                err.to_owned()
            }
            UserRbacError::Check(err) => {
                code = 401;
                let mut hash = HashMap::<&String, Vec<&String>>::new();
                for (k, v) in err {
                    hash.entry(k).or_default().push(v);
                }
                json = json.set_data(json!( {
                    "check_detail":hash,
                }));
                "check_fail".to_string()
            }
            _err => "system".to_string(),
        };
        json.set_code(code).set_sub_code(sub_code).set_message(err)
    }
}

impl From<ValidCodeError> for JsonData {
    fn from(err: ValidCodeError) -> Self {
        JsonData::default()
            .set_code(400)
            .set_sub_code("valid_code")
            .set_message(err.to_string())
    }
}

macro_rules! result_impl_system_error {
    ($err_type:ty) => {
        impl From<$err_type> for JsonData {
            fn from(err: $err_type) -> Self {
                JsonData::default()
                    .set_code(500)
                    .set_sub_code("system")
                    .set_message(err.to_string())
            }
        }
    };
}
result_impl_system_error!(AppsError);
result_impl_system_error!(WebAppSmserError);
result_impl_system_error!(WebAppMailerError);
result_impl_system_error!(std::cell::BorrowError);
result_impl_system_error!(serde_json::Error);
result_impl_system_error!(FromUtf8Error);
result_impl_system_error!(std::io::Error);
result_impl_system_error!(reqwest::Error);
result_impl_system_error!(tera::Error);
result_impl_system_error!(ScopeError);
