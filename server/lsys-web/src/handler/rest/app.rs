use crate::common::JsonData;
use crate::common::RequestDao;
use crate::common::{JsonResponse, JsonResult};
use crate::dao::access::rest::CheckRestApp;
use lsys_app::model::AppModel;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct SubAppViewParam {
    pub client_id: String,
    pub user_data: bool,
    pub oauth_data: bool,
}

pub async fn subapp_view(
    param: &SubAppViewParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, None, &CheckRestApp {})
        .await?;

    req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .inner_feature_sub_app_check(app)
        .await?;

    let out_app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .cache()
        .find_sub_app_by_client_id(app, &param.client_id)
        .await?;

    let client_secret = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .app_view_secret(&out_app, app.user_id, Some(&req_dao.req_env))
        .await?;

    let user_info = if param.user_data {
        let user_info = req_dao
            .web_dao
            .web_access
            .access_dao
            .user
            .cache()
            .find_user_by_id(&out_app.user_id)
            .await?;
        json!({
            "user_id": user_info.id,
            "user_name": user_info.user_name,
            "user_data": user_info.user_data,
        })
    } else {
        json!(null)
    };
    Ok(JsonResponse::data(JsonData::body(json!({
        "name": out_app.name,
        "client_id":out_app.client_id,
        "sub_secret": client_secret,
        "user_data":user_info,
    }))))
}

#[derive(Debug, Deserialize)]
pub struct SubAppOAuthSecretParam {
    pub client_id: String,
}

pub async fn subapp_oauth_secret(
    param: &SubAppOAuthSecretParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, None, &CheckRestApp {})
        .await?;

    //父应用开通了 OAuthServer
    req_dao
        .web_dao
        .web_app
        .app_dao
        .oauth_server
        .oauth_check(app)
        .await?;
    //查询的 client_id 为该app的子应用
    let out_app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .cache()
        .find_sub_app_by_client_id(app, &param.client_id)
        .await?;
    //子应用开通了 OAuthClient
    req_dao
        .web_dao
        .web_app
        .app_dao
        .oauth_client
        .oauth_check(&out_app)
        .await?;

    let secret = req_dao
        .web_dao
        .web_app
        .app_dao
        .oauth_client
        .oauth_view_secret(&out_app, app.user_id, Some(&req_dao.req_env))
        .await?;

    Ok(JsonResponse::data(JsonData::body(json!({
        "name": out_app.name,
        "client_id":out_app.client_id,
        "secret":secret
    }))))
}

#[derive(Debug, Deserialize)]
pub struct SubAppOAuthScopeParam {
    pub client_id: String,
}

pub async fn subapp_oauth_scope(
    param: &SubAppOAuthScopeParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, None, &CheckRestApp {})
        .await?;

    //父应用开通了 OAuthServer
    req_dao
        .web_dao
        .web_app
        .app_dao
        .oauth_server
        .oauth_check(app)
        .await?;
    //查询的 client_id 为该app的子应用
    let out_app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .cache()
        .find_sub_app_by_client_id(app, &param.client_id)
        .await?;
    //子应用开通了 OAuthClient
    req_dao
        .web_dao
        .web_app
        .app_dao
        .oauth_client
        .oauth_check(&out_app)
        .await?;

    let oauth_data = req_dao
        .web_dao
        .web_app
        .app_oauth_client_get_scope_data(&out_app)
        .await?;

    Ok(JsonResponse::data(JsonData::body(json!({
        "name": out_app.name,
        "client_id":out_app.client_id,
        "scope_data":oauth_data,
    }))))
}
