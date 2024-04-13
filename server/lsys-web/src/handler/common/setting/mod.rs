use crate::{dao::RequestAuthDao, handler::access::AccessSiteSetting, JsonData, JsonResult};
use lsys_setting::dao::NotFoundResult;
use lsys_setting::dao::{SettingDecode, SettingEncode, SettingKey};
use lsys_user::dao::auth::{SessionData, SessionTokenData, UserSession};
use serde::{Deserialize, Serialize};
use serde_json::json;


mod setting_site;
pub use setting_site::*;



pub async fn setting_set<
    'a,
    P: Deserialize<'a>,
    A: SettingKey + SettingDecode + SettingEncode + From<P>,
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    param: P,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?; //验证权限
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessSiteSetting {
                user_id: req_auth.user_data().user_id,
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .web_dao
        .setting
        .single
        .save::<A>(
            &None,
            A::key(),
            &A::from(param),
            &req_auth.user_data().user_id,
            None,
            Some(&req_dao.req_env),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::default())
}

pub async fn setting_get<
    'a,
    A: SettingKey + SettingDecode + SettingEncode + Serialize + Default,
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
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
            &AccessSiteSetting {
                user_id: req_auth.user_data().user_id,
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let data = req_dao
        .web_dao
        .setting
        .single
        .load::<A>(&None)
        .await
        .notfound_default()
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::data(json!({ "config":  &*data })))
}
