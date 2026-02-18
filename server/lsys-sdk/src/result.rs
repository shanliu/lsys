use lsys_core::{fluent_message, FluentMessage, IntoFluentMessage};
use serde_json::Value;
use std::fmt;

use crate::types::ReqInfo;

/// Error details from API response
#[derive(Debug, Clone)]
pub struct ApiErrorDetail {
    pub code: u64,
    pub state: String,
    pub message: String,
    pub response: Option<Value>,
}

impl fmt::Display for ApiErrorDetail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "code={}, state={}, message={}",
            self.code, self.state, self.message
        )
    }
}

/// HTTP rejected error details
#[derive(Debug, Clone)]
pub struct HttpRejectedError {
    pub req: ReqInfo,
    pub status: u16,
    pub body: String,
}

/// API error details
#[derive(Debug, Clone)]
pub struct ApiError {
    pub req: ReqInfo,
    pub detail: ApiErrorDetail,
}

/// Parse error details
#[derive(Debug, Clone)]
pub struct ParseError {
    pub req: ReqInfo,
    pub message: String,
}

/// Service client error
#[derive(Debug)]
pub enum ServiceError {
    /// Invalid URL configuration
    InvalidUrl(String),

    /// HTTP request error
    Http(reqwest::Error),

    /// HTTP response with non-2xx status
    HttpRejected(Box<HttpRejectedError>),

    /// API business error (result.code != "200" or result.state != "ok")
    Api(Box<ApiError>),

    /// JSON parse error
    Parse(Box<ParseError>),
}

impl IntoFluentMessage for ServiceError {
    fn to_fluent_message(&self) -> FluentMessage {
        match self {
            ServiceError::InvalidUrl(url) => {
                fluent_message!("service-invalid-url", { "url": url })
            }
            ServiceError::Http(e) => {
                fluent_message!("service-http-error", e)
            }
            ServiceError::HttpRejected(e) => {
                fluent_message!("service-http-rejected", {
                    "method": &e.req.method,
                    "url": &e.req.url,
                    "status": e.status,
                    "body": &e.body
                })
            }
            ServiceError::Api(e) => {
                fluent_message!("service-api-error", {
                    "method": &e.req.method,
                    "url": &e.req.url,
                    "code": &e.detail.code,
                    "state": &e.detail.state,
                    "message": &e.detail.message
                })
            }
            ServiceError::Parse(e) => {
                fluent_message!("service-parse-error", {
                    "method": &e.req.method,
                    "url": &e.req.url,
                    "message": &e.message
                })
            }
        }
    }
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServiceError::InvalidUrl(url) => write!(f, "invalid url: {}", url),
            ServiceError::Http(e) => write!(f, "http error: {}", e),
            ServiceError::HttpRejected(e) => {
                write!(
                    f,
                    "{} {} rejected: status={}, body={}",
                    e.req.method, e.req.url, e.status, e.body
                )
            }
            ServiceError::Api(e) => {
                write!(f, "{} {} api error: {}", e.req.method, e.req.url, e.detail)
            }
            ServiceError::Parse(e) => {
                write!(
                    f,
                    "{} {} parse error: {}",
                    e.req.method, e.req.url, e.message
                )
            }
        }
    }
}

impl std::error::Error for ServiceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ServiceError::Http(e) => Some(e),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for ServiceError {
    fn from(e: reqwest::Error) -> Self {
        ServiceError::Http(e)
    }
}

pub type ServiceResult<T> = Result<T, ServiceError>;
