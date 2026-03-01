use crate::common::JsonData;
use crate::common::JsonResponse;
use crate::common::JsonResult;
use crate::common::UserAuthQueryDao;
use crate::handler::APP_FEATURE_FILE;
use crate::handler::APP_FEATURE_MAIL;
use crate::handler::APP_FEATURE_RBAC;
use crate::handler::APP_FEATURE_SMS;
use lsys_access::dao::AccessSession;
use lsys_app::dao::SUB_APP_SECRET_NOTIFY_METHOD;
use lsys_app::model::AppNotifyDataStatus;
use lsys_app::model::AppRequestStatus;
use lsys_app::model::AppRequestType;
use lsys_app::model::AppStatus;
use lsys_app_sender::dao::SMS_NOTIFY_METHOD;
use lsys_core::db::OffsetPageParam;
use lsys_core::fluents::IntoFluentMessage;
use serde_json::json;
pub async fn mapping_data(req_dao: &UserAuthQueryDao) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let mut exter_features = if auth_data.user().app_id > 0 {
        vec![
            const_json_format!(req_dao, APP_FEATURE_SMS, { "source": "code" }),
            const_json_format!(req_dao, APP_FEATURE_MAIL, { "source": "code" }),
            const_json_format!(req_dao, APP_FEATURE_FILE, { "source": "code" }),
        ]
    } else {
        vec![
            const_json_format!(req_dao, APP_FEATURE_SMS, { "source": "code" }),
            const_json_format!(req_dao, APP_FEATURE_MAIL, { "source": "code" }),
            const_json_format!(req_dao, APP_FEATURE_RBAC, { "source": "code" }),
            const_json_format!(req_dao, APP_FEATURE_FILE, { "source": "code" }),
        ]
    };

    let db_exter_features = req_dao
        .web_dao
        .web_app
        .exter_feature_list(&OffsetPageParam::new(None))
        .await
        .map_err(|e| crate::common::JsonError::Message(e.to_fluent_message()))?;
    for item in db_exter_features {
        let obj = json!({
            "key": item.key,
            "val": item.data.title,
            "source": "database",
            "id": item.id,
        });
        exter_features.push(obj);
    }
    Ok(JsonResponse::data(JsonData::body(json!({
        "notify_method":vec![
            const_json_format!(req_dao, SMS_NOTIFY_METHOD),
            const_json_format!(req_dao, SUB_APP_SECRET_NOTIFY_METHOD),
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
        "exter_features":exter_features,
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
