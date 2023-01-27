use crate::{
    dao::{access::AppScope, RequestDao, RestAuthQueryDao, WebDao},
    {JsonData, JsonResult},
};
use lsys_app::model::AppsModel;
use lsys_user::dao::auth::{SessionData, SessionTokenData, UserSession};
use serde::{Deserialize, Serialize};
use serde_json::json;

//指定app跟对应scope权限校验跟获取
async fn get_scope<'a>(
    web_dao: &'a WebDao,
    app: &AppsModel,
    scope: &'a str,
) -> JsonResult<AppScope<'a>> {
    let scope = AppScope::try_from(scope)?;
    let res = scope.to_check_res();
    web_dao
        .user
        .rbac_dao
        .rbac
        .access
        .check(
            app.user_id,
            &[web_dao.app.app_relation_key(app).await],
            &res,
        )
        .await?;
    Ok(scope)
}

#[derive(Debug, Deserialize)]
pub struct OauthScopeGetParam {
    pub client_id: String,
    pub scope: String,
}
//当前登陆scope对应的功能
pub async fn oauth_scope_get<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: OauthScopeGetParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    req_dao.user_session.read().await.get_session_data().await?;
    let app = req_dao
        .web_dao
        .app
        .app_dao
        .app
        .cache()
        .find_by_client_id(&param.client_id)
        .await?;
    let scope = get_scope(&req_dao.web_dao, &app, &param.scope).await?;
    Ok(JsonData::message("token code").set_data(json!({ "scope": scope.to_show_data()})))
}

#[derive(Debug, Deserialize)]
pub struct OauthAuthorizeDoParam {
    pub scope: String,
    pub client_id: String,
    pub redirect_uri: String,
}
//登陆code创建
pub async fn oauth_create_code<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: OauthAuthorizeDoParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    //   用户授权 scope跟资源静态编码关系 检查授权通过scope得到res检查全局授权 接口检查授权跟检查资源是否在token的scope中
    //   1. 请求用户 /oauth/authorize?client_id=app_id&redirect_uri=CALLBACK_URL&scope=read
    //   2. 根据scope查授权[关系key]，通过，显示登录,完成登录,跳到授权页面 以 scope 查询资源列表[还没有]
    //   3. 完成授权. 生成code ,存放redis, scope,授权时间 +client_id+授权user_id ,设置5分钟超时,回到用户站点 /callback?code=AUTHORIZATION_CODE
    //   4. 请求令牌 /oauth/token?client_id=CLIENT_ID&client_secret=CLIENT_SECRET&code=AUTHORIZATION_CODE
    //   5. 读取redis 判断client_id,生成token记录并放入本地缓存
    let user = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user
        .find_by_id(
            &req_dao
                .user_session
                .read()
                .await
                .get_session_data()
                .await?
                .user_data()
                .user_id,
        )
        .await?;
    let app = req_dao
        .web_dao
        .app
        .app_dao
        .app
        .find_by_client_id(&param.client_id)
        .await?;
    if app.callback_domain.is_empty() {
        return Err(JsonData::message("not config callback domain").set_code("domain_empty"));
    }
    if !param
        .redirect_uri
        .starts_with(&("https://".to_string() + &app.callback_domain))
        && !param
            .redirect_uri
            .starts_with(&("http://".to_string() + &app.callback_domain))
    {
        return Err(JsonData::message("redirect_uri not match").set_code("domain_no_match"));
    }
    get_scope(&req_dao.web_dao, &app, &param.scope).await?;
    let code = req_dao
        .web_dao
        .app
        .app_dao
        .app_oauth
        .create_code(&app, &param.scope, user.id)
        .await?;
    Ok(JsonData::message("token code").set_data(json!({ "code": code })))
}

#[derive(Debug, Deserialize)]
pub struct OauthCodeParam {
    pub client_secret: String,
    pub client_id: String,
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct OauthRefreshCodeParam {
    pub client_secret: String,
    pub client_id: String,
    pub refresh_token: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OauthSessionData {
    access_token: String,
    refresh_token: Option<String>,
    openid: String,
    scope: String,
    expires_in: String,
}
//创建登陆token
pub async fn oauth_create_token(webdao: &WebDao, code: OauthCodeParam) -> JsonResult<JsonData> {
    let app = webdao
        .app
        .app_dao
        .app
        .cache()
        .find_by_client_id(&code.client_id)
        .await?;
    if code.client_secret != app.client_secret {
        return Err(JsonData::message_error("client_secret not match"));
    }
    let (token, user) = webdao
        .app
        .app_dao
        .app_oauth
        .create_token(&app, code.code)
        .await?;
    webdao
        .app
        .app_dao
        .app_oauth
        .create_session(&app, &token, &user)
        .await?;
    let session = OauthSessionData {
        access_token: token.token,
        refresh_token: None,
        openid: token.access_user_id.to_string(),
        scope: token.scope,
        expires_in: token.timeout.to_string(),
    };
    Ok(JsonData::message("token data").set_data(json!({ "token": session })))
}

//刷新登陆token
pub async fn oauth_refresh_token(req_dao: &RestAuthQueryDao) -> JsonResult<JsonData> {
    let mut auth_data = req_dao.user_session.write().await;
    let old_token = auth_data.get_session_data().await?;
    auth_data.refresh_session(true).await?;
    let new_token = auth_data.get_session_data().await?;
    let token = new_token.token;
    let session = OauthSessionData {
        access_token: token.token,
        refresh_token: Some(old_token.token.token),
        openid: token.access_user_id.to_string(),
        scope: token.scope,
        expires_in: token.timeout.to_string(),
    };
    Ok(JsonData::message("token data").set_data(json!({ "token": session })))
}
