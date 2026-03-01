use http::HeaderMap;
use lsys_core::api_utils::{compute_rest_sign, RestSignData};
use lsys_core::fluent_message;
use lsys_core::fluents::{FluentMessage, IntoFluentMessage};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

use lsys_sdk::{
    AppFeatureResponse, AuthVerifyResponse, ForwardedRequest, ServiceClient, ServiceError,
};
use std::time::SystemTime;
/// 认证错误
#[derive(Debug)]
pub enum AuthError {
    /// 服务调用错误
    Service(ServiceError),
    /// 参数解析错误
    ParamParse(String),
    /// Payload JSON 解析错误
    PayloadParse(String),
    /// 签名验证失败
    SignInvalid(String),
    /// 功能未启用
    FeatureDisabled(String),
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthError::Service(e) => write!(f, "Service error: {}", e),
            AuthError::ParamParse(msg) => write!(f, "Param parse error: {}", msg),
            AuthError::PayloadParse(msg) => write!(f, "Payload parse error: {}", msg),
            AuthError::SignInvalid(msg) => write!(f, "Sign invalid: {}", msg),
            AuthError::FeatureDisabled(feature) => write!(f, "Feature '{}' disabled", feature),
        }
    }
}

impl std::error::Error for AuthError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AuthError::Service(e) => Some(e),
            _ => None,
        }
    }
}

impl IntoFluentMessage for AuthError {
    fn to_fluent_message(&self) -> FluentMessage {
        match self {
            AuthError::Service(e) => e.to_fluent_message(),
            AuthError::ParamParse(msg) => {
                fluent_message!("auth-param-parse-error", { "message": msg })
            }
            AuthError::PayloadParse(msg) => {
                fluent_message!("auth-payload-parse-error", { "message": msg })
            }
            AuthError::SignInvalid(msg) => fluent_message!("auth-sign-invalid", { "message": msg }),
            AuthError::FeatureDisabled(feature) => {
                fluent_message!("auth-feature-disabled", { "feature": feature })
            }
        }
    }
}

impl From<ServiceError> for AuthError {
    fn from(err: ServiceError) -> Self {
        AuthError::Service(err)
    }
}

pub type AuthResult<T> = Result<T, AuthError>;

/// REST 请求参数
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct RestQueryParams {
    pub client_id: String,
    pub version: String,
    pub timestamp: String,
    pub sign: String,
    pub payload: Option<String>,
    pub request_ip: Option<String>,
    pub method: Option<String>,
    pub token: Option<String>,
    pub lang: Option<String>,
}

#[derive(Clone)]
pub struct BarcodeClient {
    client: ServiceClient,
}

impl BarcodeClient {
    pub fn new(client: ServiceClient) -> Self {
        Self { client }
    }

    pub fn from_base_url(base: &str, api_key: &str) -> AuthResult<Self> {
        Ok(Self::new(ServiceClient::new(base, api_key)?))
    }

    /// Verify JWT and optionally check app feature
    ///
    /// 1. auth_verify 获取用户信息
    /// 2. 如果有 app_id，检查 barcode 功能是否启用
    pub async fn jwt_authorize(
        &self,
        incoming_headers: &HeaderMap,
        req: &JwtAuthorizeRequest,
    ) -> AuthResult<AuthVerifyResponse> {
        // Use SDK's unified method to extract forwarding headers
        let forward = ForwardedRequest::from_http_headers(incoming_headers);

        // Step 1: Verify JWT and get user info
        let auth_result = self.client.auth_verify(forward, None).await?;

        // Step 2: If app_id is provided, check barcode feature
        if let Some(app_id) = req.app_id {
            let feature_result = self.client.app_feature_check(app_id, &["barcode"]).await?;
            if !feature_result.enabled {
                return Err(AuthError::FeatureDisabled("barcode".to_string()));
            }
        }

        Ok(auth_result)
    }

    /// REST API 签名验证
    ///
    /// 1. 获取 app secret
    /// 2. 本地验证签名
    /// 3. 检查 barcode 功能是否启用
    pub async fn rest_authorize(
        &self,
        _incoming_headers: &HeaderMap,
        raw_query: &str,
    ) -> AuthResult<RestAuthorizeResponse> {
        // 解析查询参数
        let params: RestQueryParams = serde_urlencoded::from_str(raw_query)
            .map_err(|e| AuthError::ParamParse(e.to_string()))?;

        // Step 1: 获取 app secret
        let secret_result = self.client.app_secret(&params.client_id).await?;

        // 解析 payload
        let payload: Option<Value> = if let Some(ref pl) = params.payload {
            if !pl.is_empty() {
                Some(serde_json::from_str(pl).map_err(|e| AuthError::PayloadParse(e.to_string()))?)
            } else {
                None
            }
        } else {
            None
        };

        // 构建签名数据
        let sign_data = RestSignData {
            client_id: &params.client_id,
            version: &params.version,
            timestamp: &params.timestamp,
            request_ip: params.request_ip.as_deref(),
            method: params.method.as_deref(),
            token: params.token.as_deref(),
            payload: payload.as_ref(),
        };

        // 验证签名 - 尝试所有密钥
        let mut sign_matched = false;
        let mut last_result = None;
        for key in &secret_result.secrets {
            if key.time_out > 0
                && key.time_out
                    < SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .map(|e| e.as_secs())
                        .unwrap_or(0)
            {
                // 密钥已过期，跳过
                continue;
            }
            let result = compute_rest_sign(&sign_data, &key.secret_data);
            if result.signature == params.sign {
                sign_matched = true;
                break;
            }
            last_result = Some(result);
        }

        if !sign_matched {
            let computed = last_result.map(|r| r.signature).unwrap_or_default();
            tracing::debug!(
                "REST sign mismatch: computed={}, requested={}",
                computed,
                params.sign
            );
            return Err(AuthError::SignInvalid("Sign is wrong".to_string()));
        }

        // Step 3: 检查 app 是否启用了 barcode 功能
        let feature_result = self.app_feature_barcode(secret_result.app_id).await?;

        if !feature_result.enabled {
            return Err(AuthError::FeatureDisabled("barcode".to_string()));
        }

        Ok(RestAuthorizeResponse {
            app_id: secret_result.app_id,
            app_user_id: secret_result.user_id,
        })
    }

    /// Check app feature for barcode
    pub async fn app_feature_barcode(&self, app_id: u64) -> AuthResult<AppFeatureResponse> {
        Ok(self.client.app_feature_check(app_id, &["barcode"]).await?)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtAuthorizeRequest {
    pub action: String,
    pub app_id: Option<u64>,
    pub res_user_id: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct RestAuthorizeResponse {
    pub app_id: u64,
    pub app_user_id: u64,
}
