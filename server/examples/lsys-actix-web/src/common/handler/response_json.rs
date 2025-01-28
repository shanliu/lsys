use actix::MailboxError;
use actix_http::StatusCode;
use actix_web::{
    body::BoxBody,
    cookie::{time::Duration, Cookie},
    error::{BlockingError, PayloadError},
    HttpMessage, HttpRequest, HttpResponse, Responder, ResponseError,
};
use lsys_core::now_time;
use lsys_user::dao::UserAuthToken;
use lsys_web::common::JsonData;
use serde_json::to_string_pretty;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::{error::Error, ops::Deref};
use tracing::warn;

use super::AUTH_COOKIE_NAME;

#[derive(Debug, Clone)]
pub struct ResponseJson {
    inner: JsonData,
}

pub type ResponseJsonResult<T> = Result<T, ResponseJson>;

impl Deref for ResponseJson {
    type Target = JsonData;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<JsonData> for ResponseJson {
    fn from(err: JsonData) -> Self {
        ResponseJson { inner: err }
    }
}

//success impl
impl Responder for ResponseJson {
    type Body = BoxBody;
    fn respond_to(self, req: &HttpRequest) -> HttpResponse {
        let mut res = HttpResponse::Ok().json(self.inner.to_value());
        if let Some(token) = req.extensions().get::<UserAuthToken>() {
            if !token.token.is_empty() {
                let now_t = now_time().unwrap_or_default();
                let age = if token.time_out > now_t {
                    token.time_out - now_t
                } else {
                    0
                };
                let cookie = Cookie::build(AUTH_COOKIE_NAME, token.token.clone())
                    //.domain("www.rust-lang.org")
                    //.secure(true)
                    .path("/")
                    .max_age(Duration::seconds(age as i64))
                    .http_only(true)
                    .finish();
                if let Err(e) = res.add_cookie(&cookie) {
                    warn!("auth add token fail:{}", e);
                }
            }
        }
        res
    }
}

//error impl
impl Display for ResponseJson {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(
            f,
            "{}",
            to_string_pretty(&self.inner.to_value()).unwrap_or_else(|e| {
                format!(
                    r#"{{
                    "result":{{
                        "code": "500",
                        "state": "system",
                        "message": "{}"
                    }},
                }}"#,
                    "display error:".to_owned() + &e.to_string()
                )
            })
        )
    }
}

impl ResponseError for ResponseJson {
    fn status_code(&self) -> StatusCode {
        StatusCode::OK
    }
}
impl Error for ResponseJson {}

macro_rules! result_impl_system_error {
    ($err_type:ty) => {
        impl From<$err_type> for ResponseJson {
            fn from(err: $err_type) -> Self {
                JsonData::message_error(err.to_string()).into()
            }
        }
    };
}

result_impl_system_error!(PayloadError);
result_impl_system_error!(actix_web::Error);
result_impl_system_error!(BlockingError);
result_impl_system_error!(MailboxError);
