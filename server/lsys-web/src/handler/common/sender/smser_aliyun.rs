use crate::{
    dao::RequestAuthDao,
    handler::access::{AccessAdminSmsConfig, AccessAppSenderSmsConfig},
    {JsonData, JsonResult},
};
use lsys_app_sender::dao::AliYunConfig;
use lsys_setting::dao::SettingData;
use lsys_user::dao::auth::{SessionData, SessionTokenData, UserSession};
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;
#[derive(Debug, Deserialize)]
pub struct SmserAliConfigListParam {
    pub ids: Option<Vec<u64>>,
    pub full_data: Option<bool>,
}

#[derive(Serialize)]
pub struct ShowAliYunConfig {
    pub id: u64,
    pub name: String,
    pub region: String,
    pub access_id: String,
    pub hide_access_id: String,
    pub access_secret: String,
    pub change_user_id: u64,
    pub change_time: u64,
    pub limit: u16,
    pub callback_url: String,
    pub callback_key: String,
}

pub async fn smser_ali_config_list<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: SmserAliConfigListParam,
    callback_call: impl Fn(&SettingData<AliYunConfig>) -> String,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let alisender = &req_dao.web_dao.sender_smser.aliyun_sender;
    let row = alisender
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
            .map(|e| ShowAliYunConfig {
                id: e.model().id,
                name: e.model().name.to_owned(),
                access_id: e.access_id.to_owned(),
                region: e.region.to_owned(),
                hide_access_id: e.hide_access_id(),
                access_secret: e.access_secret.to_owned(),
                change_user_id: e.model().change_user_id,
                change_time: e.model().change_time,
                limit: e.branch_limit,
                callback_url: callback_call(&e),
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
                   "app_id":e.hide_access_id()
                })
            })
            .collect::<Vec<Value>>();
        json!({ "data": row })
    };
    Ok(JsonData::data(out))
}

#[derive(Debug, Deserialize)]
pub struct SmserAliConfigAddParam {
    pub name: String,
    pub access_id: String,
    pub access_secret: String,
    pub region: String,
    pub callback_key: String,
    pub limit: Option<u16>,
}

pub async fn smser_ali_config_add<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: SmserAliConfigAddParam,
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
    let alisender = &req_dao.web_dao.sender_smser.aliyun_sender;
    let row = alisender
        .add_config(
            &param.name,
            &param.access_id,
            &param.access_secret,
            &param.region,
            &param.callback_key,
            &param.limit.unwrap_or_default(),
            &req_auth.user_data().user_id,
            Some(&req_dao.req_env),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::data(json!({ "id": row })))
}

#[derive(Debug, Deserialize)]
pub struct SmserAliConfigEditParam {
    pub id: u64,
    pub name: String,
    pub access_id: String,
    pub access_secret: String,
    pub region: String,
    pub callback_key: String,
    pub limit: Option<u16>,
}

pub async fn smser_ali_config_edit<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: SmserAliConfigEditParam,
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
    let alisender = &req_dao.web_dao.sender_smser.aliyun_sender;
    let row = alisender
        .edit_config(
            &param.id,
            &param.name,
            &param.access_id,
            &param.access_secret,
            &param.region,
            &param.callback_key,
            &param.limit.unwrap_or_default(),
            &req_auth.user_data().user_id,
            Some(&req_dao.req_env),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::data(json!({ "num": row })))
}

#[derive(Debug, Deserialize)]
pub struct SmserAliConfigDelParam {
    pub id: u64,
}

pub async fn smser_ali_config_del<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: SmserAliConfigDelParam,
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
    let alisender = &req_dao.web_dao.sender_smser.aliyun_sender;
    let row = alisender
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
pub struct SmserAppAliConfigAddParam {
    pub app_id: u64,
    pub user_id: Option<u64>,
    pub ali_config_id: u64,
    pub name: String,
    pub tpl_id: String,
    pub aliyun_sms_tpl: String,
    pub aliyun_sign_name: String,
}

pub async fn smser_ali_app_config_add<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: SmserAppAliConfigAddParam,
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
    let alisender = &req_dao.web_dao.sender_smser.aliyun_sender;
    let row = alisender
        .add_app_config(
            &param.name,
            &param.app_id,
            &param.ali_config_id,
            &param.tpl_id,
            &param.aliyun_sms_tpl,
            &param.aliyun_sign_name,
            &uid,
            &req_auth.user_data().user_id,
            Some(&req_dao.req_env),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::data(json!({ "id": row })))
}
