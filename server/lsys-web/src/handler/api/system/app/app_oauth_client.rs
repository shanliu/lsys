use crate::{
    common::{JsonResponse, JsonResult, UserAuthQueryDao},
    dao::access::api::system::CheckAdminApp,
};

use lsys_access::dao::AccessSession;
use lsys_app::model::AppRequestStatus;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ConfirmOAuthClientParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    #[serde(deserialize_with = "crate::common::deserialize_i8")]
    pub confirm_status: i8,
    pub confirm_note: String,
}
//oauth 接入申请审核
pub async fn oauth_client_confirm(
    param: &ConfirmOAuthClientParam,
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
        .find_by_id(param.app_id)
        .await?;
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
    Ok(JsonResponse::default())
}

#[derive(Deserialize)]
pub struct ConfirmOAuthClientScopeParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_req_id: u64,
    #[serde(deserialize_with = "crate::common::deserialize_i8")]
    pub confirm_status: i8,
    pub confirm_note: String,
}
//oauth 接入申请新权限审核
pub async fn oauth_client_scope_confirm(
    param: &ConfirmOAuthClientScopeParam,
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
        .request_find_by_id(param.app_req_id)
        .await?;
    let confirm_status = AppRequestStatus::try_from(param.confirm_status)?;
    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .find_by_id(req_app.app_id)
        .await?;

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
    Ok(JsonResponse::default())
}

#[derive(Debug, Deserialize)]
pub struct ClearOAuthClientAccessTokenParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    pub access_token: String,
}
pub async fn oauth_client_clear_access_token(
    param: &ClearOAuthClientAccessTokenParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminApp {})
        .await?;
    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .find_by_id(param.app_id)
        .await?;
    req_dao
        .web_dao
        .web_app
        .app_dao
        .oauth_client
        .clear_access_token(&app, &param.access_token)
        .await?;
    Ok(JsonResponse::default())
}

#[derive(Debug, Deserialize)]
pub struct ClearOAuthClientRefreshTokenParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    pub refresh_token: String,
}
pub async fn oauth_client_clear_refresh_token(
    param: &ClearOAuthClientRefreshTokenParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminApp {})
        .await?;
    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .find_by_id(param.app_id)
        .await?;
    req_dao
        .web_dao
        .web_app
        .app_dao
        .oauth_client
        .clear_refresh_token(&app, &param.refresh_token)
        .await?;
    Ok(JsonResponse::default())
}
