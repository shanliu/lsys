use crate::common::JsonData;
use crate::common::JsonResponse;
use crate::common::JsonResult;
use crate::common::UserAuthQueryDao;
use lsys_access::model::SessionStatus;
use lsys_user::model::AccountEmailStatus;
use lsys_user::model::AccountMobileStatus;
use lsys_user::model::AccountStatus;
use serde_json::json;
pub async fn mapping_data(req_dao: &UserAuthQueryDao) -> JsonResult<JsonResponse> {
    Ok(JsonResponse::data(JsonData::body(json!({
        "session_status":vec![
            status_format!(json req_dao, SessionStatus::Enable),
            status_format!(json req_dao, SessionStatus::Delete),
        ],
        "mobile_status":vec![
             status_format!(json req_dao, AccountMobileStatus::Init),
            status_format!(json req_dao, AccountMobileStatus::Valid),
        ],
        "email_status":vec![
            status_format!(json req_dao, AccountEmailStatus::Init),
            status_format!(json req_dao, AccountEmailStatus::Valid),
        ],
        "account_status":vec![
            status_format!(json req_dao, AccountStatus::Init),
            status_format!(json req_dao, AccountStatus::Enable),
        ],
    }))))
}
