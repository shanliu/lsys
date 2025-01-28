use crate::common::{JsonData, JsonResult, UserAuthQueryDao};
use crate::dao::access::api::system::CheckAdminSmsConfig;
use crate::dao::access::api::system::CheckAdminSmsMgr;
use lsys_access::dao::AccessSession;
use lsys_app_sender::dao::NetEaseConfig;
use lsys_setting::dao::SettingData;
use serde::{Deserialize, Serialize};
use serde_json::json;
#[derive(Debug, Deserialize)]
pub struct SmserNetEaseConfigListParam {
    pub ids: Option<Vec<u64>>,
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

pub async fn smser_netease_config_list(
    param: &SmserNetEaseConfigListParam,
    callback_call: impl Fn(&SettingData<NetEaseConfig>) -> String,
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
        .netease_sender
        .list_config(param.ids.as_deref())
        .await?;
    let out = {
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

pub async fn smser_netease_config_add(
    param: &SmserNetEaseConfigAddParam,
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
        .netease_sender
        .add_config(
            &param.name,
            &param.access_key,
            &param.access_secret,
            param.limit.unwrap_or_default(),
            req_auth.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
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

pub async fn smser_netease_config_edit(
    param: &SmserNetEaseConfigEditParam,
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
        .netease_sender
        .edit_config(
            param.id,
            &param.name,
            &param.access_key,
            &param.access_secret,
            param.limit.unwrap_or_default(),
            req_auth.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::data(json!({ "num": row })))
}

#[derive(Debug, Deserialize)]
pub struct SmserNetEaseConfigDelParam {
    pub id: u64,
}

pub async fn smser_netease_config_del(
    param: &SmserNetEaseConfigDelParam,
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
        .netease_sender
        .del_config(param.id, req_auth.user_id(), Some(&req_dao.req_env))
        .await?;
    Ok(JsonData::data(json!({ "num": row })))
}

#[derive(Debug, Deserialize)]
pub struct SmserAppNetEaseConfigAddParam {
    pub config_id: u64,
    pub name: String,
    pub tpl_id: String,
    pub template_id: String,
    pub template_map: String,
}

pub async fn smser_netease_app_config_add(
    param: &SmserAppNetEaseConfigAddParam,
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
        .netease_sender
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
