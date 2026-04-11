use crate::common::{JsonData, JsonResponse, JsonResult, UserAuthQueryDao};
use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::admin::{CheckAdminSmsConfig, CheckAdminSmsMgr};
use lsys_access::dao::AccessSession;
use lsys_app_sender::dao::EmayConfig;
use lsys_setting::dao::SettingData;
use serde::{Deserialize, Serialize};
use serde_json::json;
#[derive(Debug, Deserialize)]
pub struct SmserEmayConfigListParam {
    #[serde(
        default,
        deserialize_with = "crate::common::deserialize_option_vec_u64"
    )]
    pub ids: Option<Vec<u64>>,
}

#[derive(Serialize)]
pub struct ShowEmayConfigRecord {
    pub id: u64,
    pub name: String,
    pub host: String,
    pub app_id: String,
    pub hide_app_id: String,
    pub secret_key: String,
    pub change_user_id: u64,
    pub change_time: u64,
    pub callback_url: String,
    pub callback_key: String,
    pub limit: u16,
}

pub async fn smser_emay_config_list(
    param: &SmserEmayConfigListParam,
    callback_call: impl Fn(&SettingData<EmayConfig>) -> String,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminSmsConfig {},
        )
        .await?;
    let row = req_dao
        .web_dao
        .app_sender
        .smser
        .emay_sender
        .list_config(param.ids.as_deref())
        .await?;
    let out = {
        let tmp = row
            .into_iter()
            .map(|e| ShowEmayConfigRecord {
                id: e.model().id,
                name: e.model().name.to_owned(),
                host: e.host.to_owned(),
                app_id: e.app_id.to_owned(),
                hide_app_id: e.hide_app_id(),
                secret_key: e.secret_key.to_owned(),
                change_user_id: e.model().change_user_id,
                change_time: e.model().change_time,
                callback_url: callback_call(&e),
                limit: e.branch_limit,
                callback_key: e.callback_key.to_owned(),
            })
            .collect::<Vec<_>>();
        json!({ "data": tmp })
    };
    Ok(JsonResponse::data(JsonData::body(out)))
}

#[derive(Debug, Deserialize)]
pub struct SmserEmayConfigAddParam {
    pub name: String,
    pub host: String,
    pub app_id: String,
    pub secret_key: String,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u16")]
    pub limit: Option<u16>,
    pub callback_key: String,
}

pub async fn smser_emay_config_add(
    param: &SmserEmayConfigAddParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminSmsConfig {},
        )
        .await?;
    let row = req_dao
        .web_dao
        .app_sender
        .smser
        .emay_sender
        .add_config(
            &param.name,
            &param.host,
            &param.app_id,
            &param.secret_key,
            param.limit.unwrap_or_default(),
            &param.callback_key,
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({ "id": row }))))
}

#[derive(Debug, Deserialize)]
pub struct SmserEmayConfigEditParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub id: u64,
    pub name: String,
    pub host: String,
    pub app_id: String,
    pub secret_key: String,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u16")]
    pub limit: Option<u16>,
    pub callback_key: String,
}

pub async fn smser_emay_config_edit(
    param: &SmserEmayConfigEditParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminSmsConfig {},
        )
        .await?;

    let row = req_dao
        .web_dao
        .app_sender
        .smser
        .emay_sender
        .edit_config(
            param.id,
            &param.name,
            &param.host,
            &param.app_id,
            &param.secret_key,
            param.limit.unwrap_or_default(),
            &param.callback_key,
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({ "num": row }))))
}

#[derive(Debug, Deserialize)]
pub struct SmserEmayConfigDelParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub id: u64,
}

pub async fn smser_emay_config_del(
    param: &SmserEmayConfigDelParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminSmsConfig {},
        )
        .await?;
    let row = req_dao
        .web_dao
        .app_sender
        .smser
        .emay_sender
        .del_config(param.id, auth_data.user_id(), Some(&req_dao.req_env))
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({ "num": row }))))
}

#[derive(Debug, Deserialize)]
pub struct SmserAppEmayConfigAddParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub config_id: u64,
    pub name: String,
    pub tpl_key: String,
    pub extended_code: String,
}

pub async fn smser_tpl_config_emay_add(
    param: &SmserAppEmayConfigAddParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminSmsMgr {},
        )
        .await?;

    let row = req_dao
        .web_dao
        .app_sender
        .smser
        .emay_sender
        .add_app_config(
            &param.name,
            0,
            param.config_id,
            &param.tpl_key,
            &param.extended_code,
            0,
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({ "id": row }))))
}
