use crate::common::JsonData;
use crate::common::JsonResponse;
use crate::common::JsonResult;
use crate::common::RequestDao;
use lsys_rbac::model::RbacAuditIs;
use lsys_rbac::model::RbacAuditResult;
use lsys_rbac::model::RbacRoleResRange;
use lsys_rbac::model::RbacRoleUserRange;
use serde_json::json;
pub async fn mapping_data(req_dao: &RequestDao) -> JsonResult<JsonResponse> {
    Ok(JsonResponse::data(JsonData::body(json!({
        "role_res_range":vec![
            status_format!(json req_dao, RbacRoleResRange::Exclude),
            status_format!(json req_dao, RbacRoleResRange::Any),
            status_format!(json req_dao, RbacRoleResRange::Include),
        ],
        "role_user_range":vec![
            status_format!(json req_dao, RbacRoleUserRange::Custom),
            status_format!(json req_dao, RbacRoleUserRange::Session),
        ],
        "audit_result":vec![
            status_format!(json req_dao, RbacAuditResult::Succ),
            status_format!(json req_dao, RbacAuditResult::Fail),
        ],
        "audit_is":vec![
            status_format!(json req_dao, RbacAuditIs::Yes),
            status_format!(json req_dao, RbacAuditIs::No),
        ],
    }))))
}
