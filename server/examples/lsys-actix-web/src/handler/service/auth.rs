use actix_web::post;
use lsys_web::handler::service::auth as service_auth;

use crate::common::handler::{ReqQuery, ResponseJson, ResponseJsonResult, ServiceQuery};

/// Verify auth token and return user information
///
/// POST /service/auth/verify
///
/// Headers:
/// - X-API-Key: required
/// - Authorization: Bearer <token>
///
/// Returns user_id, nickname, username if valid
#[post("/auth/verify")]
pub async fn verify(
    service: ServiceQuery,
    req_query: ReqQuery,
) -> ResponseJsonResult<ResponseJson> {
    // Require and validate opaque bearer token
    service.set_user_token_from_bearer().await?;

    // 使用 lsys-web 中的服务实现
    service_auth::verify(&*service)
        .await
        .map(|r| r.into())
        .map_err(|e| req_query.fluent_error_json_response(&e).into())
}
