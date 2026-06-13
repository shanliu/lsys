use actix_utils::future::{Ready, err, ok};
use actix_web::{FromRequest, HttpRequest, dev::Payload, web::Data};
use lsys_web::lsys_access::dao::AccessSession;
use lsys_web::lsys_core::api_utils::{
    SERVICE_SIGNATURE_HEADER, SERVICE_TIMESTAMP_HEADER, compute_service_sign,
};
use lsys_web::lsys_user::dao::{UserAuthSession, UserAuthToken};
use lsys_web::{common::{JsonData, JsonResponse, RequestAuthDao}, dao::WebDao};
use std::ops::Deref;
use std::str::FromStr;

use super::ResponseJson;
use super::request_token::{TokenSignConfig, verify_token};

/// 时间戳有效期（秒）
const TIMESTAMP_TOLERANCE_SECS: i64 = 3600; // 1小时

/// Service request query extractor
///
/// 验证 X-Timestamp + X-Signature 头部，用于服务间调用
///
/// 签名验证: X-Signature = MD5(service_api_key + X-Timestamp)
/// 时间戳验证: X-Timestamp 必须在 1小时
pub struct ServiceQuery {
    inner:
        RequestAuthDao<UserAuthToken, lsys_web::lsys_user::dao::UserAuthData, UserAuthSession>,
    bearer_token: Option<String>,
}

impl Deref for ServiceQuery {
    type Target =
        RequestAuthDao<UserAuthToken, lsys_web::lsys_user::dao::UserAuthData, UserAuthSession>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl FromRequest for ServiceQuery {
    type Error = ResponseJson;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let web_dao_opt = req.app_data::<Data<WebDao>>();

        let web_dao = match web_dao_opt {
            Some(dao) => dao,
            None => {
                return err(JsonResponse::data(JsonData::error())
                    .set_message("not find webdao")
                    .into());
            }
        };

        // Get headers
        let timestamp = req
            .headers()
            .get(SERVICE_TIMESTAMP_HEADER)
            .and_then(|v| v.to_str().ok());

        let signature = req
            .headers()
            .get(SERVICE_SIGNATURE_HEADER)
            .and_then(|v| v.to_str().ok());

        let service_api_key = web_dao
            .app_core
            .config
            .find(None)
            .get_string("service_api_key");

        // Validate required headers
        let (timestamp, signature, service_api_key) = match (timestamp, signature, service_api_key)
        {
            (None, _, _) => {
                return err(JsonResponse::data(
                    JsonData::error().set_sub_code("timestamp_missing"),
                )
                .set_message("X-Timestamp header is required")
                .into());
            }
            (Some(t), _, _) if t.trim().is_empty() => {
                return err(JsonResponse::data(
                    JsonData::error().set_sub_code("timestamp_invalid"),
                )
                .set_message("X-Timestamp header is invalid")
                .into());
            }
            (_, None, _) => {
                return err(JsonResponse::data(
                    JsonData::error().set_sub_code("signature_missing"),
                )
                .set_message("X-Signature header is required")
                .into());
            }
            (_, Some(s), _) if s.trim().is_empty() => {
                return err(JsonResponse::data(
                    JsonData::error().set_sub_code("signature_invalid"),
                )
                .set_message("X-Signature header is invalid")
                .into());
            }
            (_, _, Err(errobj)) => {
                return err(JsonResponse::data(
                    JsonData::error().set_sub_code("service_key_not_configured"),
                )
                .set_message(format!("load config fail:{}", errobj))
                .into());
            }
            (_, _, Ok(k)) if k.trim().is_empty() => {
                return err(JsonResponse::data(
                    JsonData::error().set_sub_code("service_key_invalid"),
                )
                .set_message("Service API key is invalid")
                .into());
            }
            (Some(t), Some(s), Ok(k)) => (t, s, k),
        };

        // Verify timestamp
        let ts: i64 = match timestamp.parse() {
            Ok(t) => t,
            Err(_) => {
                return err(JsonResponse::data(
                    JsonData::error().set_sub_code("timestamp_invalid"),
                )
                .set_message("Invalid timestamp format")
                .into());
            }
        };
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let diff = (now - ts).abs();
        if diff > TIMESTAMP_TOLERANCE_SECS {
            return err(
                JsonResponse::data(JsonData::error().set_sub_code("timestamp_expired"))
                    .set_message(format!("Timestamp expired, diff: {}s", diff))
                    .into(),
            );
        }

        // Verify signature
        let sign_result = compute_service_sign(&service_api_key, Some(timestamp));
        if sign_result.signature != signature {
            return err(
                JsonResponse::data(JsonData::error().set_sub_code("signature_invalid"))
                    .set_message(format!(
                        "Invalid signature, expected: {}",
                        sign_result.signature
                    ))
                    .into(),
            );
        }

        // Parse opaque bearer token if present (optional - some endpoints may not require auth)
        let bearer_token = parse_bearer_from_request(req);

        ok(Self {
            inner: RequestAuthDao::new(
                UserAuthSession::new(
                    web_dao.web_user.user_dao.auth_dao.clone(),
                    UserAuthToken::default(),
                ),
            ),
            bearer_token,
        })
    }
}

fn parse_bearer_from_request(req: &HttpRequest) -> Option<String> {
    let auth_header = req.headers().get("Authorization")?;
    let token_str = auth_header.to_str().ok()?;

    if !token_str.trim().starts_with("Bearer ") {
        return None;
    }

    let token = token_str.trim()[7..].trim();
    if token.is_empty() {
        return None;
    }

    // 过「前缀 + 校验和」闸门，拿到内部不透明 token
    let sign_key = TokenSignConfig::from_request(req).sign_key().to_string();
    verify_token(token, &sign_key).ok()
}

impl ServiceQuery {
    /// Get the opaque bearer token, returning error if not present
    pub fn require_token(&self) -> Result<&String, ResponseJson> {
        self.bearer_token.as_ref().ok_or_else(|| {
            JsonResponse::data(JsonData::error().set_sub_code("token_required"))
                .set_message("Authorization header with valid token is required")
                .into()
        })
    }

    /// Set user token from the opaque bearer token
    pub async fn set_user_token_from_bearer(&self) -> Result<(), ResponseJson> {
        let token_str = self.require_token()?;
        let token = UserAuthToken::from_str(token_str).map_err(|e| {
            JsonResponse::data(JsonData::error().set_sub_code("token_invalid"))
                .set_message(format!("Invalid token: {:?}", e))
        })?;
        self.user_session.write().await.set_session_token(token);
        Ok(())
    }
}
