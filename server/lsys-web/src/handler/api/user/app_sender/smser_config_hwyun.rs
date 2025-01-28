use crate::common::{JsonData, JsonResult, UserAuthQueryDao};
use crate::dao::access::api::user::CheckAppSenderSmsConfig;
use lsys_access::dao::AccessSession;

use serde::Deserialize;
use serde_json::json;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct SmserHwConfigListParam {
    pub ids: Option<Vec<u64>>,
}

pub async fn smser_hw_config_list(
    param: &SmserHwConfigListParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let data = req_dao
        .web_dao
        .app_sender
        .smser
        .hwyun_sender
        .list_config(param.ids.as_deref())
        .await?;

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

    Ok(JsonData::data(json!({ "data": row })))
}

#[derive(Debug, Deserialize)]
pub struct SmserAppHwConfigAddParam {
    pub app_id: u64,
    pub hw_config_id: u64,
    pub name: String,
    pub tpl_id: String,
    pub signature: String,
    pub sender: String,
    pub template_id: String,
    pub template_map: String,
}

pub async fn smser_hw_app_config_add(
    param: &SmserAppHwConfigAddParam,
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
        .hwyun_sender
        .add_app_config(
            &param.name,
            param.app_id,
            param.hw_config_id,
            &param.tpl_id,
            &param.signature,
            &param.sender,
            &param.template_id,
            &param.template_map,
            req_auth.user_id(),
            req_auth.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::data(json!({ "id": row })))
}
