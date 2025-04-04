use crate::{
    common::{JsonData, JsonResult, UserAuthQueryDao},
    dao::{access::api::user::CheckUserInfoEdit, InfoSetUserInfoData},
};
use lsys_access::dao::AccessSession;
use serde::Deserialize;
use serde_json::json;
#[derive(Debug, Deserialize)]
pub struct InfoSetUserNameParam {
    pub name: String,
}
pub async fn info_set_username(
    param: &InfoSetUserNameParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
  
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckUserInfoEdit {})
        .await?;

    req_dao
        .web_dao
        .web_user
        .account
        .user_info_set_username(&param.name, &req_dao.user_session, Some(&req_dao.req_env))
        .await?;

    Ok(JsonData::default())
}

#[derive(Debug, Deserialize)]
pub struct InfoCheckUserNameParam {
    pub name: String,
}
pub async fn info_check_username(
    param: &InfoCheckUserNameParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
  
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckUserInfoEdit {})
        .await?;
    req_dao
        .web_dao
        .web_user
        .account
        .user_info_check_username(&param.name)
        .await?;
    Ok(JsonData::default().set_data(json!({
        "pass":"1"
    })))
}

#[derive(Debug, Deserialize)]
pub struct InfoSetUserInfoParam {
    pub nikename: Option<String>,
    pub gender: Option<i32>,
    pub headimg: Option<String>,
    pub birthday: Option<String>,
}
pub async fn info_set_data(
    param: &InfoSetUserInfoParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
  
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckUserInfoEdit {})
        .await?;

    req_dao
        .web_dao
        .web_user
        .account
        .user_info_set_data(
            &InfoSetUserInfoData {
                nikename: param.nikename.as_deref(),
                gender: param.gender,
                headimg: param.headimg.as_deref(),
                birthday: param.birthday.as_deref(),
            },
            &req_dao.user_session,
            Some(&req_dao.req_env),
        )
        .await?;

    Ok(JsonData::default())
}

pub async fn password_last_modify(req_dao: &UserAuthQueryDao) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    let (passwrod, passwrod_timeout) = req_dao
        .web_dao
        .web_user
        .account
        .password_last_modify(&auth_data)
        .await?;

    Ok(JsonData::data(json!({
        "last_time":passwrod.add_time,
        "password_timeout":passwrod_timeout,
    })))
}
