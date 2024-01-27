use crate::{
    dao::RequestAuthDao,
    handler::access::{AccessAdminSmsConfig, AccessAppSenderSmsConfig},
    {JsonData, JsonResult},
};
use lsys_sender::dao::TenYunConfig;
use lsys_setting::dao::SettingData;
use lsys_user::dao::auth::{SessionData, SessionTokenData, UserSession};
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;
#[derive(Debug, Deserialize)]
pub struct SmserTenConfigListParam {
    pub ids: Option<Vec<u64>>,
    pub full_data: Option<bool>,
}

#[derive(Serialize)]
pub struct ShowTenYunConfig {
    pub id: u64,
    pub name: String,
    pub region: String,
    pub secret_id: String,
    pub hide_secret_id: String,
    pub sms_app_id: String,
    pub secret_key: String,
    pub change_user_id: u64,
    pub callback_url: String,
    pub change_time: u64,
    pub callback_key: String,
    pub limit: u16,
}

pub async fn smser_ten_config_list<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: SmserTenConfigListParam,
    callback_call: impl Fn(&SettingData<TenYunConfig>) -> String,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let tensender = &req_dao.web_dao.sender_smser.tenyun_sender;
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
            .map(|e| ShowTenYunConfig {
                id: e.model().id,
                region: e.region.to_owned(),
                name: e.model().name.to_owned(),
                secret_id: e.secret_id.to_owned(),
                hide_secret_id: e.hide_secret_id(),
                secret_key: e.secret_key.to_owned(),
                change_user_id: e.model().change_user_id,
                change_time: e.model().change_time,
                callback_url: callback_call(&e),
                sms_app_id: e.sms_app_id.to_owned(),
                limit: e.branch_limit,
                callback_key: e.callback_key.to_owned(),
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
                   "app_id":e.hide_secret_id()
                })
            })
            .collect::<Vec<Value>>();
        json!({ "data": row })
    };
    Ok(JsonData::data(out))
}

#[derive(Debug, Deserialize)]
pub struct SmserTenConfigAddParam {
    pub name: String,
    pub region: String,
    pub secret_id: String,
    pub secret_key: String,
    pub sms_app_id: String,
    pub limit: Option<u16>,
    pub callback_key: String,
}

pub async fn smser_ten_config_add<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: SmserTenConfigAddParam,
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
    let tensender = &req_dao.web_dao.sender_smser.tenyun_sender;
    let row = tensender
        .add_config(
            &param.name,
            &param.region,
            &param.secret_id,
            &param.secret_key,
            &param.sms_app_id,
            &param.limit.unwrap_or_default(),
            &param.callback_key,
            &req_auth.user_data().user_id,
            Some(&req_dao.req_env),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::data(json!({ "id": row })))
}

#[derive(Debug, Deserialize)]
pub struct SmserTenConfigEditParam {
    pub id: u64,
    pub name: String,
    pub region: String,
    pub secret_id: String,
    pub secret_key: String,
    pub sms_app_id: String,
    pub limit: Option<u16>,
    pub callback_key: String,
}

pub async fn smser_ten_config_edit<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: SmserTenConfigEditParam,
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
    let tensender = &req_dao.web_dao.sender_smser.tenyun_sender;
    let row = tensender
        .edit_config(
            &param.id,
            &param.name,
            &param.region,
            &param.secret_id,
            &param.secret_key,
            &param.sms_app_id,
            &param.limit.unwrap_or_default(),
            &param.callback_key,
            &req_auth.user_data().user_id,
            Some(&req_dao.req_env),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::data(json!({ "num": row })))
}

#[derive(Debug, Deserialize)]
pub struct SmserTenConfigDelParam {
    pub id: u64,
}

pub async fn smser_ten_config_del<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: SmserTenConfigDelParam,
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
    let tensender = &req_dao.web_dao.sender_smser.tenyun_sender;
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
pub struct SmserAppTenConfigAddParam {
    pub app_id: u64,
    pub user_id: Option<u64>,
    pub config_id: u64,
    pub name: String,
    pub tpl_id: String,
    pub sign_name: String,
    pub template_id: String,
    pub template_map: String,
}

pub async fn smser_ten_app_config_add<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: SmserAppTenConfigAddParam,
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
    let tensender = &req_dao.web_dao.sender_smser.tenyun_sender;

    let row = tensender
        .add_app_config(
            &param.name,
            &param.app_id,
            &param.config_id,
            &param.tpl_id,
            &param.sign_name,
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
