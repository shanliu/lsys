use crate::common::{JsonData, JsonResponse, JsonResult, UserAuthQueryDao};
use crate::dao::access::api::system::admin::CheckAdminFileManage;
use crate::dao::access::RbacAccessCheckEnv;
use lsys_access::dao::AccessSession;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct SubmitTaskParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub script_id: u64,
    #[serde(default)]
    pub request_id: Option<String>,
    #[serde(default)]
    pub params: Option<serde_json::Value>,
}

/// POST /api/system/collector/submit_task — 提交采集任务（管理员测试脚本）
pub async fn submit_task(
    param: &SubmitTaskParam,
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

    let user_id = auth_data.user_id();

    let request_id = match &param.request_id {
        Some(rid) if !rid.trim().is_empty() => rid.trim().to_string(),
        _ => crate::dao::collector::WebFileCollector::resolve_request_id(&req_dao.req_env),
    };

    let params = param.params.clone().unwrap_or(serde_json::json!({}));

    let (record_id, task_id, script_name) = req_dao
        .web_dao
        .web_files
        .collector
        .submit_task(param.script_id, 0, user_id, 0, &request_id, &params, Some(&req_dao.req_env))
        .await?;

    Ok(JsonResponse::data(JsonData::body(json!({
        "request_id": request_id,
        "record_id": record_id.to_string(),
        "task_id": task_id.to_string(),
        "script_name": script_name,
    }))))
}
