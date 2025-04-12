use crate::common::{JsonData, JsonResponse, JsonResult, UserAuthQueryDao};
use crate::dao::access::api::system::CheckAdminSmsConfig;
use crate::dao::access::api::system::CheckAdminSmsMgr;
use lsys_access::dao::AccessSession;
use lsys_app_sender::dao::HwYunConfig;
use lsys_setting::dao::SettingData;
use serde::{Deserialize, Serialize};
use serde_json::json;
#[derive(Serialize)]
pub struct ShowHwConfig {
    pub id: u64,
    pub name: String,
    pub url: String,
    pub app_key: String,
    pub hide_app_key: String,
    pub app_secret: String,
    pub change_user_id: u64,
    pub change_time: u64,
    pub callback_url: String,
    pub limit: u16,
    pub callback_key: String,
}

#[derive(Debug, Deserialize)]
pub struct SmserHwConfigListParam {
    pub ids: Option<Vec<u64>>,
}

pub async fn smser_hw_config_list(
    param: &SmserHwConfigListParam,
    callback_call: impl Fn(&SettingData<HwYunConfig>) -> String,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminSmsConfig {})
        .await?;
    let data = req_dao
        .web_dao
        .app_sender
        .smser
        .hwyun_sender
        .list_config(param.ids.as_deref())
        .await?;
    let row = {
        let tmp = data
            .into_iter()
            .map(|e| ShowHwConfig {
                id: e.model().id,
                url: e.url.clone(),
                name: e.model().name.to_owned(),
                app_key: e.app_key.to_owned(),
                hide_app_key: e.hide_access_key(),
                app_secret: e.app_secret.to_owned(),
                change_user_id: e.model().change_user_id,
                change_time: e.model().change_time,
                callback_url: callback_call(&e),
                callback_key: e.callback_key.to_string(),

                limit: e.branch_limit,
            })
            .collect::<Vec<_>>();
        json!({ "data": tmp })
    };
    Ok(JsonResponse::data(JsonData::body(row)))
}

#[derive(Debug, Deserialize)]
pub struct SmserHwConfigAddParam {
    pub name: String,
    pub url: String,
    pub app_key: String,
    pub app_secret: String,
    pub limit: Option<u16>,
    pub callback_key: String,
}

pub async fn smser_hw_config_add(
    param: &SmserHwConfigAddParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminSmsConfig {})
        .await?;

    let row = req_dao
        .web_dao
        .app_sender
        .smser
        .hwyun_sender
        .add_config(
            &param.name,
            &param.url,
            &param.app_key,
            &param.app_secret,
            param.limit.unwrap_or_default(),
            &param.callback_key,
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({ "id": row }))))
}

#[derive(Debug, Deserialize)]
pub struct SmserHwConfigEditParam {
    pub id: u64,
    pub name: String,
    pub url: String,
    pub app_key: String,
    pub app_secret: String,
    pub limit: Option<u16>,
    pub callback_key: String,
}

pub async fn smser_hw_config_edit(
    param: &SmserHwConfigEditParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminSmsConfig {})
        .await?;

    let row = req_dao
        .web_dao
        .app_sender
        .smser
        .hwyun_sender
        .edit_config(
            param.id,
            &param.name,
            &param.url,
            &param.app_key,
            &param.app_secret,
            param.limit.unwrap_or_default(),
            &param.callback_key,
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({ "num": row }))))
}

#[derive(Debug, Deserialize)]
pub struct SmserHwConfigDelParam {
    pub id: u64,
}

pub async fn smser_hw_config_del(
    param: &SmserHwConfigDelParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminSmsConfig {})
        .await?;

    let row = req_dao
        .web_dao
        .app_sender
        .smser
        .hwyun_sender
        .del_config(param.id, auth_data.user_id(), Some(&req_dao.req_env))
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({ "num": row }))))
}

#[derive(Debug, Deserialize)]
pub struct SmserAppHwConfigAddParam {
    pub hw_config_id: u64,
    pub name: String,
    pub tpl_id: String,
    pub signature: String,
    pub sender: String,
    pub template_id: String,
    pub template_map: String,
}

pub async fn smser_tpl_config_hw_add(
    param: &SmserAppHwConfigAddParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminSmsMgr {})
        .await?;

    let row = req_dao
        .web_dao
        .app_sender
        .smser
        .hwyun_sender
        .add_app_config(
            &param.name,
            0,
            param.hw_config_id,
            &param.tpl_id,
            &param.signature,
            &param.sender,
            &param.template_id,
            &param.template_map,
            auth_data.user_id(),
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({ "id": row }))))
}
