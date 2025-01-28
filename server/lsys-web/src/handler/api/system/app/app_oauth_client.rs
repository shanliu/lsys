use crate::{
    common::{JsonData, JsonResult, UserAuthQueryDao},
    dao::access::api::system::CheckAdminApp,
};

use lsys_access::dao::AccessSession;
use lsys_app::model::AppRequestStatus;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct AppConfirmOAuthClientParam {
    pub app_id: u64,
    pub confirm_status: i8,
    pub confirm_note: String,
}
//oauth 接入申请审核
pub async fn app_oauth_client_confirm(
    param: &AppConfirmOAuthClientParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.access_env().await?, &CheckAdminApp {}, None)
        .await?;
    let confirm_status = AppRequestStatus::try_from(param.confirm_status)?;
    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .find_by_id(&param.app_id)
        .await?;
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_app
        .app_dao
        .oauth_client
        .oauth_confirm(
            &app,
            confirm_status,
            &param.confirm_note,
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::default())
}

#[derive(Deserialize)]
pub struct AppConfirmOAuthClientScopeParam {
    pub app_id: u64,
    pub app_req_id: u64,
    pub confirm_status: i8,
    pub confirm_note: String,
}
//oauth 接入申请新权限审核
pub async fn app_oauth_client_scope_confirm(
    param: &AppConfirmOAuthClientScopeParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.access_env().await?, &CheckAdminApp {}, None)
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
        .find_by_id(&param.app_id)
        .await?;
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_app
        .app_dao
        .oauth_client
        .scope_confirm(
            &app,
            &req_app,
            confirm_status,
            &param.confirm_note,
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::default())
}
