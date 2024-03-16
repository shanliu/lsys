use crate::{
    dao::RequestAuthDao,
    handler::access::{AccessAdminSmsConfig, AccessAppSenderSmsConfig},
    {JsonData, JsonResult},
};
use lsys_app_sender::dao::NetEaseConfig;
use lsys_setting::dao::SettingData;
use lsys_user::dao::auth::{SessionData, SessionTokenData, UserSession};
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;
#[derive(Debug, Deserialize)]
pub struct SmserNetEaseConfigListParam {
    pub ids: Option<Vec<u64>>,
    pub full_data: Option<bool>,
}

#[derive(Serialize)]
pub struct ShowNeteaseConfig {
    pub id: u64,
    pub name: String,
    pub access_key: String,
    pub hide_access_key: String,
    pub access_secret: String,
    pub change_user_id: u64,
    pub change_time: u64,
    pub callback_url: String,
    pub limit: u16,
}

pub async fn smser_netease_config_list<
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    param: SmserNetEaseConfigListParam,
    callback_call: impl Fn(&SettingData<NetEaseConfig>) -> String,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let tensender = &req_dao.web_dao.sender_smser.netease_sender;
    let row = tensender
        .list_config(&param.ids)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let out = if param.full_data.unwrap_or(false) {
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
                &AccessAdminSmsConfig {
                    user_id: req_auth.user_data().user_id,
                },
                None,
            )
            .await
            .map_err(|e| req_dao.fluent_json_data(e))?;
        let tmp = row
            .into_iter()
            .map(|e| ShowNeteaseConfig {
                id: e.model().id,
                name: e.model().name.to_owned(),
                access_key: e.access_key.to_owned(),
                hide_access_key: e.hide_access_key(),
                access_secret: e.access_secret.to_owned(),
                change_user_id: e.model().change_user_id,
                change_time: e.model().change_time,
                callback_url: callback_call(&e),
                limit: e.branch_limit,
            })
            .collect::<Vec<_>>();
        json!({ "data": tmp })
    } else {
        let row = row
            .into_iter()
            .map(|e| {
                json!({
                   "id": e.model().id,
                   "name": e.model().name,
                   "app_id":e.hide_access_key()
                })
            })
            .collect::<Vec<Value>>();
        json!({ "data": row })
    };
    Ok(JsonData::data(out))
}

#[derive(Debug, Deserialize)]
pub struct SmserNetEaseConfigAddParam {
    pub name: String,
    pub access_key: String,
    pub access_secret: String,
    pub limit: Option<u16>,
}

pub async fn smser_netease_config_add<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: SmserNetEaseConfigAddParam,
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
            &AccessAdminSmsConfig {
                user_id: req_auth.user_data().user_id,
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let tensender = &req_dao.web_dao.sender_smser.netease_sender;
    let row = tensender
        .add_config(
            &param.name,
            &param.access_key,
            &param.access_secret,
            &param.limit.unwrap_or_default(),
            &req_auth.user_data().user_id,
            Some(&req_dao.req_env),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::data(json!({ "id": row })))
}

#[derive(Debug, Deserialize)]
pub struct SmserNetEaseConfigEditParam {
    pub id: u64,
    pub name: String,
    pub access_key: String,
    pub access_secret: String,
    pub limit: Option<u16>,
}

pub async fn smser_netease_config_edit<
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    param: SmserNetEaseConfigEditParam,
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
            &AccessAdminSmsConfig {
                user_id: req_auth.user_data().user_id,
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let tensender = &req_dao.web_dao.sender_smser.netease_sender;
    let row = tensender
        .edit_config(
            &param.id,
            &param.name,
            &param.access_key,
            &param.access_secret,
            &param.limit.unwrap_or_default(),
            &req_auth.user_data().user_id,
            Some(&req_dao.req_env),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::data(json!({ "num": row })))
}

#[derive(Debug, Deserialize)]
pub struct SmserNetEaseConfigDelParam {
    pub id: u64,
}

pub async fn smser_netease_config_del<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: SmserNetEaseConfigDelParam,
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
            &AccessAdminSmsConfig {
                user_id: req_auth.user_data().user_id,
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let tensender = &req_dao.web_dao.sender_smser.netease_sender;
    let row = tensender
        .del_config(
            &param.id,
            &req_auth.user_data().user_id,
            Some(&req_dao.req_env),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::data(json!({ "num": row })))
}

#[derive(Debug, Deserialize)]
pub struct SmserAppNetEaseConfigAddParam {
    pub app_id: u64,
    pub user_id: Option<u64>,
    pub config_id: u64,
    pub name: String,
    pub tpl_id: String,
    pub template_id: String,
    pub template_map: String,
}

pub async fn smser_netease_app_config_add<
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    param: SmserAppNetEaseConfigAddParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let uid = param.user_id.unwrap_or(req_auth.user_data().user_id);

    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessAppSenderSmsConfig {
                user_id: req_auth.user_data().user_id,
                res_user_id: uid,
                app_id: param.app_id,
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let tensender = &req_dao.web_dao.sender_smser.netease_sender;

    let row = tensender
        .add_app_config(
            &param.name,
            &param.app_id,
            &param.config_id,
            &param.tpl_id,
            &param.template_id,
            &param.template_map,
            &uid,
            &req_auth.user_data().user_id,
            Some(&req_dao.req_env),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::data(json!({ "id": row })))
}
