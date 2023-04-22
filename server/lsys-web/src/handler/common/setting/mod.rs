use crate::{dao::RequestDao, handler::access::AccessSiteSetting, JsonData, JsonResult};
use lsys_setting::dao::NotFoundDefault;
use lsys_setting::dao::{SettingDecode, SettingEncode, SettingKey};
use lsys_user::dao::auth::{SessionData, SessionTokenData, UserSession};
use serde::{Deserialize, Serialize};
use serde_json::json;
pub async fn setting_set<
    'a,
    P: Deserialize<'a>,
    A: SettingKey + SettingDecode + SettingEncode + From<P>,
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    param: P,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    //验证权限
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
        .await?;
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
        )
        .await
        .map_err(|e| e.to_string())?;
    Ok(JsonData::message("ok"))
}

pub async fn setting_get<
    'a,
    A: SettingKey + SettingDecode + SettingEncode + Serialize + Default,
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
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
        .await?;
    let data = req_dao
        .web_dao
        .setting
        .single
        .load::<A>(&None)
        .await
        .notfound_default()?;
    Ok(JsonData::data(json!({ "config":  &*data })))
}

mod site_config;
pub use site_config::*;
