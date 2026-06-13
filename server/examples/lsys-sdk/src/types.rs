use reqwest::header::HeaderMap;
use serde::Deserialize;

// Header constants for forwarding - use these to ensure consistency
pub const FORWARDED_FOR_HEADER: &str = "X-Forwarded-For";
pub const REQUEST_ID_HEADER: &str = "X-Request-ID";
pub const DEVICE_ID_HEADER: &str = "X-Device-ID";
pub const ACCEPT_LANGUAGE_HEADER: &str = "Accept-Language";

/// Request info for error/debug context
#[derive(Clone, Debug)]
pub struct ReqInfo {
    pub method: String,
    pub url: String,
}

/// Forwarded request information from the original client
///
/// This struct contains headers and metadata that should be forwarded
/// from the original client request to the upstream service.
///
/// The caller (e.g., axum/actix-web handler) is responsible for
/// extracting these values from their framework's request type.
#[derive(Debug, Clone, Default)]
pub struct ForwardedRequest {
    /// Auth token from Authorization header (without "Bearer " prefix)
    pub authorization: Option<String>,

    /// Accept-Language header value
    pub accept_language: Option<String>,

    /// User-Agent header value
    pub user_agent: Option<String>,

    /// X-Request-ID header value
    pub request_id: Option<String>,

    /// X-Device-ID header value
    pub device_id: Option<String>,

    /// Original client IP address
    pub client_ip: Option<String>,
}

impl ForwardedRequest {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create ForwardedRequest by extracting values from HTTP headers
    ///
    /// This method extracts all relevant headers for forwarding to upstream services.
    /// Use this instead of manually constructing ForwardedRequest to ensure
    /// header names stay consistent with the SDK.
    ///
    /// # Example
    /// ```ignore
    /// // In axum handler:
    /// let forward = ForwardedRequest::from_http_headers(&headers);
    /// let result = client.auth_verify(forward, None).await?;
    /// ```
    pub fn from_http_headers(headers: &HeaderMap) -> Self {
        Self {
            authorization: headers
                .get(reqwest::header::AUTHORIZATION)
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string()),
            accept_language: headers
                .get(ACCEPT_LANGUAGE_HEADER)
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string()),
            user_agent: headers
                .get(reqwest::header::USER_AGENT)
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string()),
            request_id: headers
                .get(REQUEST_ID_HEADER)
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string()),
            device_id: headers
                .get(DEVICE_ID_HEADER)
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string()),
            client_ip: headers
                .get(FORWARDED_FOR_HEADER)
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.split(',').next())
                .map(|s| s.trim().to_string()),
        }
    }

    pub fn authorization(mut self, value: impl Into<String>) -> Self {
        self.authorization = Some(value.into());
        self
    }

    pub fn accept_language(mut self, value: impl Into<String>) -> Self {
        self.accept_language = Some(value.into());
        self
    }

    pub fn user_agent(mut self, value: impl Into<String>) -> Self {
        self.user_agent = Some(value.into());
        self
    }

    pub fn request_id(mut self, value: impl Into<String>) -> Self {
        self.request_id = Some(value.into());
        self
    }

    pub fn device_id(mut self, value: impl Into<String>) -> Self {
        self.device_id = Some(value.into());
        self
    }

    pub fn client_ip(mut self, value: impl Into<String>) -> Self {
        self.client_ip = Some(value.into());
        self
    }
}

/// Response envelope from lsys-web services
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ResponseEnvelope {
    pub result: ResponseResult,
    pub response: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ResponseResult {
    #[serde(deserialize_with = "crate::utils::deserialize_u64_from_string")]
    pub code: u64,
    pub state: String,
    pub message: String,
}

impl ResponseResult {
    pub fn is_ok(&self) -> bool {
        self.code == 200 && self.state == "ok"
    }
}
