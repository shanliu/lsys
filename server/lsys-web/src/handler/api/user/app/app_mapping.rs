use crate::common::JsonData;
use crate::common::JsonResponse;
use crate::common::JsonResult;
use crate::common::UserAuthQueryDao;
use lsys_app::dao::SUB_APP_SECRET_NOTIFY_TYPE;
use lsys_app::model::AppNotifyDataStatus;
use lsys_app::model::AppRequestStatus;
use lsys_app::model::AppStatus;
use lsys_app_sender::dao::SMS_NOTIFY_TYPE;
use serde_json::json;
pub async fn mapping_data(req_dao: &UserAuthQueryDao) -> JsonResult<JsonResponse> {
    Ok(JsonResponse::data(JsonData::body(json!({
        "notify_method":vec![
            const_json_format!(req_dao, SMS_NOTIFY_TYPE),
            const_json_format!(req_dao, SUB_APP_SECRET_NOTIFY_TYPE),
        ],
        "notify_status":vec![
            status_json_format!(req_dao, AppNotifyDataStatus::Init),
            status_json_format!(req_dao, AppNotifyDataStatus::Succ),
            status_json_format!(req_dao, AppNotifyDataStatus::Fail),
        ],
         "app_status":vec![
            status_json_format!(req_dao, AppStatus::Enable),
            status_json_format!(req_dao, AppStatus::Init),
            status_json_format!(req_dao, AppStatus::Disable),
        ],
        "request_status":vec![
            status_json_format!(req_dao, AppRequestStatus::Pending),
            status_json_format!(req_dao, AppRequestStatus::Approved),
            status_json_format!(req_dao, AppRequestStatus::Rejected),
            status_json_format!(req_dao, AppRequestStatus::Invalid),
        ],
    }))))
}
