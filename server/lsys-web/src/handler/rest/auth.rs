use std::collections::HashMap;

use crate::{
    common::{JsonData, JsonResult, RequestDao},
    dao::access::rest::CheckRestApp,
};
use lsys_access::dao::AccessLoginData;
use lsys_app::model::AppModel;

use lsys_core::now_time;
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Debug, Deserialize)]
pub struct DoLoginParam {
    pub token_code: String,
    pub user_data: String,
    pub user_name: String,
    pub expire_time: u64,
    pub device_name: Option<String>,
    pub user_account: Option<String>,
    pub login_ip: Option<String>,
    pub device_id: Option<String>,
    pub session_data: Option<HashMap<String, Value>>,
}
pub async fn do_login(
    param: &DoLoginParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, None, &CheckRestApp { app_id: app.id })
        .await?;
    req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .inner_feature_exter_login_check(app)
        .await?;

    let session_data = param
        .session_data
        .as_ref()
        .map(|t| {
            t.iter()
                .map(|(k, v)| (k, v.to_string()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let seession_body = req_dao
        .web_dao
        .web_user
        .user_dao
        .auth_code_dao
        .code_login(
            app.id,
            &param.token_code,
            &param.user_data,
            &param.user_name,
            &AccessLoginData {
                user_account: param.user_account.as_deref(),
                login_ip: param.login_ip.as_deref(),
                device_id: param.device_id.as_deref(),
                device_name: param.device_name.as_deref(),
                expire_time: now_time().unwrap_or_default() + param.expire_time,
                session_data: session_data
                    .iter()
                    .map(|e| (e.0.as_str(), e.1.as_str()))
                    .collect::<Vec<_>>(),
            },
        )
        .await?;
    Ok(JsonData::data(json!({
        "token_data": seession_body.token_data(),
        "user_id": seession_body.user_id(),
        "user_name": seession_body.user().user_name,
    })))
}

#[derive(Debug, Deserialize)]
pub struct DoLogoutParam {
    token_data: String,
}
pub async fn do_logout(
    param: &DoLogoutParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .inner_feature_exter_login_check(app)
        .await?;
    req_dao
        .web_dao
        .web_user
        .user_dao
        .auth_code_dao
        .code_logout(app.id, &param.token_data)
        .await?;

    Ok(JsonData::default())
}

#[derive(Debug, Deserialize)]
pub struct LoginInfoParam {
    pub token_data: String,
}
pub async fn login_info(
    param: &LoginInfoParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, None, &CheckRestApp { app_id: app.id })
        .await?;
    req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .inner_feature_exter_login_check(app)
        .await?;

    let session = req_dao
        .web_dao
        .web_user
        .user_dao
        .auth_code_dao
        .login_data(app.id, &param.token_data)
        .await?;

    Ok(JsonData::data(json!({
        "session":session.session(),
        "user":session.user()
    })))
}
