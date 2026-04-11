use actix_http::header;
use actix_utils::future::{Ready, err, ok};
use actix_web::{FromRequest, HttpRequest, dev::Payload, web::Data};
use jsonwebtoken::{DecodingKey, Validation, decode};
use lsys_web::lsys_access::dao::AccessSession;
use lsys_web::lsys_core::api_utils::{
    SERVICE_SIGNATURE_HEADER, SERVICE_TIMESTAMP_HEADER, compute_service_sign,
};
use lsys_web::lsys_core::fluents::IntoFluentMessage;
use lsys_web::lsys_core::utils::RequestEnv;
use lsys_web::lsys_user::dao::{UserAuthSession, UserAuthToken};
use lsys_web::{
    common::{JsonData, JsonResponse, RequestAuthDao},
    dao::WebDao,
};
use std::ops::Deref;
use std::str::FromStr;

use super::ResponseJson;
use super::request_jwt::{JwtClaims, JwtQueryConfig};

const FORWARDED_FOR_HEADER: &str = "X-Forwarded-For";

/// 时间戳有效期（秒）
const TIMESTAMP_TOLERANCE_SECS: i64 = 3600; // 1小时

/// Service request query extractor
///
/// 验证 X-Timestamp + X-Signature 头部，用于服务间调用
///
/// 签名验证: X-Signature = MD5(service_api_key + X-Timestamp)
/// 时间戳验证: X-Timestamp 必须在 1小时
pub struct ServiceQuery {
    pub inner:
        RequestAuthDao<UserAuthToken, lsys_web::lsys_user::dao::UserAuthData, UserAuthSession>,
    pub jwt_claims: Option<JwtClaims>,
    #[allow(unused)]
    pub req: HttpRequest,
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

        // Now we trust forwarded headers
        let user_lang = req
            .headers()
            .get(header::ACCEPT_LANGUAGE)
            .and_then(|t| t.to_str().map(|s| s.split(',').next().unwrap_or(s)).ok())
            .unwrap_or("zh_CN")
            .replace('-', "_");

        let user_agent = req
            .headers()
            .get("User-Agent")
            .and_then(|e| e.to_str().ok());

        let request_id = req
            .headers()
            .get("X-Request-ID")
            .and_then(|e| e.to_str().ok());

        let device_id = req
            .headers()
            .get("X-Device-ID")
            .and_then(|e| e.to_str().ok());

        // Trust X-Forwarded-For from validated service calls
        let client_ip: Option<String> = req
            .headers()
            .get(FORWARDED_FOR_HEADER)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.split(',').next())
            .map(|s| s.trim().to_string())
            .or_else(|| {
                req.connection_info()
                    .realip_remote_addr()
                    .map(|s| s.to_string())
            });

        let env = match RequestEnv::new(
            Some(&user_lang),
            client_ip.as_deref(),
            request_id,
            user_agent,
            device_id,
        ) {
            Ok(tmp) => tmp,
            Err(verr) => {
                return err(JsonResponse::data(
                    JsonData::default()
                        .set_sub_code("env_valid_err")
                        .set_code(400),
                )
                .set_message(verr.to_fluent_message().default_format())
                .into());
            }
        };

        // Parse JWT if present (optional - some endpoints may not require auth)
        let jwt_claims = parse_jwt_from_request(req, web_dao);

        ok(Self {
            inner: RequestAuthDao::new(
                web_dao.clone().into_inner(),
                env,
                UserAuthSession::new(
                    web_dao.web_user.user_dao.auth_dao.clone(),
                    UserAuthToken::default(),
                ),
            ),
            jwt_claims,
            req: req.to_owned(),
        })
    }
}

fn parse_jwt_from_request(req: &HttpRequest, web_dao: &Data<WebDao>) -> Option<JwtClaims> {
    let auth_header = req.headers().get("Authorization")?;
    let token_str = auth_header.to_str().ok()?;

    if !token_str.trim().starts_with("Bearer ") {
        return None;
    }

    let token = token_str.trim()[7..].trim();

    // Try to get JWT config from app_data
    if let Some(config) = req.app_data::<JwtQueryConfig>()
        && let Ok(token_data) = decode::<JwtClaims>(token, &config.decode_key, &config.validation)
    {
        return Some(token_data.claims);
    }

    // Fallback: try to decode with app_jwt_key from config
    let jwt_key = web_dao
        .app_core
        .config
        .find(None)
        .get_string("app_jwt_key")
        .ok()?;

    let mut validation = Validation::default();
    validation.validate_exp = true;

    decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(jwt_key.as_bytes()),
        &validation,
    )
    .ok()
    .map(|t| t.claims)
}

impl ServiceQuery {
    /// Get JWT claims, returning error if not present
    pub fn require_jwt(&self) -> Result<&JwtClaims, ResponseJson> {
        self.jwt_claims.as_ref().ok_or_else(|| {
            JsonResponse::data(JsonData::error().set_sub_code("jwt_required"))
                .set_message("Authorization header with valid JWT is required")
                .into()
        })
    }

    /// Set user token from JWT claims
    pub async fn set_user_token_from_jwt(&self) -> Result<(), ResponseJson> {
        let claims = self.require_jwt()?;
        let token = UserAuthToken::from_str(&claims.token).map_err(|e| {
            JsonResponse::data(JsonData::error().set_sub_code("jwt_token_invalid"))
                .set_message(format!("Invalid token in JWT: {:?}", e))
        })?;
        self.user_session.write().await.set_session_token(token);
        Ok(())
    }
}
