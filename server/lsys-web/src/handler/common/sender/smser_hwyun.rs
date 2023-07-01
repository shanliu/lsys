use crate::{
    dao::RequestDao,
    handler::access::{AccessAdminHwSmsConfig, AccessAppSenderSmsConfig},
    {JsonData, JsonResult},
};
use lsys_user::dao::auth::{SessionData, SessionTokenData, UserSession};
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;

#[derive(Serialize)]
pub struct ShowHwConfig {
    pub id: u64,
    pub name: String,

    pub app_key: String,
    pub hide_app_key: String,
    pub app_secret: String,
    pub change_user_id: u64,
    pub change_time: u64,
}

#[derive(Debug, Deserialize)]
pub struct SmserHwConfigListParam {
    pub ids: Option<Vec<u64>>,
    pub full_data: Option<bool>,
}

pub async fn smser_hw_config_list<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: SmserHwConfigListParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let hwsender = req_dao.web_dao.sender_smser.hwyun_sender();
    let data = hwsender.list_config(&param.ids).await?;
    let row = if param.full_data.unwrap_or(false) {
        let req_auth = req_dao.user_session.read().await.get_session_data().await?;
        req_dao
            .web_dao
            .user
            .rbac_dao
            .rbac
            .check(
                &AccessAdminHwSmsConfig {
                    user_id: req_auth.user_data().user_id,
                },
                None,
            )
            .await?;
        let tmp = data
            .into_iter()
            .map(|e| ShowHwConfig {
                id: e.model().id,
                name: e.model().name.to_owned(),
                app_key: e.app_key.to_owned(),
                hide_app_key: e.hide_access_key(),
                app_secret: e.app_secret.to_owned(),
                change_user_id: e.model().change_user_id,
                change_time: e.model().change_time,
            })
            .collect::<Vec<_>>();
        json!({ "data": tmp })
    } else {
        let row = data
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
    Ok(JsonData::data(row))
}

#[derive(Debug, Deserialize)]
pub struct SmserHwConfigAddParam {
    pub name: String,

    pub app_key: String,
    pub app_secret: String,
}

pub async fn smser_hw_config_add<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: SmserHwConfigAddParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessAdminHwSmsConfig {
                user_id: req_auth.user_data().user_id,
            },
            None,
        )
        .await?;
    let hwsender = &req_dao.web_dao.sender_smser.hwyun_sender();
    let row = hwsender
        .add_config(
            &param.name,
            &param.app_key,
            &param.app_secret,
            &req_auth.user_data().user_id,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::data(json!({ "id": row })))
}

#[derive(Debug, Deserialize)]
pub struct SmserHwConfigEditParam {
    pub id: u64,
    pub name: String,

    pub app_key: String,
    pub app_secret: String,
}

pub async fn smser_hw_config_edit<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: SmserHwConfigEditParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessAdminHwSmsConfig {
                user_id: req_auth.user_data().user_id,
            },
            None,
        )
        .await?;
    let hwsender = req_dao.web_dao.sender_smser.hwyun_sender();
    let row = hwsender
        .edit_config(
            &param.id,
            &param.name,
            &param.app_key,
            &param.app_secret,
            &req_auth.user_data().user_id,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::data(json!({ "num": row })))
}

#[derive(Debug, Deserialize)]
pub struct SmserHwConfigDelParam {
    pub id: u64,
}

pub async fn smser_hw_config_del<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: SmserHwConfigDelParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessAdminHwSmsConfig {
                user_id: req_auth.user_data().user_id,
            },
            None,
        )
        .await?;
    let hwsender = req_dao.web_dao.sender_smser.hwyun_sender();
    let row = hwsender
        .del_config(
            &param.id,
            &req_auth.user_data().user_id,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::data(json!({ "num": row })))
}

#[derive(Debug, Deserialize)]
pub struct SmserAppHwConfigAddParam {
    pub app_id: u64,
    pub user_id: Option<u64>,
    pub hw_config_id: u64,
    pub url: String,
    pub name: String,
    pub tpl_id: String,
    pub signature: String,
    pub sender: String,
    pub template_id: String,
    pub template_map: String,
}

pub async fn smser_config_add_hw<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: SmserAppHwConfigAddParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
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
        .await?;

    let hwsender = req_dao.web_dao.sender_smser.hwyun_sender();
    let row = hwsender
        .add_app_config(
            &param.name,
            &param.url,
            &param.app_id,
            &param.hw_config_id,
            &param.tpl_id,
            &param.signature,
            &param.sender,
            &param.template_id,
            &param.template_map,
            &uid,
            &req_auth.user_data().user_id,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::data(json!({ "id": row })))
}
