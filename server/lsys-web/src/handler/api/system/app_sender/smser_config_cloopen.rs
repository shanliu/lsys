use crate::common::{JsonData, JsonResult, UserAuthQueryDao};
use crate::dao::access::api::system::{CheckAdminSmsConfig, CheckAdminSmsMgr};
use lsys_access::dao::AccessSession;
use lsys_app_sender::dao::CloOpenConfig;
use lsys_setting::dao::SettingData;
use serde::{Deserialize, Serialize};
use serde_json::json;
#[derive(Debug, Deserialize)]
pub struct SmserCloOpenConfigListParam {
    pub ids: Option<Vec<u64>>,
}

#[derive(Serialize)]
pub struct ShowCloOpenConfig {
    pub id: u64,
    pub name: String,
    pub account_sid: String,
    pub hide_account_sid: String,
    pub account_token: String,
    pub change_user_id: u64,
    pub change_time: u64,
    pub callback_url: String,
    pub callback_key: String,
    pub sms_app_id: String,
    pub limit: u16,
}

pub async fn smser_cloopen_config_list(
    param: &SmserCloOpenConfigListParam,
    callback_call: impl Fn(&SettingData<CloOpenConfig>) -> String,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.access_env().await?, &CheckAdminSmsConfig {}, None)
        .await?;
    let row = req_dao
        .web_dao
        .app_sender
        .smser
        .cloopen_sender
        .list_config(param.ids.as_deref())
        .await?;
    let out = {
        let tmp = row
            .into_iter()
            .map(|e| ShowCloOpenConfig {
                id: e.model().id,
                name: e.model().name.to_owned(),
                account_sid: e.account_sid.to_owned(),
                hide_account_sid: e.hide_access_key(),
                account_token: e.account_token.to_owned(),
                sms_app_id: e.sms_app_id.to_owned(),
                change_user_id: e.model().change_user_id,
                change_time: e.model().change_time,
                callback_url: callback_call(&e),
                limit: e.branch_limit,
                callback_key: e.callback_key.to_owned(),
            })
            .collect::<Vec<_>>();
        json!({ "data": tmp })
    };
    Ok(JsonData::data(out))
}

#[derive(Debug, Deserialize)]
pub struct SmserCloOpenConfigAddParam {
    pub name: String,
    pub account_sid: String,
    pub account_token: String,
    pub limit: Option<u16>,
    pub sms_app_id: String,
    pub callback_key: String,
}

pub async fn smser_cloopen_config_add(
    param: &SmserCloOpenConfigAddParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.access_env().await?, &CheckAdminSmsConfig {}, None)
        .await?;
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    let row = req_dao
        .web_dao
        .app_sender
        .smser
        .cloopen_sender
        .add_config(
            &param.name,
            &param.account_sid,
            &param.account_token,
            &param.sms_app_id,
            param.limit.unwrap_or_default(),
            &param.callback_key,
            req_auth.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::data(json!({ "id": row })))
}

#[derive(Debug, Deserialize)]
pub struct SmserCloOpenConfigEditParam {
    pub id: u64,
    pub name: String,
    pub account_sid: String,
    pub account_token: String,
    pub sms_app_id: String,
    pub limit: Option<u16>,
    pub callback_key: String,
}

pub async fn smser_cloopen_config_edit(
    param: &SmserCloOpenConfigEditParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.access_env().await?, &CheckAdminSmsConfig {}, None)
        .await?;
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;

    let row = req_dao
        .web_dao
        .app_sender
        .smser
        .cloopen_sender
        .edit_config(
            param.id,
            &param.name,
            &param.account_sid,
            &param.account_token,
            &param.sms_app_id,
            param.limit.unwrap_or_default(),
            &param.callback_key,
            req_auth.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::data(json!({ "num": row })))
}

#[derive(Debug, Deserialize)]
pub struct SmserCloOpenConfigDelParam {
    pub id: u64,
}

pub async fn smser_cloopen_config_del(
    param: &SmserCloOpenConfigDelParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.access_env().await?, &CheckAdminSmsConfig {}, None)
        .await?;
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    let row = req_dao
        .web_dao
        .app_sender
        .smser
        .cloopen_sender
        .del_config(param.id, req_auth.user_id(), Some(&req_dao.req_env))
        .await?;
    Ok(JsonData::data(json!({ "num": row })))
}

#[derive(Debug, Deserialize)]
pub struct SmserAppCloopenConfigAddParam {
    pub config_id: u64,
    pub name: String,
    pub tpl_id: String,
    pub template_id: String,
    pub template_map: String,
}

pub async fn smser_cloopen_app_config_add(
    param: &SmserAppCloopenConfigAddParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.access_env().await?, &CheckAdminSmsMgr {}, None)
        .await?;
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;

    let row = req_dao
        .web_dao
        .app_sender
        .smser
        .cloopen_sender
        .add_app_config(
            &param.name,
            0,
            param.config_id,
            &param.tpl_id,
            &param.template_id,
            &param.template_map,
            req_auth.user_id(),
            req_auth.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::data(json!({ "id": row })))
}
