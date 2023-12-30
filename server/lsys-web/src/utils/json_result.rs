use config::ConfigError;
use deadpool_redis::PoolError;
use lsys_core::ValidCodeError;
use lsys_docs::dao::GitDocError;
use lsys_logger::dao::LoggerError;
use lsys_notify::dao::NotifyError;
use lsys_rbac::dao::rbac::UserRbacError;
use lsys_sender::dao::SenderError;
use lsys_setting::dao::SettingError;
use lsys_user::dao::{account::UserAccountError, auth::UserAuthError};
use serde_json::{json, Value};
use std::string::FromUtf8Error;
use std::{collections::HashMap, error::Error};
use tracing::warn;

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
impl JsonData {
    pub fn data(value: Value) -> Self {
        JsonData::default().set_message("ok").set_data(value)
    }
    pub fn error<T: Error>(error: T) -> Self {
        JsonData::message_error(format!("err:{}", error))
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

impl From<UserAuthError> for JsonData {
    fn from(err: UserAuthError) -> Self {
        let err_str = format!("{:?}", err);
        warn!("user auth error: {}", err_str);
        let mut out = JsonData::default()
            .set_code(200)
            .set_message(err.to_string());
        match err {
            UserAuthError::PasswordNotMatch(_) => out.set_sub_code("password_wrong"),
            UserAuthError::PasswordNotSet(_) => out.set_sub_code("password_empty"),
            UserAuthError::StatusError(_) => out.set_sub_code("status_wrong"),
            UserAuthError::UserNotFind(_) => out.set_sub_code("not_find"),
            UserAuthError::NotLogin(_) => out.set_sub_code("not_login"),
            UserAuthError::Sqlx(sqlx::Error::RowNotFound) => out.set_sub_code("not_found"),
            UserAuthError::Sqlx(_) => out.set_code(501).set_sub_code("sqlx"),
            UserAuthError::UserAccount(_) => out.set_sub_code("system"),
            UserAuthError::System(_) => out.set_code(500).set_sub_code("system"),
            UserAuthError::CheckCaptchaNeed(_) => out.set_sub_code("need_captcha"),
            UserAuthError::Redis(_) => out.set_code(502).set_sub_code("redis"),
            UserAuthError::CheckUserLock(_) => out.set_sub_code("user_lock"),
            UserAuthError::TokenParse(_) => out.set_sub_code("token_wrong"),
            UserAuthError::ValidCode(err) => {
                out = out.set_sub_code("valid_code");
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
        let mut code = 501;
        let sub_code = match &err {
            sqlx::Error::RowNotFound => {
                code = 200;
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
            UserAccountError::Sqlx(sqlx::Error::RowNotFound) => out.set_sub_code("not_found"),
            UserAccountError::ValidCode(err) => match err {
                ValidCodeError::DelayTimeout(err) => out.set_data(json!({
                    "type":err.prefix
                })),
                ValidCodeError::NotMatch(err) => out.set_data(json!({
                    "type":err.prefix
                })),
                _ => out,
            },
            UserAccountError::Param(_) => out.set_sub_code("param"),
            _err => out.set_sub_code("param"),
        }
    }
}

impl From<UserRbacError> for JsonData {
    fn from(err: UserRbacError) -> Self {
        let mut code = 500;
        let mut json = JsonData::default();
        let mut msg = err.to_string();
        let sub_code = match &err {
            UserRbacError::Sqlx(sqlx::Error::RowNotFound) => {
                code = 200;
                "not_found".to_string()
            }
            UserRbacError::NotLogin(_) => {
                code = 200;
                "not_login".to_string()
            }
            UserRbacError::Check(err) => {
                code = 200;
                let mut hash = HashMap::<&String, Vec<&String>>::new();
                for (k, v) in err {
                    hash.entry(k).or_default().push(v);
                }
                json = json.set_data(json!( {
                    "check_detail":hash,
                }));
                msg = "authorization verification failure".to_string();
                "check_fail".to_string()
            }
            _err => "system".to_string(),
        };
        json.set_code(code).set_sub_code(sub_code).set_message(msg)
    }
}

impl From<ValidCodeError> for JsonData {
    fn from(err: ValidCodeError) -> Self {
        JsonData::default()
            .set_sub_code("valid_code")
            .set_message(err.to_string())
    }
}

impl From<SenderError> for JsonData {
    fn from(err: SenderError) -> Self {
        match err {
            SenderError::Sqlx(err) => match err {
                sqlx::Error::RowNotFound => JsonData::default()
                    .set_code(200)
                    .set_sub_code("not_found")
                    .set_message(err.to_string()),
                _ => JsonData::default()
                    .set_code(500)
                    .set_sub_code("system")
                    .set_message(err.to_string()),
            },
            SenderError::Redis(err) => JsonData::default()
                .set_code(500)
                .set_sub_code("redis")
                .set_message(err),
            SenderError::System(err) => JsonData::default().set_sub_code("system").set_message(err),
            SenderError::Tpl(err) => JsonData::default()
                .set_sub_code("tpl")
                .set_message(format!("tpl error:{}", err)),
        }
    }
}

impl From<AppsError> for JsonData {
    fn from(err: AppsError) -> Self {
        match err {
            AppsError::Sqlx(err) => match err {
                sqlx::Error::RowNotFound => JsonData::default()
                    .set_code(200)
                    .set_sub_code("not_found")
                    .set_message(err.to_string()),
                _ => JsonData::default()
                    .set_code(500)
                    .set_sub_code("system")
                    .set_message(err.to_string()),
            },
            AppsError::System(_) => JsonData::default()
                .set_code(200)
                .set_sub_code("system")
                .set_message(err.to_string()),
            AppsError::Redis(err) => JsonData::default()
                .set_code(500)
                .set_sub_code("system")
                .set_message(err),
            AppsError::ScopeNotFind(err) => JsonData::default()
                .set_code(200)
                .set_sub_code("need_access")
                .set_message(err),
        }
    }
}

impl From<SettingError> for JsonData {
    fn from(err: SettingError) -> Self {
        match err {
            SettingError::Sqlx(err) => match err {
                sqlx::Error::RowNotFound => JsonData::default()
                    .set_code(200)
                    .set_sub_code("not_found")
                    .set_message(err.to_string()),
                _ => JsonData::default()
                    .set_code(500)
                    .set_sub_code("system")
                    .set_message(err.to_string()),
            },
            SettingError::System(_) => JsonData::default()
                .set_code(200)
                .set_sub_code("system")
                .set_message(err.to_string()),
        }
    }
}

impl From<NotifyError> for JsonData {
    fn from(err: NotifyError) -> Self {
        match err {
            NotifyError::Sqlx(err) => match err {
                sqlx::Error::RowNotFound => JsonData::default()
                    .set_code(200)
                    .set_sub_code("not_found")
                    .set_message(err.to_string()),
                _ => JsonData::default()
                    .set_code(500)
                    .set_sub_code("system")
                    .set_message(err.to_string()),
            },
            NotifyError::Redis(err) => JsonData::default()
                .set_code(200)
                .set_sub_code("system")
                .set_message(err),
            NotifyError::System(err) => JsonData::default()
                .set_code(200)
                .set_sub_code("system")
                .set_message(err),
        }
    }
}
impl From<GitDocError> for JsonData {
    fn from(err: GitDocError) -> Self {
        match err {
            GitDocError::Sqlx(err) => match err {
                sqlx::Error::RowNotFound => JsonData::default()
                    .set_code(200)
                    .set_sub_code("not_found")
                    .set_message(err.to_string()),
                _ => JsonData::default()
                    .set_code(500)
                    .set_sub_code("system")
                    .set_message(err.to_string()),
            },
            GitDocError::System(err) => JsonData::default()
                .set_code(200)
                .set_sub_code("system")
                .set_message(err),
            GitDocError::Redis(err) => JsonData::default()
                .set_code(200)
                .set_sub_code("system")
                .set_message(err),
            GitDocError::Remote(err) => JsonData::default()
                .set_code(200)
                .set_sub_code("system")
                .set_message(err),
        }
    }
}
impl From<LoggerError> for JsonData {
    fn from(err: LoggerError) -> Self {
        match err {
            LoggerError::Sqlx(err) => match err {
                sqlx::Error::RowNotFound => JsonData::default()
                    .set_code(200)
                    .set_sub_code("not_found")
                    .set_message(err.to_string()),
                _ => JsonData::default()
                    .set_code(500)
                    .set_sub_code("system")
                    .set_message(err.to_string()),
            },
            LoggerError::System(_) => JsonData::default()
                .set_code(200)
                .set_sub_code("system")
                .set_message(err.to_string()),
        }
    }
}

impl From<String> for JsonData {
    fn from(err: String) -> Self {
        JsonData::default().set_message(err)
    }
}

impl From<area_db::AreaError> for JsonData {
    fn from(err: area_db::AreaError) -> Self {
        match err {
            area_db::AreaError::DB(err) => JsonData::default()
                .set_code(500)
                .set_sub_code("system")
                .set_message(err),
            area_db::AreaError::System(err) => JsonData::default()
                .set_code(500)
                .set_sub_code("system")
                .set_message(err),
            area_db::AreaError::NotFind(_) => {
                JsonData::default().set_message("not find area record")
            }
            area_db::AreaError::Store(e) => {
                JsonData::default().set_message(format!("index area data fail:{}", e))
            }
            area_db::AreaError::Tantivy(e) => {
                JsonData::default().set_message(format!("tantivy area data fail:{}", e))
            }
        }
    }
}

macro_rules! result_impl_system_error {
    ($err_type:ty,$code:literal) => {
        impl From<$err_type> for JsonData {
            fn from(err: $err_type) -> Self {
                JsonData::default()
                    .set_code($code)
                    .set_sub_code("system")
                    .set_message(err.to_string())
            }
        }
    };
}

result_impl_system_error!(WebAppSmserError, 200);
result_impl_system_error!(WebAppMailerError, 200);
result_impl_system_error!(std::cell::BorrowError, 200);
result_impl_system_error!(serde_json::Error, 200);
result_impl_system_error!(FromUtf8Error, 500);
result_impl_system_error!(std::io::Error, 500);
result_impl_system_error!(reqwest::Error, 500);
result_impl_system_error!(tera::Error, 500);
result_impl_system_error!(PoolError, 500);
