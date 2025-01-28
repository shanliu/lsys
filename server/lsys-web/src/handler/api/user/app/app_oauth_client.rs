use crate::common::JsonResult;
use crate::common::{JsonData, UserAuthQueryDao};
use crate::dao::access::api::user::{CheckUserAppEdit, CheckUserAppView};

use lsys_access::dao::AccessSession;
use lsys_app::dao::AppOAuthClientParam;
use serde::Deserialize;
use serde_json::json;

pub struct AppOAuthClientRequestParam {
    pub scope_data: Vec<String>,
    pub app_id: u64,
}

pub async fn app_oauth_client_request(
    param: &AppOAuthClientRequestParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
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
            &req_dao.access_env().await?,
            &CheckUserAppEdit {
                res_user_id: app.user_id,
            },
            None,
        )
        .await?;
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
    Ok(JsonData::default())
}

pub async fn app_oauth_client_scope_request(
    param: &AppOAuthClientRequestParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
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
            &req_dao.access_env().await?,
            &CheckUserAppEdit {
                res_user_id: app.user_id,
            },
            None,
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
    Ok(JsonData::default())
}

pub struct AppConfirmOAuthClientSettingParam {
    pub app_id: u64,
    pub callback_domain: String,
    pub oauth_secret: String,
}

pub async fn app_oauth_client_setting(
    param: &AppConfirmOAuthClientSettingParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
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
            &req_dao.access_env().await?,
            &CheckUserAppEdit {
                res_user_id: app.user_id,
            },
            None,
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
    Ok(JsonData::default())
}

#[derive(Deserialize)]
pub struct AppOAuthClientViewSecretParam {
    pub app_id: u64,
}

pub async fn app_oauth_client_secret_view(
    param: &AppOAuthClientViewSecretParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
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
            &req_dao.access_env().await?,
            &CheckUserAppView {
                res_user_id: app.user_id,
            },
            None,
        )
        .await?;
    req_dao
        .web_dao
        .web_app
        .app_dao
        .oauth_client
        .oauth_check(&app)
        .await?;
    let secret_data = req_dao
        .web_dao
        .web_app
        .app_dao
        .oauth_client
        .oauth_view_secret(&app, auth_data.user_id(), Some(&req_dao.req_env))
        .await?;

    Ok(JsonData::data(json!({"data":secret_data})))
}
