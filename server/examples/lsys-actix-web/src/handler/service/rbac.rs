use actix_web::post;
use lsys_web::handler::service::rbac::{self as service_rbac, RbacMenuListParam};

use crate::common::handler::{JsonQuery, ResponseJson, ResponseJsonResult, ServiceQuery};

/// Check RBAC permissions for multiple items
///
/// POST /service/rbac/check_list
///
/// Headers:
/// - X-Timestamp: required
/// - X-Signature: required
///
/// Body:
/// - menu_res: array of menu items to check, each with:
///   - name: item name for identification
///   - check_res: same as single check body
///
/// Returns status for each check item
#[post("/rbac/check")]
pub async fn check(
    service: ServiceQuery,
    json_param: JsonQuery,
) -> ResponseJsonResult<ResponseJson> {
    let param = json_param.param::<RbacMenuListParam>()?;

    service_rbac::check_list(&param, &service.inner)
        .await
        .map(|r| r.into())
        .map_err(|e| service.fluent_error_json_response(&e).into())
}
