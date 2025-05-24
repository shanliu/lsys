mod request_auth;
mod request_json;
mod request_jwt;
mod request_query;
mod request_query_get;
mod request_rest;
mod response_json;
use actix_web::error::ResponseError;
pub use request_auth::*;
pub use request_json::*;
pub use request_jwt::*;
pub use request_query::*;
pub use request_rest::*;
pub use response_json::*;
use std::fmt::{Display, Formatter, Result as FmtResult};

use std::error::Error;

const AUTH_COOKIE_NAME: &str = "token-auth";

//统一错误
#[derive(Debug)]
pub struct WebHandError {
    message: String,
}

impl WebHandError {
    pub fn string(msg: String) -> WebHandError {
        WebHandError { message: msg }
    }
}
impl Display for WebHandError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.message)
    }
}
impl From<lsys_web::tera::Error> for WebHandError {
    fn from(err: lsys_web::tera::Error) -> Self {
        WebHandError::string(format!("{:?}", err))
    }
}
impl ResponseError for WebHandError {}
impl Error for WebHandError {}
