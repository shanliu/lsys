use crate::common::JsonData;
use crate::common::JsonResponse;
use crate::common::JsonResult;
use crate::common::UserAuthQueryDao;
use lsys_rbac::model::RbacAuditResult;
use lsys_rbac::model::RbacRoleResRange;
use lsys_rbac::model::RbacRoleUserRange;
use serde_json::json;
pub async fn mapping_data(req_dao: &UserAuthQueryDao) -> JsonResult<JsonResponse> {
    Ok(JsonResponse::data(JsonData::body(json!({
        "role_res_range":vec![
            status_json_format!(req_dao, RbacRoleResRange::Exclude),
            status_json_format!(req_dao, RbacRoleResRange::Any),
            status_json_format!(req_dao, RbacRoleResRange::Include),
        ],
           "role_user_range":vec![
            status_json_format!(req_dao, RbacRoleUserRange::Custom),
            status_json_format!(req_dao, RbacRoleUserRange::Session),
        ],
        "audit_result":vec![
            status_json_format!(req_dao, RbacAuditResult::Succ),
            status_json_format!(req_dao, RbacAuditResult::Fail),
        ],
    }))))
}
