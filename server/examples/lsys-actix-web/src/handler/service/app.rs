
use actix_web::post;
use lsys_web::handler::service::app::{self as service_app, AppFeatureParam, AppSecretParam};
use lsys_web::dao::WebDao;

use crate::common::handler::{JsonQuery, ReqQuery, ResponseJson, ResponseJsonResult, ServiceQuery};

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
    _: ServiceQuery,
    path: actix_web::web::Path<String>,
    json_param: JsonQuery,
    req_query: ReqQuery,
    web_dao: actix_web::web::Data<WebDao>,
) -> ResponseJsonResult<ResponseJson> {
    let method = path.into_inner();
    let result = match method.as_str() {
        "feature" => {
            service_app::feature(&json_param.param::<AppFeatureParam>()?, web_dao.as_ref()).await
        }
        "secret" => {
            service_app::secret(&json_param.param::<AppSecretParam>()?, web_dao.as_ref()).await
        }
        _ => handler_not_found!(method),
    };
    result
        .map(|r| r.into())
        .map_err(|e| req_query.fluent_error_json_response(&e).into())
}
