use crate::common::{JsonData, JsonResult, UserAuthQueryDao};
use crate::dao::access::api::system::CheckAdminSmsConfig;
use crate::dao::access::api::system::CheckAdminSmsMgr;
use lsys_access::dao::AccessSession;
use lsys_app_sender::dao::TenYunConfig;
use lsys_setting::dao::SettingData;
use serde::{Deserialize, Serialize};
use serde_json::json;
#[derive(Debug, Deserialize)]
pub struct SmserTenConfigListParam {
    pub ids: Option<Vec<u64>>,
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

pub async fn smser_ten_config_list(
    param: &SmserTenConfigListParam,
    callback_call: impl Fn(&SettingData<TenYunConfig>) -> String,
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
        .tenyun_sender
        .list_config(param.ids.as_deref())
        .await?;
    let out = {
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

pub async fn smser_ten_config_add(
    param: &SmserTenConfigAddParam,
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
        .tenyun_sender
        .add_config(
            &param.name,
            &param.region,
            &param.secret_id,
            &param.secret_key,
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

pub async fn smser_ten_config_edit(
    param: &SmserTenConfigEditParam,
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
        .tenyun_sender
        .edit_config(
            param.id,
            &param.name,
            &param.region,
            &param.secret_id,
            &param.secret_key,
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
pub struct SmserTenConfigDelParam {
    pub id: u64,
}

pub async fn smser_ten_config_del(
    param: &SmserTenConfigDelParam,
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
        .tenyun_sender
        .del_config(param.id, req_auth.user_id(), Some(&req_dao.req_env))
        .await?;
    Ok(JsonData::data(json!({ "num": row })))
}

#[derive(Debug, Deserialize)]
pub struct SmserAppTenConfigAddParam {
    pub config_id: u64,
    pub name: String,
    pub tpl_id: String,
    pub sign_name: String,
    pub template_id: String,
    pub template_map: String,
}

pub async fn smser_ten_app_config_add(
    param: &SmserAppTenConfigAddParam,
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
        .tenyun_sender
        .add_app_config(
            &param.name,
            0,
            param.config_id,
            &param.tpl_id,
            &param.sign_name,
            &param.template_id,
            &param.template_map,
            req_auth.user_id(),
            req_auth.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::data(json!({ "id": row })))
}
