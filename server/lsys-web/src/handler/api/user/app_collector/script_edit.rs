use crate::common::{JsonData, JsonResponse, JsonResult, UserAuthQueryDao};
use crate::handler::api::user::app_collector::app_check_get;
use lsys_access::dao::AccessSession;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct ScriptEditParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub script_id: u64,
    pub name: String,
    pub script_code: String,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u32")]
    pub timeout_secs: Option<u32>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u64")]
    pub memory_limit: Option<u64>,
}

/// POST /api/user/collector/script_edit — 更新脚本
pub async fn script_edit(
    param: &ScriptEditParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let _app = app_check_get(param.app_id, true, &auth_data, req_dao).await?;

    let affected = req_dao
        .web_dao
        .web_files
        .collector
        .script_edit(
            param.script_id,
            &param.name,
            &param.script_code,
            param.timeout_secs,
            param.memory_limit,
            Some(&req_dao.req_env),
        )
        .await?;

    Ok(JsonResponse::data(JsonData::body(json!({
        "affected": affected,
    }))))
}
