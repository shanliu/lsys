use crate::common::JsonData;
use crate::dao::access::RbacAccessCheckEnv;
use crate::{
    common::{JsonError, JsonResponse, JsonResult, RequestDao, UserAuthQueryDao},
    dao::access::rest::CheckRestApp,
};
use lsys_access::dao::{AccessSession, AccessSessionData};
use lsys_app::{dao::AppOAuthCodeData, model::AppModel};
use lsys_core::fluent_message;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct ScopeGetParam {
    pub client_id: String,
    pub scope: String,
}
//登录页面显示的scope,当前登陆scope对应的功能
pub async fn scope_get(
    param: &ScopeGetParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckRestApp {},
        )
        .await?;
    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .cache()
        .find_by_client_id(&param.client_id)
        .await?;

    req_dao
        .web_dao
        .web_app
        .app_dao
        .oauth_client
        .oauth_check(&app)
        .await?;
    let scope_data = param.scope.split(',').collect::<Vec<&str>>();
    let scope = req_dao
        .web_dao
        .web_app
        .app_oauth_server_parse_scope_data(&app, &scope_data)
        .await?;
    Ok(JsonResponse::data(JsonData::body(
        json!({ "scope": scope }),
    )))
}

#[derive(Debug, Deserialize)]
pub struct AuthorizeDoParam {
    pub scope: String,
    pub client_id: String,
    pub redirect_uri: String,
}
//登陆code后创建,从访问页面来
pub async fn create_code(
    param: &AuthorizeDoParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    //   用户授权 scope跟资源静态编码关系 检查授权通过scope得到res检查全局授权 接口检查授权跟检查资源是否在token的scope中
    //   1. 请求用户 /oauth/authorize?client_id=app_id&redirect_uri=CALLBACK_URL&scope=read
    //   2. 根据scope查授权[关系key]，通过，显示登录,完成登录,跳到授权页面 以 scope 查询资源列表[还没有]
    //   3. 完成授权. 生成code ,存放redis, scope,授权时间 +client_id+授权user_id ,设置5分钟超时,回到用户站点 /callback?code=AUTHORIZATION_CODE
    //   4. 请求令牌 /oauth/token?client_id=CLIENT_ID&client_secret=CLIENT_SECRET&code=AUTHORIZATION_CODE
    //   5. 读取redis 判断client_id,生成token记录并放入本地缓存
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .find_by_client_id(&param.client_id)
        .await?;

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

    if !req_dao
        .web_dao
        .web_app
        .app_dao
        .oauth_client
        .check_callback_domain(&app, &param.redirect_uri)
        .await?
    {
        return Err(JsonError::JsonResponse(
            JsonData::default().set_sub_code("domain_no_match"),
            fluent_message!("app-redirect-uri-not-match"),
        ));
    }
    let scope_data = param.scope.split(',').collect::<Vec<&str>>();
    req_dao
        .web_dao
        .web_app
        .app_oauth_server_parse_scope_data(&app, &scope_data)
        .await?;
    let user_data = auth_data.session_body().user();
    let session_data = auth_data.session_body().session();
    let code = req_dao
        .web_dao
        .web_app
        .app_dao
        .oauth_client
        .create_code(
            &app,
            &AppOAuthCodeData {
                user_data: &user_data.user_data,
                user_nickname: &user_data.user_nickname,
                user_account: Some(&user_data.user_account),
                login_ip: Some(&session_data.login_ip),
                device_id: Some(&session_data.device_id),
                device_name: Some(&session_data.device_name),
                scope_data,
                session_data: vec![],
            },
        )
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({ "code": code }))))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SessionRecord {
    access_token: String,
    refresh_token: String,
    openid: String,
    scope: Vec<String>,
    expires_in: u64,
}

async fn check_app_secret(
    req_dao: &RequestDao,
    client_id: &str,
    client_secret: &String,
) -> JsonResult<AppModel> {
    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .cache()
        .find_by_client_id(client_id)
        .await?;
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

    let oauth_secret = req_dao
        .web_dao
        .web_app
        .app_dao
        .oauth_client
        .cache()
        .find_secret_by_app_id(app.id)
        .await?;
    if !oauth_secret.iter().any(|e| e.secret_data == *client_secret) {
        return Err(JsonError::JsonResponse(
            JsonData::default(),
            fluent_message!("client-secret-not-match"),
        ));
    }
    Ok(app)
}

#[derive(Debug, Deserialize)]
pub struct CodeParam {
    pub client_secret: String,
    pub client_id: String,
    pub code: String,
}

//创建登陆token
pub async fn create_token(req_dao: &RequestDao, code: &CodeParam) -> JsonResult<JsonResponse> {
    let app = check_app_secret(req_dao, &code.client_id, &code.client_secret).await?;
    let (auth_data, refresh_token) = req_dao
        .web_dao
        .web_app
        .app_dao
        .oauth_client
        .create_session(&app, &code.code)
        .await?;
    let session = SessionRecord {
        access_token: auth_data.session_body().token_data().to_owned(),
        refresh_token,
        openid: auth_data.session_body().user_id().to_string(),
        scope: req_dao
            .web_dao
            .web_app
            .app_dao
            .oauth_client
            .get_session_scope_data(&auth_data)
            .await?,
        expires_in: auth_data.session_body().session().expire_time,
    };
    Ok(JsonResponse::data(JsonData::body(json!(session))))
}

#[derive(Debug, Deserialize)]
pub struct RefreshCodeParam {
    pub client_secret: String,
    pub client_id: String,
    pub refresh_token: String,
}

//刷新登陆token
pub async fn refresh_token(
    param: &RefreshCodeParam,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    let app = check_app_secret(req_dao, &param.client_id, &param.client_secret).await?;
    let new_token = req_dao
        .web_dao
        .web_app
        .app_dao
        .oauth_client
        .refresh_session(&app, &param.refresh_token)
        .await?;
    let session = SessionRecord {
        access_token: new_token.session_body().token_data().to_owned(),
        refresh_token: param.refresh_token.to_owned(),
        openid: new_token.session_body().user_id().to_string(),
        scope: req_dao
            .web_dao
            .web_app
            .app_dao
            .oauth_client
            .get_session_scope_data(&new_token)
            .await?,
        expires_in: new_token.session_body().session().expire_time,
    };
    Ok(JsonResponse::data(JsonData::body(json!(session))))
}
