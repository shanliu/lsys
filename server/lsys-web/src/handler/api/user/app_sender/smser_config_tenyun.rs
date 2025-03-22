use crate::common::{JsonData, JsonResult, UserAuthQueryDao};
use crate::dao::access::api::user::CheckAppSenderSmsConfig;
use lsys_access::dao::AccessSession;
use serde::Deserialize;
use serde_json::json;
use serde_json::Value;
#[derive(Debug, Deserialize)]
pub struct SmserTenConfigListParam {
    pub ids: Option<Vec<u64>>,
}

pub async fn smser_ten_config_list(
    param: &SmserTenConfigListParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &req_dao.req_env,Some(&auth_data),
            &CheckAppSenderSmsConfig {
                res_user_id: auth_data.user_id(),
            },
        )
        .await?;

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
               "app_id":e.hide_secret_id()
            })
        })
        .collect::<Vec<Value>>();

    Ok(JsonData::data(json!({ "data": row })))
}

#[derive(Debug, Deserialize)]
pub struct SmserAppTenConfigAddParam {
    pub app_id: u64,
    pub config_id: u64,
    pub name: String,
    pub tpl_id: String,
    pub sign_name: String,
    pub template_id: String,
    pub template_map: String,
}

pub async fn smser_ten_app_config_add(
    param: &SmserAppTenConfigAddParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    super::smser_inner_access_check(param.app_id, auth_data.user_id(), req_dao).await?;
    let row = req_dao
        .web_dao
        .app_sender
        .smser
        .tenyun_sender
        .add_app_config(
            &param.name,
            param.app_id,
            param.config_id,
            &param.tpl_id,
            &param.sign_name,
            &param.template_id,
            &param.template_map,
            auth_data.user_id(),
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::data(json!({ "id": row })))
}
