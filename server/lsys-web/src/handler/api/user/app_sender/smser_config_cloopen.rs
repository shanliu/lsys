use crate::common::{JsonData, JsonResult, UserAuthQueryDao};
use crate::dao::access::api::user::CheckAppSenderSmsConfig;
use lsys_access::dao::AccessSession;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;
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
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let row = req_dao
        .web_dao
        .app_sender
        .smser
        .tenyun_sender
        .list_config(param.ids.as_deref())
        .await?;

    let row = row
        .into_iter()
        .map(|e| {
            json!({
               "id": e.model().id,
               "name": e.model().name,
               "app_id":e.hide_secret_key()
            })
        })
        .collect::<Vec<Value>>();

    Ok(JsonData::data(json!({ "data": row })))
}

#[derive(Debug, Deserialize)]
pub struct SmserAppCloopenConfigAddParam {
    pub app_id: u64,
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
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &req_dao.access_env().await?,
            &CheckAppSenderSmsConfig {
                res_user_id: req_auth.user_id(),
            },
            None,
        )
        .await?;

    let row = req_dao
        .web_dao
        .app_sender
        .smser
        .cloopen_sender
        .add_app_config(
            &param.name,
            param.app_id,
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
