use crate::common::{JsonResponse, UserAuthQueryDao};
use crate::common::{JsonError, JsonResult};
use crate::dao::access::api::user::CheckUserAppEdit;
use lsys_access::dao::AccessSession;
use lsys_app::dao::AppOAuthServerScopeParam;
use lsys_app::model::AppRequestStatus;
use lsys_core::fluent_message;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct ConfirmOAuthClientParam {
    pub app_id: u64,
    pub confirm_status: i8,
    pub confirm_note: String,
}

pub async fn oauth_server_client_confirm(
    param: &ConfirmOAuthClientParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let confirm_status = AppRequestStatus::try_from(param.confirm_status)?;
    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .find_by_id(&param.app_id)
        .await?;

    if app.user_app_id == 0 {
        return Err(JsonError::Message(fluent_message!("not-user-app-confirm")));
    }

    let parent_app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .find_by_id(&app.parent_app_id)
        .await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &req_dao.req_env,
            Some(&auth_data),
            &CheckUserAppEdit {
                res_user_id: parent_app.user_id,
            },
        )
        .await?;

    req_dao
        .web_dao
        .web_app
        .app_dao
        .oauth_server
        .oauth_check(&parent_app)
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
    pub app_id: u64,
    pub app_req_id: u64,
    pub confirm_status: i8,
    pub confirm_note: String,
}

pub async fn oauth_server_client_scope_confirm(
    param: &ConfirmOAuthClientScopeParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
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

    if app.user_app_id == 0 {
        return Err(JsonError::Message(fluent_message!("not-user-app-confirm")));
    }

    let parent_app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .find_by_id(&app.parent_app_id)
        .await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &req_dao.req_env,
            Some(&auth_data),
            &CheckUserAppEdit {
                res_user_id: parent_app.user_id,
            },
        )
        .await?;

    //开通过的不影响scope申请审核

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

#[derive(Deserialize)]
pub struct OAuthServerRequestData {
    pub app_id: u64,
}

pub async fn oauth_server_request(
    param: &OAuthServerRequestData,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
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
        .oauth_server
        .oauth_request(&app, auth_data.user_id(), Some(&req_dao.req_env))
        .await?;
    Ok(JsonResponse::default())
}

#[derive(Deserialize)]
pub struct ConfirmOAuthServerSettingScopeParam {
    pub key: String,
    pub name: String,
    pub desc: String,
}

#[derive(Deserialize)]
pub struct ConfirmOAuthServerSettingParam {
    app_id: u64,
    scope_data: Vec<ConfirmOAuthServerSettingScopeParam>,
}

pub async fn oauth_server_setting(
    param: &ConfirmOAuthServerSettingParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
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
        .oauth_server
        .oauth_check(&app)
        .await?;

    let scope_data = param
        .scope_data
        .iter()
        .map(|e| AppOAuthServerScopeParam {
            key: &e.key,
            name: &e.name,
            desc: &e.desc,
        })
        .collect::<Vec<_>>();
    req_dao
        .web_dao
        .web_app
        .app_dao
        .oauth_server
        .oauth_setting(
            &app,
            &scope_data,
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::default())
}
