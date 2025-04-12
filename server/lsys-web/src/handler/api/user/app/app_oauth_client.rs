use crate::common::JsonResult;
use crate::common::{JsonResponse, UserAuthQueryDao};
use crate::dao::access::api::user::CheckUserAppEdit;

use lsys_access::dao::AccessSession;
use lsys_app::dao::AppOAuthClientParam;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct OAuthClientRequestParam {
    pub scope_data: Vec<String>,
    pub app_id: u64,
}

pub async fn oauth_client_request(
    param: &OAuthClientRequestParam,
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
        .web_rbac
        .check(
            &req_dao.req_env,
            Some(&auth_data),
            &CheckUserAppEdit {
                res_user_id: app.user_id,
            },
        )
        .await?;

    if app.parent_app_id > 0 {
        let parent_app = req_dao
            .web_dao
            .web_app
            .app_dao
            .app
            .find_by_id(&app.parent_app_id)
            .await?;
        //父应用必须已开通OAUTH SERVER功能
        req_dao
            .web_dao
            .web_app
            .app_dao
            .oauth_server
            .oauth_check(&parent_app)
            .await?;
    }

    req_dao
        .web_dao
        .web_app
        .app_dao
        .oauth_client
        .oauth_request(
            &app,
            &param
                .scope_data
                .iter()
                .map(|e| e.as_str())
                .collect::<Vec<_>>(),
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::default())
}

pub async fn oauth_client_scope_request(
    param: &OAuthClientRequestParam,
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
    //开通过 oauth_server 不影响scope申请
    req_dao
        .web_dao
        .web_rbac
        .check(
            &req_dao.req_env,
            Some(&auth_data),
            &CheckUserAppEdit {
                res_user_id: app.user_id,
            },
        )
        .await?;
    req_dao
        .web_dao
        .web_app
        .app_dao
        .oauth_client
        .scope_request(
            &app,
            &param
                .scope_data
                .iter()
                .map(|e| e.as_str())
                .collect::<Vec<_>>(),
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::default())
}

#[derive(Deserialize)]
pub struct ConfirmOAuthClientSettingParam {
    pub app_id: u64,
    pub callback_domain: String,
    pub oauth_secret: String,
}

pub async fn oauth_client_setting(
    param: &ConfirmOAuthClientSettingParam,
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
        .web_rbac
        .check(
            &req_dao.req_env,
            Some(&auth_data),
            &CheckUserAppEdit {
                res_user_id: app.user_id,
            },
        )
        .await?;

    req_dao
        .web_dao
        .web_app
        .app_dao
        .oauth_client
        .oauth_check(&app)
        .await?;

    req_dao
        .web_dao
        .web_app
        .app_dao
        .oauth_client
        .oauth_setting(
            &app,
            &AppOAuthClientParam {
                oauth_secret: Some(&param.oauth_secret),
                callback_domain: Some(&param.callback_domain),
            },
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::default())
}
