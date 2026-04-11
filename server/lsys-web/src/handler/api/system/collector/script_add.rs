use crate::common::{JsonData, JsonResponse, JsonResult, UserAuthQueryDao};
use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::admin::CheckAdminFileManage;
use lsys_access::dao::AccessSession;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct ScriptAddParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub user_id: u64,
    pub name: String,
    pub script_code: String,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u32")]
    pub timeout_secs: Option<u32>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u64")]
    pub memory_limit: Option<u64>,
}

/// POST /api/system/collector/script_add — 创建系统脚本 (app_id=0)
pub async fn script_add(
    param: &ScriptAddParam,
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

    let script_id = req_dao
        .web_dao
        .web_files
        .collector
        .script_add(
            param.user_id,
            0, // 系统级 app_id=0
            0, // 系统级脚本 app_user_id=0
            &param.name,
            &param.script_code,
            param.timeout_secs,
            param.memory_limit,
            Some(&req_dao.req_env),
        )
        .await?;

    Ok(JsonResponse::data(JsonData::body(json!({
        "id": script_id.to_string(),
    }))))
}
