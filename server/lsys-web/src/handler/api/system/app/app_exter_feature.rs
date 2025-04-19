use crate::{
    common::{JsonResponse, JsonResult, UserAuthQueryDao},
    dao::access::api::system::CheckAdminApp,
};
use lsys_access::dao::AccessSession;
use lsys_app::model::AppRequestStatus;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ConfirmExterFeatureParam {
    pub app_req_id: u64,
    pub confirm_status: i8,
    pub confirm_note: String,
}
// APP功能审核,如邮件,短信等
pub async fn confirm_exter_feature(
    param: &ConfirmExterFeatureParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
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
    let confirm_status = AppRequestStatus::try_from(param.confirm_status)?;
    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .find_by_id(&req_app.app_id)
        .await?;
    req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .exter_feature_confirm(
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
