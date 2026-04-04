use crate::common::{JsonData, JsonResponse, JsonResult, UserAuthQueryDao};
use crate::handler::api::user::app_collector::app_check_get;
use lsys_access::dao::AccessSession;
use serde::Deserialize;
use serde_json::json;
#[derive(Debug, Deserialize)]
pub struct ScriptDeleteParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub script_id: u64,
}

/// POST /api/user/collector/script_del — 删除脚本
pub async fn script_del(
    param: &ScriptDeleteParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let _app = app_check_get(param.app_id, true, &auth_data, req_dao).await?;

    let affected = req_dao
        .web_dao
        .web_files
        .collector
        .script_delete(param.script_id, Some(&req_dao.req_env))
        .await?;

    Ok(JsonResponse::data(JsonData::body(json!({
        "affected": affected,
    }))))
}
