use crate::common::{JsonData, JsonResponse, JsonResult, UserAuthQueryDao};
use crate::dao::access::api::system::admin::CheckAdminFileManage;
use crate::dao::access::RbacAccessCheckEnv;
use crate::model::CollectorScriptStatus;
use lsys_access::dao::AccessSession;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct ScriptStatusParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub script_id: u64,
    #[serde(deserialize_with = "crate::common::deserialize_i8")]
    pub status: i8,
}

/// POST /api/system/collector/script_status — 启用/禁用脚本
pub async fn script_status(
    param: &ScriptStatusParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminFileManage {},
        )
        .await?;

    let status = match param.status {
        1 => CollectorScriptStatus::Enable,
        2 => CollectorScriptStatus::Disable,
        _ => {
            return Ok(JsonResponse::data(JsonData::error())
                .set_message("invalid status, must be 1 (enable) or 2 (disable)"));
        }
    };

    let affected = req_dao
        .web_dao
        .web_files
        .collector
        .script_change_status(param.script_id, status, Some(&req_dao.req_env))
        .await?;

    Ok(JsonResponse::data(JsonData::body(json!({
        "affected": affected,
    }))))
}
