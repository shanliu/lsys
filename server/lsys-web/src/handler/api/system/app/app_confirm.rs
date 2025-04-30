use crate::common::{JsonError, JsonResponse, JsonResult, UserAuthQueryDao};
use crate::dao::access::api::system::CheckAdminApp;
use lsys_access::dao::AccessSession;
use lsys_app::model::AppRequestStatus;
use lsys_core::fluent_message;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ConfirmParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_req_id: u64,
    #[serde(deserialize_with = "crate::common::deserialize_i8")]
    pub confirm_status: i8,
    pub confirm_note: String,
}
//APP 申请审核
pub async fn confirm(param: &ConfirmParam, req_dao: &UserAuthQueryDao) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminApp {})
        .await?;
    let req_app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .request_find_by_id(&param.app_req_id)
        .await?;
    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .find_by_id(&req_app.app_id)
        .await?;

    if app.user_app_id > 0 {
        return Err(JsonError::Message(fluent_message!(
            "not-system-app-confirm"
        )));
    }

    let confirm_status = AppRequestStatus::try_from(param.confirm_status)?;
    req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .app_confirm_request(
            &app,
            &req_app,
            confirm_status,
            &param.confirm_note,
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::default())
}
