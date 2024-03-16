use crate::{
    dao::RequestAuthDao,
    handler::access::{AccessUserInfoEdit, AccessUserNameEdit},
    {JsonData, JsonResult},
};
use lsys_core::fluent_message;
use lsys_user::dao::auth::{SessionData, SessionTokenData, UserSession};
use lsys_user::{dao::account::UserAccountError, model::UserInfoModelRef};
use serde::Deserialize;
use serde_json::json;
use sqlx_model::model_option_set;
#[derive(Debug, Deserialize)]
pub struct InfoSetUserNameParam {
    pub name: String,
}
pub async fn user_info_set_username<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: InfoSetUserNameParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessUserNameEdit {
                user_id: req_auth.user_data().user_id,
                res_user_id: req_auth.user_data().user_id,
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let user = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user
        .find_by_id(&req_auth.user_data().user_id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user_name
        .change_username(&user, param.name, None, Some(&req_dao.req_env))
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .user_session
        .write()
        .await
        .refresh_session(false)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::default())
}

#[derive(Debug, Deserialize)]
pub struct InfoCheckUserNameParam {
    pub name: String,
}
pub async fn user_info_check_username<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: InfoCheckUserNameParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessUserNameEdit {
                user_id: req_auth.user_data().user_id,
                res_user_id: req_auth.user_data().user_id,
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let user_res = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user_name
        .find_by_name(param.name)
        .await;
    match user_res {
        Err(UserAccountError::Sqlx(sqlx::Error::RowNotFound)) => {
            Ok(JsonData::default().set_data(json!({
                "pass":"1"
            })))
        }
        Err(err) => Err(req_dao.fluent_json_data(err)),
        Ok(user) => Ok(
            req_dao
                .fluent_json_data(fluent_message!("username-is-exists",{
                    "id":user.id
                }))
                .set_sub_code("username_exists"), // JsonData::message(format!("Username already exists [{}]", user.id))
                                                  //     ,
        ),
    }
}

#[derive(Debug, Deserialize)]
pub struct InfoSetUserInfoParam {
    pub nikename: Option<String>,
    pub gender: Option<i32>,
    pub headimg: Option<String>,
    pub birthday: Option<String>,
}
pub async fn user_info_set_data<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: InfoSetUserInfoParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessUserInfoEdit {
                user_id: req_auth.user_data().user_id,
                res_user_id: req_auth.user_data().user_id,
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let user = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user
        .find_by_id(&req_auth.user_data().user_id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let mut db = req_dao
        .web_dao
        .db
        .begin()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    if let Some(nikename) = param.nikename {
        let res = req_dao
            .web_dao
            .user
            .user_dao
            .user_account
            .user
            .set_nikename(&user, nikename, Some(&mut db), Some(&req_dao.req_env))
            .await;
        if let Err(err) = res {
            db.rollback()
                .await
                .map_err(|e| req_dao.fluent_json_data(e))?;
            return Err(req_dao.fluent_json_data(err));
        }
    }
    let mut info = model_option_set!(UserInfoModelRef, {});
    info.gender = param.gender.as_ref();
    info.headimg = param.headimg.as_ref();
    info.birthday = param.birthday.as_ref();
    let res = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user_info
        .set_info(&user, info, Some(&mut db), Some(&req_dao.req_env))
        .await;
    if let Err(err) = res {
        db.rollback()
            .await
            .map_err(|e| req_dao.fluent_json_data(e))?;
        return Err(req_dao.fluent_json_data(err));
    }
    db.commit().await.map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .user_session
        .write()
        .await
        .refresh_session(false)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::default())
}

pub async fn password_last_modify<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessUserInfoEdit {
                user_id: req_auth.user_data().user_id,
                res_user_id: req_auth.user_data().user_id,
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let user = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user
        .find_by_id(&req_auth.user_data().user_id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    if user.password_id == 0 {
        return Ok(req_dao
            .fluent_json_data(fluent_message!("password-not-set"))
            .set_sub_code("not_set"));
    }
    let user = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user_password
        .find_by_id(&user.password_id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let passwrod_timeout = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user_password
        .password_timeout(&user.id)
        .await
        .unwrap_or(false);
    Ok(JsonData::data(json!({
        "last_time":user.add_time,
        "password_timeout":passwrod_timeout,
    })))
}

pub async fn user_delete<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let user = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user
        .find_by_id(&req_auth.user_data().user_id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .web_dao
        .user
        .del_user(&user, Some(&req_dao.req_env))
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let _ = req_dao.user_session.write().await.clear_session().await;
    Ok(JsonData::default())
}
