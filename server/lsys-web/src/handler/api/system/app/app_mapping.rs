use crate::common::JsonData;
use crate::common::JsonResponse;
use crate::common::JsonResult;
use crate::common::UserAuthQueryDao;
use crate::handler::APP_FEATURE_BARCODE;
use crate::handler::APP_FEATURE_MAIL;
use crate::handler::APP_FEATURE_RBAC;
use crate::handler::APP_FEATURE_SMS;
use lsys_app::model::AppRequestStatus;
use lsys_app::model::AppRequestType;
use lsys_app::model::AppSecretStatus;
use lsys_app::model::AppStatus;
use serde_json::json;
pub async fn mapping_data(req_dao: &UserAuthQueryDao) -> JsonResult<JsonResponse> {
    Ok(JsonResponse::data(JsonData::body(json!({
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
        "secret_status":vec![
            status_json_format!(req_dao, AppSecretStatus::Enable),
            status_json_format!(req_dao, AppSecretStatus::Delete),
        ],
        "exter_features":vec![
            const_json_format!(req_dao, APP_FEATURE_SMS),
            const_json_format!(req_dao, APP_FEATURE_MAIL),
            const_json_format!(req_dao, APP_FEATURE_BARCODE),
            const_json_format!(req_dao, APP_FEATURE_RBAC),
        ],
         "request_type":vec![
            status_json_format!(req_dao, AppRequestType::AppReq),
            status_json_format!(req_dao, AppRequestType::AppChange),
            status_json_format!(req_dao, AppRequestType::SubApp),
            status_json_format!(req_dao, AppRequestType::ExterLogin),
            status_json_format!(req_dao, AppRequestType::OAuthServer),
            status_json_format!(req_dao, AppRequestType::OAuthClient),
            status_json_format!(req_dao, AppRequestType::OAuthClientScope),
            status_json_format!(req_dao, AppRequestType::ExterFeatuer),
        ],
    }))))
}
