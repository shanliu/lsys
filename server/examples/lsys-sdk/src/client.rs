use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue, USER_AGENT};
use serde::{Serialize, de::DeserializeOwned};
use url::Url;

use lsys_core::api_utils::{
    SERVICE_SIGNATURE_HEADER, SERVICE_TIMESTAMP_HEADER, compute_service_sign,
};

use crate::result::{
    ApiError, ApiErrorDetail, HttpRejectedError, ParseError, ServiceError, ServiceResult,
};
use crate::types::{
    ACCEPT_LANGUAGE_HEADER, DEVICE_ID_HEADER, FORWARDED_FOR_HEADER, ForwardedRequest,
    REQUEST_ID_HEADER, ReqInfo, ResponseEnvelope,
};

/// Service client for internal service-to-service communication
#[derive(Clone)]
pub struct ServiceClient {
    base_url: Url,
    api_key: String,
    http: reqwest::Client,
}

impl ServiceClient {
    /// Create a new service client
    ///
    /// # Arguments
    /// * `base_url` - Base URL of the upstream service (e.g., "http://127.0.0.1:8080")
    /// * `api_key` - API key for authentication (configured in upstream's app.toml as `service_api_key`)
    pub fn new(base_url: &str, api_key: &str) -> ServiceResult<Self> {
        let base_url =
            Url::parse(base_url).map_err(|_| ServiceError::InvalidUrl(base_url.to_string()))?;

        let http = reqwest::Client::builder()
            .build()
            .map_err(ServiceError::Http)?;

        Ok(Self {
            base_url,
            api_key: api_key.to_string(),
            http,
        })
    }

    /// Build endpoint URL from path
    pub(crate) fn endpoint(&self, path: &str) -> ServiceResult<Url> {
        self.base_url
            .join(path)
            .map_err(|_| ServiceError::InvalidUrl(format!("{}{}", self.base_url, path)))
    }

    /// Create a POST request
    pub fn post(&self, path: &str) -> ServiceResult<ServiceRequest<'_>> {
        let url = self.endpoint(path)?;
        Ok(ServiceRequest {
            client: self,
            method: reqwest::Method::POST,
            url,
            forward: None,
            body: None,
        })
    }

    /// Create a GET request
    pub fn get(&self, path: &str) -> ServiceResult<ServiceRequest<'_>> {
        let url = self.endpoint(path)?;
        Ok(ServiceRequest {
            client: self,
            method: reqwest::Method::GET,
            url,
            forward: None,
            body: None,
        })
    }

    /// 通过上传令牌直传一个文件分片（multipart）。
    ///
    /// 该端点仅凭 `upload_token` 鉴权（令牌即身份），无需服务签名。
    ///
    /// # Arguments
    /// * `path` - 令牌上传端点路径，如 `/service/file/upload_by_token` 或 `/rest/file/upload_by_token`
    /// * `upload_token` - 由 upload_create / upload_retoken 返回的令牌
    /// * `chunk_index` - 分片索引（单文件传 0）
    /// * `data` - 该分片的二进制数据
    pub async fn upload_by_token<T: DeserializeOwned>(
        &self,
        path: &str,
        upload_token: &str,
        chunk_index: u32,
        data: Vec<u8>,
    ) -> ServiceResult<T> {
        let url = self.endpoint(path)?;
        let req_info = ReqInfo {
            method: "POST".to_string(),
            url: url.to_string(),
        };

        let form = reqwest::multipart::Form::new()
            .text("upload_token", upload_token.to_string())
            .text("chunk_index", chunk_index.to_string())
            .part(
                "file",
                reqwest::multipart::Part::bytes(data).file_name("file"),
            );

        let response = self
            .http
            .post(url)
            .multipart(form)
            .send()
            .await?;
        let status = response.status();
        let body_text = response.text().await?;

        if !status.is_success() {
            return Err(ServiceError::HttpRejected(Box::new(HttpRejectedError {
                req: req_info,
                status: status.as_u16(),
                body: body_text,
            })));
        }

        let envelope: ResponseEnvelope = serde_json::from_str(&body_text).map_err(|e| {
            ServiceError::Parse(Box::new(ParseError {
                req: req_info.clone(),
                message: format!("invalid json: {}; raw={}", e, body_text),
            }))
        })?;

        if !envelope.result.is_ok() {
            return Err(ServiceError::Api(Box::new(ApiError {
                req: req_info,
                detail: ApiErrorDetail {
                    code: envelope.result.code,
                    state: envelope.result.state,
                    message: envelope.result.message,
                    response: envelope.response,
                },
            })));
        }

        let value = envelope.response.unwrap_or(serde_json::Value::Null);
        serde_json::from_value(value).map_err(|e| {
            ServiceError::Parse(Box::new(ParseError {
                req: req_info,
                message: format!("invalid response: {}", e),
            }))
        })
    }
}

