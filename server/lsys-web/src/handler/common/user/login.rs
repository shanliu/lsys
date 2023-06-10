use crate::{
    dao::RequestDao,
    {JsonData, JsonResult, PageParam},
};
use lsys_user::dao::auth::{SessionData, SessionTokenData, UserSession};

use serde::Deserialize;
use serde_json::json;

/// logout
pub async fn user_logout<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    req_dao.user_session.write().await.clear_session().await?;
    Ok(JsonData::message("logout ok"))
}

#[derive(Debug, Deserialize)]
pub struct LoginHistoryParam {
    pub login_type: Option<String>,
    pub login_account: Option<String>,
    pub login_ip: Option<String>,
    pub is_login: Option<i8>,
    pub page: Option<PageParam>,
}
pub async fn user_login_history<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: LoginHistoryParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let data = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user_login
        .history_data(
            Some(auth_data.user_data().user_id),
            param.login_account,
            param.is_login,
            param.login_type.clone(),
            param.login_ip.clone(),
            &Some(param.page.unwrap_or_default().into()),
        )
        .await?;
    let total = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user_login
        .history_count(
            Some(auth_data.user_data().user_id),
            None,
            param.is_login,
            param.login_type,
        )
        .await?;
    Ok(JsonData::data(json!({
        "data": data ,
        "total":total,
    })))
}
