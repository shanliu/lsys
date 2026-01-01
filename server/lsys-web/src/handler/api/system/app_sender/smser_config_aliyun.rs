use crate::common::{JsonData, JsonResponse, JsonResult, UserAuthQueryDao};
use crate::dao::access::api::system::admin::{CheckAdminSmsConfig, CheckAdminSmsMgr};
use crate::dao::access::RbacAccessCheckEnv;
use lsys_access::dao::AccessSession;
use lsys_app_sender::dao::AliYunConfig;
use lsys_setting::dao::SettingData;
use serde::{Deserialize, Serialize};
use serde_json::json;
#[derive(Debug, Deserialize)]
pub struct SmserAliConfigListParam {
    #[serde(
        default,
        deserialize_with = "crate::common::deserialize_option_vec_u64"
    )]
    pub ids: Option<Vec<u64>>,
}

#[derive(Serialize)]
pub struct ShowAliYunConfigRecord {
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

pub async fn smser_ali_config_list(
    param: &SmserAliConfigListParam,
    callback_call: impl Fn(&SettingData<AliYunConfig>) -> String,
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
        .aliyun_sender
        .list_config(param.ids.as_deref())
        .await?;

    let tmp = row
        .into_iter()
        .map(|e| ShowAliYunConfigRecord {
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
    Ok(JsonResponse::data(JsonData::body(json!({ "data": tmp }))))
}

#[derive(Debug, Deserialize)]
pub struct SmserAliConfigAddParam {
    pub name: String,
    pub access_id: String,
    pub access_secret: String,
    pub region: String,
    pub callback_key: String,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u16")]
    pub limit: Option<u16>,
}

pub async fn smser_ali_config_add(
    param: &SmserAliConfigAddParam,
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
        .aliyun_sender
        .add_config(
            &param.name,
            &param.access_id,
            &param.access_secret,
            &param.region,
            &param.callback_key,
            param.limit.unwrap_or_default(),
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({ "id": row }))))
}

#[derive(Debug, Deserialize)]
pub struct SmserAliConfigEditParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub id: u64,
    pub name: String,
    pub access_id: String,
    pub access_secret: String,
    pub region: String,
    pub callback_key: String,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u16")]
    pub limit: Option<u16>,
}

pub async fn smser_ali_config_edit(
    param: &SmserAliConfigEditParam,
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
        .aliyun_sender
        .edit_config(
            param.id,
            &param.name,
            &param.access_id,
            &param.access_secret,
            &param.region,
            &param.callback_key,
            param.limit.unwrap_or_default(),
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({ "num": row }))))
}

#[derive(Debug, Deserialize)]
pub struct SmserAliConfigDelParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub id: u64,
}

pub async fn smser_ali_config_del(
    param: &SmserAliConfigDelParam,
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
        .aliyun_sender
        .del_config(param.id, auth_data.user_id(), Some(&req_dao.req_env))
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({ "num": row }))))
}

#[derive(Debug, Deserialize)]
pub struct SmserAppAliConfigAddParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub ali_config_id: u64,
    pub name: String,
    pub tpl_key: String,
    pub aliyun_sms_tpl: String,
    pub aliyun_sign_name: String,
}

pub async fn smser_tpl_config_ali_add(
    param: &SmserAppAliConfigAddParam,
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
        .aliyun_sender
        .add_app_config(
            &param.name,
            0,
            param.ali_config_id,
            &param.tpl_key,
            &param.aliyun_sms_tpl,
            &param.aliyun_sign_name,
            0,
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({ "id": row }))))
}