/// Request builder for service calls
pub struct ServiceRequest<'a> {
    client: &'a ServiceClient,
    method: reqwest::Method,
    url: Url,
    forward: Option<ForwardedRequest>,
    body: Option<serde_json::Value>,
}

impl<'a> ServiceRequest<'a> {
    /// Build request info for error/debug context
    pub fn req_info(&self) -> ReqInfo {
        ReqInfo {
            method: self.method.to_string(),
            url: self.url.to_string(),
        }
    }

    /// Set forwarded request information
    pub fn forward(mut self, info: ForwardedRequest) -> Self {
        self.forward = Some(info);
        self
    }

    /// Set JSON body
    pub fn json<T: Serialize>(mut self, body: &T) -> Self {
        self.body = serde_json::to_value(body).ok();
        self
    }

    /// Send request and parse response
    pub async fn send(self) -> ServiceResult<serde_json::Value> {
        let req_info = self.req_info();
        let mut headers = HeaderMap::new();

        // 生成时间戳和签名
        let sign_result = compute_service_sign(&self.client.api_key, None);

        // Timestamp + Signature authentication
        headers.insert(
            SERVICE_TIMESTAMP_HEADER,
            HeaderValue::from_str(&sign_result.timestamp)
                .unwrap_or_else(|_| HeaderValue::from_static("0")),
        );
        headers.insert(
            SERVICE_SIGNATURE_HEADER,
            HeaderValue::from_str(&sign_result.signature)
                .unwrap_or_else(|_| HeaderValue::from_static("")),
        );

        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        // Forward headers if provided
        if let Some(ref fwd) = self.forward {
            if let Some(ref auth) = fwd.authorization
                && let Ok(val) = HeaderValue::from_str(auth)
            {
                headers.insert(AUTHORIZATION, val);
            }
            if let Some(ref lang) = fwd.accept_language
                && let Ok(val) = HeaderValue::from_str(lang)
            {
                headers.insert(ACCEPT_LANGUAGE_HEADER, val);
            }
            if let Some(ref ua) = fwd.user_agent
                && let Ok(val) = HeaderValue::from_str(ua)
            {
                headers.insert(USER_AGENT, val);
            }
            if let Some(ref rid) = fwd.request_id
                && let Ok(val) = HeaderValue::from_str(rid)
            {
                headers.insert(REQUEST_ID_HEADER, val);
            }
            if let Some(ref did) = fwd.device_id
                && let Ok(val) = HeaderValue::from_str(did)
            {
                headers.insert(DEVICE_ID_HEADER, val);
            }
            if let Some(ref ip) = fwd.client_ip
                && let Ok(val) = HeaderValue::from_str(ip)
            {
                headers.insert(FORWARDED_FOR_HEADER, val);
            }
        }

        let mut builder = self
            .client
            .http
            .request(self.method.clone(), self.url.clone());
        builder = builder.headers(headers);

        if let Some(body) = self.body {
            builder = builder.json(&body);
        }

        let response = builder.send().await?;
        let status = response.status();
        let body_text = response.text().await?;

        if !status.is_success() {
            return Err(ServiceError::HttpRejected(Box::new(HttpRejectedError {
                req: req_info,
                status: status.as_u16(),
                body: body_text,
            })));
        }

        // Parse response envelope
        let envelope: ResponseEnvelope = serde_json::from_str(&body_text).map_err(|e| {
            ServiceError::Parse(Box::new(ParseError {
                req: req_info.clone(),
                message: format!("invalid json: {}; raw={}", e, body_text),
            }))
        })?;

        // Check result
        if !envelope.result.is_ok() {
            return Err(ServiceError::Api(Box::new(ApiError {
                req: req_info,
                detail: ApiErrorDetail {
                    code: envelope.result.code,
                    state: envelope.result.state,
                    message: envelope.result.message,
                    response: envelope.response,
                },
            })));
        }

        Ok(envelope.response.unwrap_or(serde_json::Value::Null))
    }

    /// Send request and parse response as typed JSON
    ///
    /// This is a convenience method that combines send() with JSON deserialization.
    /// It automatically includes method/url info in parse errors.
    pub async fn send_json<T: DeserializeOwned>(self) -> ServiceResult<T> {
        let req_info = self.req_info();

        let value = self.send().await?;

        serde_json::from_value(value).map_err(|e| {
            ServiceError::Parse(Box::new(ParseError {
                req: req_info,
                message: format!("invalid response: {}", e),
            }))
        })
    }
}
