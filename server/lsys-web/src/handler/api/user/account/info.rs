use crate::common::JsonData;
use crate::{
    common::{JsonResponse, JsonResult, UserAuthQueryDao},
    dao::{access::api::system::user::CheckUserInfoEdit, InfoSetUserInfoData},
};
use lsys_access::dao::AccessSession;
use serde::Deserialize;
use serde_json::json;
use crate::dao::access::RbacAccessCheckEnv;
#[derive(Debug, Deserialize)]
pub struct InfoSetUserNameParam {
    pub name: String,
}
pub async fn info_set_username(
    param: &InfoSetUserNameParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserInfoEdit {
                res_user_id: auth_data.user_id(),
            },
        )
        .await?;

    req_dao
        .web_dao
        .web_user
        .account
        .user_info_set_username(&param.name, &req_dao.user_session, Some(&req_dao.req_env))
        .await?;

    Ok(JsonResponse::default())
}

#[derive(Debug, Deserialize)]
pub struct InfoCheckUserNameParam {
    pub name: String,
}
pub async fn info_check_username(
    param: &InfoCheckUserNameParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserInfoEdit {
                res_user_id: auth_data.user_id(),
            },
        )
        .await?;
    req_dao
        .web_dao
        .web_user
        .account
        .user_info_check_username(&param.name)
        .await?;
    Ok(JsonResponse::default().set_data(JsonData::body(json!({
        "pass":"1"
    }))))
}

#[derive(Debug, Deserialize)]
pub struct InfoSetUserInfoParam {
    pub nikename: Option<String>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_i32")]
    pub gender: Option<i32>,
    pub headimg: Option<String>,
    pub birthday: Option<String>,
}
pub async fn info_set_data(
    param: &InfoSetUserInfoParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserInfoEdit {
                res_user_id: auth_data.user_id(),
            },
        )
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

    Ok(JsonResponse::default())
}

pub async fn password_last_modify(req_dao: &UserAuthQueryDao) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    let (passwrod, passwrod_timeout) = req_dao
        .web_dao
        .web_user
        .account
        .password_last_modify(&auth_data)
        .await?;

    Ok(JsonResponse::data(JsonData::body(json!({
        "last_time":passwrod.add_time,
        "password_timeout":passwrod_timeout,
    }))))
}
