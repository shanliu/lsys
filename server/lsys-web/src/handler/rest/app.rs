use crate::common::JsonData;
use crate::common::RequestDao;
use crate::common::{JsonResponse, JsonResult};
use crate::dao::access::rest::CheckRestApp;
use crate::dao::access::RbacAccessCheckEnv;
use lsys_app::model::AppModel;
use serde::Deserialize;
use serde_json::json;
#[derive(Debug, Deserialize)]
pub struct SubAppInfoParam {
    pub client_id: String,
}

pub async fn subapp_info(
    param: &SubAppInfoParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    let app_user = req_dao
        .web_dao
        .web_access
        .access_dao
        .user
        .cache()
        .find_by_id(&app.user_id)
        .await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::user(&app_user, &req_dao.req_env),
            &CheckRestApp {},
        )
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

    let client_data = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .cache()
        .find_app_secret_by_client_id(&out_app.client_id)
        .await?;

    Ok(JsonResponse::data(JsonData::body(json!({
        "name": out_app.name,
        "client_id":out_app.client_id,
        "sub_secret": client_data,
    }))))
}

#[derive(Debug, Deserialize)]
pub struct SubAppUserParam {
    pub client_id: String,
}

pub async fn subapp_user(
    param: &SubAppUserParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    let app_user = req_dao
        .web_dao
        .web_access
        .access_dao
        .user
        .cache()
        .find_by_id(&app.user_id)
        .await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::user(&app_user, &req_dao.req_env),
            &CheckRestApp {},
        )
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

    let user_info = req_dao
        .web_dao
        .web_access
        .access_dao
        .user
        .cache()
        .find_user_by_id(out_app.user_id)
        .await?;

    Ok(JsonResponse::data(JsonData::body(json!({
        "name": out_app.name,
        "client_id":out_app.client_id,
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
    let app_user = req_dao
        .web_dao
        .web_access
        .access_dao
        .user
        .cache()
        .find_by_id(&app.user_id)
        .await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::user(&app_user, &req_dao.req_env),
            &CheckRestApp {},
        )
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
    let app_user = req_dao
        .web_dao
        .web_access
        .access_dao
        .user
        .cache()
        .find_by_id(&app.user_id)
        .await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::user(&app_user, &req_dao.req_env),
            &CheckRestApp {},
        )
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
