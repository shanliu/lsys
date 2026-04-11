use crate::common::{JsonData, JsonResponse, JsonResult, UserAuthQueryDao};
use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::admin::CheckAdminFileManage;
use lsys_access::dao::AccessSession;
use serde::Deserialize;
use serde_json::json;

/// 提交系统导出任务参数（app_id 固定为 0）
#[derive(Debug, Deserialize)]
pub struct AdminExportSubmitParam {
    /// 导出类型标识（需已注册到 WebExportTask）
    pub export_type: String,
    /// 导出过滤参数（各 Exporter 自行解析）
    #[serde(default)]
    pub params: Option<serde_json::Value>,
}

/// 提交系统导出任务（app_id=0）
pub async fn admin_export_submit(
    param: &AdminExportSubmitParam,
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

    let add_user_id = auth_data.user_id();
    let params = param
        .params
        .clone()
        .unwrap_or_else(|| serde_json::Value::Object(Default::default()));

    let task_id = req_dao
        .web_dao
        .web_files
        .export_task
        .submit(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            0,           // app_id=0 → 系统级任务
            0,           // app_user_id=0 → 系统
            add_user_id, // user_id → 实际操作的管理员
            &param.export_type,
            params,
            Some(&req_dao.req_env),
        )
        .await?;

    Ok(JsonResponse::data(JsonData::body(json!({ "id": task_id }))))
}
