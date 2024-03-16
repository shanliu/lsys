use crate::{
    dao::{RequestAuthDao, RequestDao, RestAuthQueryDao},
    handler::access::{
        AccessOauthUserAddress, AccessOauthUserEmail, AccessOauthUserInfo, AccessOauthUserMobile,
    },
    {JsonData, JsonResult},
};
use lsys_app::model::AppsModel;
use lsys_core::fluent_message;
use lsys_user::dao::auth::{SessionData, SessionTokenData, UserSession};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize)]
pub struct ScopeItem {
    pub name: &'static str,
    pub key: &'static str,
}
//指定app跟对应scope权限校验跟获取
async fn get_scope<'a>(
    req_dao: &'a RequestDao,
    app: &AppsModel,
    scope: &'a str,
) -> JsonResult<Vec<ScopeItem>> {
    let spoces = scope.split(',').collect::<Vec<&str>>();
    let mut out = vec![];
    for tmp in spoces {
        let rbac = &req_dao.web_dao.user.rbac_dao.rbac;
        let data = match tmp {
            "user_info" => {
                rbac.check(
                    &AccessOauthUserInfo {
                        app_id: app.id,
                        user_id: app.user_id,
                    },
                    None,
                )
                .await
                .map_err(|e| req_dao.fluent_json_data(e))?;
                ScopeItem {
                    name: "用户资料",
                    key: "user_info",
                }
            }
            "user_email" => {
                rbac.check(
                    &AccessOauthUserEmail {
                        app_id: app.id,
                        user_id: app.user_id,
                    },
                    None,
                )
                .await
                .map_err(|e| req_dao.fluent_json_data(e))?;
                ScopeItem {
                    name: "用户邮箱",
                    key: "user_email",
                }
            }
            "user_mobile" => {
                rbac.check(
                    &AccessOauthUserMobile {
                        app_id: app.id,
                        user_id: app.user_id,
                    },
                    None,
                )
                .await
                .map_err(|e| req_dao.fluent_json_data(e))?;
                ScopeItem {
                    name: "用户手机号",
                    key: "user_mobile",
                }
            }
            "user_address" => {
                rbac.check(
                    &AccessOauthUserAddress {
                        app_id: app.id,
                        user_id: app.user_id,
                    },
                    None,
                )
                .await
                .map_err(|e| req_dao.fluent_json_data(e))?;
                ScopeItem {
                    name: "用户收货地址",
                    key: "user_address",
                }
            }
            _ => return Err(JsonData::message(format!("not support {}", tmp))),
        };
        out.push(data);
    }
    Ok(out)
}

#[derive(Debug, Deserialize)]
pub struct OauthScopeGetParam {
    pub client_id: String,
    pub scope: String,
}
//当前登陆scope对应的功能
pub async fn oauth_scope_get<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: OauthScopeGetParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let app = req_dao
        .web_dao
        .app
        .app_dao
        .app
        .cache()
        .find_by_client_id(&param.client_id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let scope = get_scope(req_dao, &app, &param.scope).await?;
    Ok(JsonData::data(json!({ "scope": scope })))
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
    req_dao: &RequestAuthDao<T, D, S>,
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
                .await
                .map_err(|e| req_dao.fluent_json_data(e))?
                .user_data()
                .user_id,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let app = req_dao
        .web_dao
        .app
        .app_dao
        .app
        .find_by_client_id(&param.client_id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    if app.callback_domain.is_empty() {
        return Err(
            req_dao
                .fluent_json_data(fluent_message!("app-domain-not-config"))
                .set_sub_code("domain_empty"), // JsonData::message("not config callback domain").set_sub_code("")
        );
    }
    if !param
        .redirect_uri
        .starts_with(&("https://".to_string() + &app.callback_domain))
        && !param
            .redirect_uri
            .starts_with(&("http://".to_string() + &app.callback_domain))
    {
        return Err(
            req_dao
                .fluent_json_data(fluent_message!("app-redirect-uri-not-match"))
                .set_sub_code("domain_no_match"), // JsonData::message("redirect_uri not match")
        );
    }
    get_scope(req_dao, &app, &param.scope).await?;
    let code = req_dao
        .web_dao
        .app
        .app_dao
        .app_oauth
        .create_code(&app, &param.scope, user.id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::data(json!({ "code": code })))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OauthSessionData {
    access_token: String,
    refresh_token: Option<String>,
    openid: String,
    scope: String,
    expires_in: u64,
}

async fn check_app_secret(
    req_dao: &RequestDao,
    client_id: &String,
    client_secret: &String,
) -> JsonResult<AppsModel> {
    let app = req_dao
        .web_dao
        .app
        .app_dao
        .app
        .cache()
        .find_by_client_id(client_id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    if *client_secret != app.oauth_secret {
        return Err(req_dao.fluent_json_data(fluent_message!("client-secret-not-match")));
    }
    Ok(app)
}

#[derive(Debug, Deserialize)]
pub struct OauthCodeParam {
    pub client_secret: String,
    pub client_id: String,
    pub code: String,
}

//创建登陆token
pub async fn oauth_create_token(
    req_dao: &RequestDao,
    code: OauthCodeParam,
) -> JsonResult<JsonData> {
    let app = check_app_secret(req_dao, &code.client_id, &code.client_secret).await?;
    let (token, user) = req_dao
        .web_dao
        .app
        .app_dao
        .app_oauth
        .create_token(&app, code.code)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .web_dao
        .app
        .app_dao
        .app_oauth
        .create_session(&app, &token, &user)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let session = OauthSessionData {
        access_token: token.token,
        refresh_token: None,
        openid: token.access_user_id.to_string(),
        scope: token.scope,
        expires_in: token.timeout,
    };
    Ok(JsonData::data(json!(session)))
}

#[derive(Debug, Deserialize)]
pub struct OauthRefreshCodeParam {
    pub client_secret: String,
    pub client_id: String,
    pub refresh_token: String,
}

//刷新登陆token
pub async fn oauth_refresh_token(
    req_dao: &RestAuthQueryDao,
    param: OauthRefreshCodeParam,
) -> JsonResult<JsonData> {
    check_app_secret(req_dao, &param.client_id, &param.client_secret).await?;
    let mut auth_data = req_dao.user_session.write().await;
    let old_token = auth_data
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    auth_data
        .refresh_session(true)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let new_token = auth_data
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let token = new_token.token;
    let session = OauthSessionData {
        access_token: token.token,
        refresh_token: Some(old_token.token.token),
        openid: token.access_user_id.to_string(),
        scope: token.scope,
        expires_in: token.timeout,
    };
    Ok(JsonData::data(json!(session)))
}
