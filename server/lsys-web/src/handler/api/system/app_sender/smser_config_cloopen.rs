use crate::common::{JsonData, JsonResponse, JsonResult, UserAuthQueryDao};
use crate::dao::access::api::system::admin::{CheckAdminSmsConfig, CheckAdminSmsMgr};
use crate::dao::access::RbacAccessCheckEnv;
use lsys_access::dao::AccessSession;
use lsys_app_sender::dao::CloOpenConfig;
use lsys_setting::dao::SettingData;
use serde::{Deserialize, Serialize};
use serde_json::json;
#[derive(Debug, Deserialize)]
pub struct SmserCloOpenConfigListParam {
    #[serde(
        default,
        deserialize_with = "crate::common::deserialize_option_vec_u64"
    )]
    pub ids: Option<Vec<u64>>,
}

#[derive(Serialize)]
pub struct ShowCloOpenConfigRecord {
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
        .cloopen_sender
        .list_config(param.ids.as_deref())
        .await?;
    let out = {
        let tmp = row
            .into_iter()
            .map(|e| ShowCloOpenConfigRecord {
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
    Ok(JsonResponse::data(JsonData::body(out)))
}

#[derive(Debug, Deserialize)]
pub struct SmserCloOpenConfigAddParam {
    pub name: String,
    pub account_sid: String,
    pub account_token: String,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u16")]
    pub limit: Option<u16>,
    pub sms_app_id: String,
    pub callback_key: String,
}

pub async fn smser_cloopen_config_add(
    param: &SmserCloOpenConfigAddParam,
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
        .cloopen_sender
        .add_config(
            &param.name,
            &param.account_sid,
            &param.account_token,
            &param.sms_app_id,
            param.limit.unwrap_or_default(),
            &param.callback_key,
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({ "id": row }))))
}

#[derive(Debug, Deserialize)]
pub struct SmserCloOpenConfigEditParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub id: u64,
    pub name: String,
    pub account_sid: String,
    pub account_token: String,
    pub sms_app_id: String,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u16")]
    pub limit: Option<u16>,
    pub callback_key: String,
}

pub async fn smser_cloopen_config_edit(
    param: &SmserCloOpenConfigEditParam,
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
        .cloopen_sender
        .edit_config(
            param.id,
            &param.name,
            &param.account_sid,
            &param.account_token,
            &param.sms_app_id,
            param.limit.unwrap_or_default(),
            &param.callback_key,
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({ "num": row }))))
}

#[derive(Debug, Deserialize)]
pub struct SmserCloOpenConfigDelParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub id: u64,
}

pub async fn smser_cloopen_config_del(
    param: &SmserCloOpenConfigDelParam,
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
        .cloopen_sender
        .del_config(param.id, auth_data.user_id(), Some(&req_dao.req_env))
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({ "num": row }))))
}

#[derive(Debug, Deserialize)]
pub struct SmserAppCloopenConfigAddParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub config_id: u64,
    pub name: String,
    pub tpl_key: String,
    pub template_id: String,
    pub template_map: String,
}

pub async fn smser_tpl_config_cloopen_add(
    param: &SmserAppCloopenConfigAddParam,
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
        .cloopen_sender
        .add_app_config(
            &param.name,
            0,
            param.config_id,
            &param.tpl_key,
            &param.template_id,
            &param.template_map,
            0,
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({ "id": row }))))
}
