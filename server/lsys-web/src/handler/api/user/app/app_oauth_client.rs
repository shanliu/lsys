use crate::common::{JsonData, JsonResult};
use crate::common::{JsonResponse, UserAuthQueryDao};
use crate::dao::access::api::user::CheckUserAppEdit;

use lsys_access::dao::AccessSession;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct OAuthClientScopeDataParam {
    pub app_id: Option<u64>,
}

pub async fn oauth_client_scope_data(
    param: &OAuthClientScopeDataParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let papp = if param.app_id.unwrap_or_default() > 0 {
        let parent_app = req_dao
            .web_dao
            .web_app
            .app_dao
            .app
            .cache()
            .find_by_id(param.app_id.unwrap_or_default())
            .await?;
        //父应用必须已开通OAUTH SERVER功能
        req_dao
            .web_dao
            .web_app
            .app_dao
            .oauth_server
            .oauth_check(&parent_app)
            .await?;
        Some(parent_app)
    } else {
        None
    };
    let server_spoce = req_dao
        .web_dao
        .web_app
        .app_oauth_server_scope_data(papp.as_ref())
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({
        "scope":server_spoce
    }))))
}

#[derive(Deserialize)]
pub struct OAuthClientRequestParam {
    pub scope_data: Vec<String>,
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
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
        .find_by_id(param.app_id)
        .await?;
    app.app_status_check()?;
    req_dao
        .web_dao
        .web_app
        .self_app_check(&app, &auth_data)
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
            .find_by_id(app.parent_app_id)
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
        .find_by_id(param.app_id)
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
pub struct ConfirmOAuthClientSetDomainParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    pub callback_domain: String,
}

pub async fn oauth_client_set_domain(
    param: &ConfirmOAuthClientSetDomainParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .find_by_id(param.app_id)
        .await?;
    app.app_status_check()?;
    req_dao
        .web_dao
        .web_app
        .self_app_check(&app, &auth_data)
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
        .oauth_set_domain(
            &app,
            &param.callback_domain,
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::default())
}

#[derive(Deserialize)]
pub struct AddOAuthSecretParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    pub secret: Option<String>,
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub secret_timeout: u64,
}

pub async fn oauth_secret_add(
    param: &AddOAuthSecretParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .find_by_id(param.app_id)
        .await?;
    app.app_status_check()?;
    req_dao
        .web_dao
        .web_app
        .self_app_check(&app, &auth_data)
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

    let secret_data = req_dao
        .web_dao
        .web_app
        .app_dao
        .oauth_client
        .secret_add(
            &app,
            param.secret.as_deref(),
            param.secret_timeout,
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;

    Ok(JsonResponse::data(JsonData::body(
        json!({"data":secret_data}),
    )))
}

#[derive(Deserialize)]
pub struct ChangeOAuthSecretParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    pub old_secret: String,
    pub secret: Option<String>,
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub secret_timeout: u64,
}

pub async fn oauth_secret_change(
    param: &ChangeOAuthSecretParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .find_by_id(param.app_id)
        .await?;
    app.app_status_check()?;
    req_dao
        .web_dao
        .web_app
        .self_app_check(&app, &auth_data)
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

    let secret_data = req_dao
        .web_dao
        .web_app
        .app_dao
        .oauth_client
        .secret_change(
            &app,
            &param.old_secret,
            param.secret.as_deref(),
            param.secret_timeout,
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;

    Ok(JsonResponse::data(JsonData::body(
        json!({"data":secret_data}),
    )))
}

#[derive(Deserialize)]
pub struct DelOAuthSecretParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    pub old_secret: String,
}

pub async fn oauth_secret_del(
    param: &DelOAuthSecretParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
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
        .self_app_check(&app, &auth_data)
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
        .secret_del(
            &app,
            &param.old_secret,
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;

    Ok(JsonResponse::default())
}
