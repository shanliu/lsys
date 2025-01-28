use crate::common::{JsonData, JsonResult, UserAuthQueryDao};
use crate::dao::access::api::system::{CheckAdminSmsConfig, CheckAdminSmsMgr};
use lsys_access::dao::AccessSession;
use serde::{Deserialize, Serialize};
use serde_json::json;
#[derive(Debug, Deserialize)]
pub struct SmserJDConfigListParam {
    pub ids: Option<Vec<u64>>,
}

#[derive(Serialize)]
pub struct ShowJDYunConfig {
    pub id: u64,
    pub name: String,
    pub region: String,
    pub access_key: String,
    pub hide_access_key: String,
    pub access_secret: String,
    pub change_user_id: u64,
    pub change_time: u64,
    pub limit: u16,
}

pub async fn smser_jd_config_list(
    param: &SmserJDConfigListParam,
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
        .jd_sender
        .list_config(param.ids.as_deref())
        .await?;

    let tmp = row
        .into_iter()
        .map(|e| ShowJDYunConfig {
            id: e.model().id,
            region: e.region.to_owned(),
            name: e.model().name.to_owned(),
            access_key: e.access_key.to_owned(),
            hide_access_key: e.hide_access_key(),
            access_secret: e.access_secret.to_owned(),
            change_user_id: e.model().change_user_id,
            change_time: e.model().change_time,
            limit: e.branch_limit,
        })
        .collect::<Vec<_>>();

    Ok(JsonData::data(json!({ "data": tmp })))
}

#[derive(Debug, Deserialize)]
pub struct SmserJDConfigAddParam {
    pub name: String,
    pub region: String,
    pub access_key: String,
    pub access_secret: String,
    pub limit: Option<u16>,
}

pub async fn smser_jd_config_add(
    param: &SmserJDConfigAddParam,
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
        .jd_sender
        .add_config(
            &param.name,
            &param.region,
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
pub struct SmserJDConfigEditParam {
    pub id: u64,
    pub name: String,
    pub region: String,
    pub access_key: String,
    pub access_secret: String,
    pub limit: Option<u16>,
}

pub async fn smser_jd_config_edit(
    param: &SmserJDConfigEditParam,
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
        .jd_sender
        .edit_config(
            param.id,
            &param.name,
            &param.region,
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
pub struct SmserJDConfigDelParam {
    pub id: u64,
}

pub async fn smser_jd_config_del(
    param: &SmserJDConfigDelParam,
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
        .jd_sender
        .del_config(param.id, req_auth.user_id(), Some(&req_dao.req_env))
        .await?;
    Ok(JsonData::data(json!({ "num": row })))
}

#[derive(Debug, Deserialize)]
pub struct SmserAppJDConfigAddParam {
    pub config_id: u64,
    pub name: String,
    pub tpl_id: String,
    pub sign_id: String,
    pub template_id: String,
    pub template_map: String,
}

pub async fn smser_jd_app_config_add(
    param: &SmserAppJDConfigAddParam,
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
        .jd_sender
        .add_app_config(
            &param.name,
            0,
            param.config_id,
            &param.tpl_id,
            &param.sign_id,
            &param.template_id,
            &param.template_map,
            req_auth.user_id(),
            req_auth.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::data(json!({ "id": row })))
}
