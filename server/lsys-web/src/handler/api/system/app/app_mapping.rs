use crate::common::JsonData;
use crate::common::JsonResponse;
use crate::common::JsonResult;
use crate::common::UserAuthQueryDao;
use lsys_app::model::AppRequestStatus;
use lsys_app::model::AppRequestType;
use lsys_app::model::AppSecretStatus;
use lsys_app::model::AppStatus;
use serde_json::json;
pub async fn mapping_data(req_dao: &UserAuthQueryDao) -> JsonResult<JsonResponse> {
    Ok(JsonResponse::data(JsonData::body(json!({
        "app_status":vec![
            status_format!(json req_dao, AppStatus::Enable),
            status_format!(json req_dao, AppStatus::Init),
            status_format!(json req_dao, AppStatus::Disable),
        ],
        "request_status":vec![
            status_format!(json req_dao, AppRequestStatus::Pending),
            status_format!(json req_dao, AppRequestStatus::Approved),
            status_format!(json req_dao, AppRequestStatus::Rejected),
            status_format!(json req_dao, AppRequestStatus::Invalid),
        ],
        "secret_status":vec![
            status_format!(json req_dao, AppSecretStatus::Enable),
            status_format!(json req_dao, AppSecretStatus::Delete),
        ],
         "request_type":vec![
            status_format!(json req_dao, AppRequestType::AppReq),
            status_format!(json req_dao, AppRequestType::AppChange),
            status_format!(json req_dao, AppRequestType::SubApp),
            status_format!(json req_dao, AppRequestType::ExterLogin),
            status_format!(json req_dao, AppRequestType::OAuthServer),
            status_format!(json req_dao, AppRequestType::OAuthClient),
            status_format!(json req_dao, AppRequestType::OAuthClientScope),
            status_format!(json req_dao, AppRequestType::ExterFeatuer),
        ],
    }))))
}
