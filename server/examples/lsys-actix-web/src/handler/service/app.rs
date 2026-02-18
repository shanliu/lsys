use actix_web::post;
use lsys_web::handler::service::app::{self as service_app, AppFeatureParam, AppSecretParam};

use crate::common::handler::{JsonQuery, ResponseJson, ResponseJsonResult, ServiceQuery};

/// Application service endpoints
///
/// POST /service/app/{method}
///
/// Headers:
/// - X-Timestamp: required
/// - X-Signature: required
///
/// Methods:
/// - feature: Check if app has specific features enabled
/// - secret: Get application secrets by client_id
#[post("/app/{method}")]
pub async fn app(
    service: ServiceQuery,
    path: actix_web::web::Path<String>,
    json_param: JsonQuery,
) -> ResponseJsonResult<ResponseJson> {
    let method = path.into_inner();
    let result = match method.as_str() {
        "feature" => {
            service_app::feature(&json_param.param::<AppFeatureParam>()?, &service.inner).await
        }
        "secret" => {
            service_app::secret(&json_param.param::<AppSecretParam>()?, &service.inner).await
        }
        _ => handler_not_found!(method),
    };
    result
        .map(|r| r.into())
        .map_err(|e| service.fluent_error_json_response(&e).into())
}
