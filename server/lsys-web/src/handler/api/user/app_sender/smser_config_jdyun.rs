use crate::common::{JsonResponse, JsonResult, UserAuthQueryDao};
use crate::dao::access::api::user::CheckUserAppSenderSmsConfig;
use lsys_access::dao::AccessSession;
use serde::Deserialize;
use serde_json::json;
use serde_json::Value;
#[derive(Debug, Deserialize)]
pub struct SmserJDConfigListParam {
    pub ids: Option<Vec<u64>>,
}
use crate::common::JsonData;
pub async fn smser_jd_config_list(
    param: &SmserJDConfigListParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &req_dao.req_env,Some(&auth_data),
            &CheckUserAppSenderSmsConfig {
                res_user_id: auth_data.user_id(),
            },
        )
        .await?;

    let row = req_dao
        .web_dao
        .app_sender
        .smser
        .jd_sender
        .list_config(param.ids.as_deref())
        .await?;

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

    Ok(JsonResponse::data(JsonData::body(json!({ "data": row }))))
}

#[derive(Debug, Deserialize)]
pub struct SmserAppJDConfigAddParam {
    pub app_id: u64,
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
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    super::smser_inner_access_check(param.app_id, auth_data.user_id(), req_dao).await?;

    let row = req_dao
        .web_dao
        .app_sender
        .smser
        .jd_sender
        .add_app_config(
            &param.name,
            param.app_id,
            param.config_id,
            &param.tpl_id,
            &param.sign_id,
            &param.template_id,
            &param.template_map,
            auth_data.user_id(),
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({ "id": row }))))
}
