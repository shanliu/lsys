use crate::{
    common::{JsonResponse, JsonResult, UserAuthQueryDao},
    dao::access::api::system::CheckAdminApp,
};

use lsys_access::dao::AccessSession;
use lsys_app::model::AppRequestStatus;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ConfirmExterLoginFeatureParam {
    pub app_id: u64,
    pub confirm_status: i8,
    pub confirm_note: String,
}
// 通过能APP登录系统审核
pub async fn confirm_inner_feature_exter_login_confirm(
    param: &ConfirmExterLoginFeatureParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminApp {})
        .await?;
    let confirm_status = AppRequestStatus::try_from(param.confirm_status)?;
    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .find_by_id(&param.app_id)
        .await?;
    req_dao
        .web_dao
        .web_app
        .app_dao
        .exter_login
        .inner_feature_exter_login_confirm(
            &app,
            confirm_status,
            &param.confirm_note,
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::default())
}

pub struct ConfirmExterSubAppParam {
    pub app_id: u64,
    pub confirm_status: i8,
    pub confirm_note: String,
}

pub async fn confirm_inner_feature_sub_app_confirm(
    param: &ConfirmExterSubAppParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminApp {})
        .await?;
    let confirm_status = AppRequestStatus::try_from(param.confirm_status)?;
    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .find_by_id(&param.app_id)
        .await?;

    req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .inner_feature_sub_app_confirm(
            &app,
            confirm_status,
            &param.confirm_note,
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::default())
}
