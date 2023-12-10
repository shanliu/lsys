use crate::{
    dao::RequestDao,
    handler::access::{AccessAdminSmsConfig, AccessAppSenderSmsConfig},
    {JsonData, JsonResult},
};
use lsys_user::dao::auth::{SessionData, SessionTokenData, UserSession};
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;
#[derive(Debug, Deserialize)]
pub struct SmserJDConfigListParam {
    pub ids: Option<Vec<u64>>,
    pub full_data: Option<bool>,
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

pub async fn smser_jd_config_list<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: SmserJDConfigListParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let tensender = &req_dao.web_dao.sender_smser.jd_sender;
    let row = tensender.list_config(&param.ids).await?;
    let out = if param.full_data.unwrap_or(false) {
        let req_auth = req_dao.user_session.read().await.get_session_data().await?;
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
pub struct SmserJDConfigAddParam {
    pub name: String,
    pub region: String,
    pub access_key: String,
    pub access_secret: String,
    pub limit: Option<u16>,
}

pub async fn smser_jd_config_add<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: SmserJDConfigAddParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
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
        .await?;
    let tensender = &req_dao.web_dao.sender_smser.jd_sender;
    let row = tensender
        .add_config(
            &param.name,
            &param.region,
            &param.access_key,
            &param.access_secret,
            &param.limit.unwrap_or_default(),
            &req_auth.user_data().user_id,
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

pub async fn smser_jd_config_edit<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: SmserJDConfigEditParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
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
        .await?;
    let tensender = &req_dao.web_dao.sender_smser.jd_sender;
    let row = tensender
        .edit_config(
            &param.id,
            &param.name,
            &param.region,
            &param.access_key,
            &param.access_secret,
            &param.limit.unwrap_or_default(),
            &req_auth.user_data().user_id,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::data(json!({ "num": row })))
}

#[derive(Debug, Deserialize)]
pub struct SmserJDConfigDelParam {
    pub id: u64,
}

pub async fn smser_jd_config_del<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: SmserJDConfigDelParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
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
        .await?;
    let tensender = &req_dao.web_dao.sender_smser.jd_sender;
    let row = tensender
        .del_config(
            &param.id,
            &req_auth.user_data().user_id,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::data(json!({ "num": row })))
}

#[derive(Debug, Deserialize)]
pub struct SmserAppJDConfigAddParam {
    pub app_id: u64,
    pub user_id: Option<u64>,
    pub config_id: u64,
    pub name: String,
    pub tpl_id: String,
    pub sign_id: String,
    pub template_id: String,
    pub template_map: String,
}

pub async fn smser_jd_app_config_add<
    't,
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    param: SmserAppJDConfigAddParam,
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

    let tensender = &req_dao.web_dao.sender_smser.jd_sender;

    let row = tensender
        .add_app_config(
            &param.name,
            &param.app_id,
            &param.config_id,
            &param.tpl_id,
            &param.sign_id,
            &param.template_id,
            &param.template_map,
            &uid,
            &req_auth.user_data().user_id,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::data(json!({ "id": row })))
}
