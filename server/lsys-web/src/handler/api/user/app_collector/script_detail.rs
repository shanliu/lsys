use crate::common::{JsonData, JsonResponse, JsonResult, RequestDao, UserAuthQueryDao};
use crate::dao::WebDao;
use crate::handler::api::user::app_collector::app_check_get;
use lsys_access::dao::AccessSession;
use serde::Deserialize;
use serde_json::json;
#[derive(Debug, Deserialize)]
pub struct ScriptDetailParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub script_id: u64,
}

pub async fn script_detail(
    param: &ScriptDetailParam,
    req_dao: &RequestDao,
    auth_dao: &UserAuthQueryDao,
    web_dao: &WebDao,
) -> JsonResult<JsonResponse> {
    let auth_data = auth_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await?;
    app_check_get(param.app_id, false, &auth_data, req_dao, web_dao).await?;

    let script = web_dao
        .web_collector.collector
        .find_script_by_id(param.script_id)
        .await?;

    match script {
        Some(s) if s.app_id == param.app_id => Ok(JsonResponse::data(JsonData::body(json!({
            "id": s.id,
            "add_user_id": s.add_user_id,
            "app_id": s.app_id,
            "name": s.name,
            "script_code": s.script_code,
            "script_md5": s.script_md5,
            "timeout_secs": s.timeout_secs,
            "memory_limit": s.memory_limit,
            "status": s.status,
            "add_time": s.add_time,
            "change_time": s.change_time,
        })))),
        _ => Err(crate::common::JsonError::Message(
            lsys_core::fluent_message!("collector-script-not-found"),
        )),
    }
}
