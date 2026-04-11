use crate::common::{JsonData, JsonResponse, JsonResult, UserAuthQueryDao};
use crate::dao::ExportTaskStatus;
use serde_json::json;

/// POST /api/user/app_export_task/mapping — 导出任务字典映射（用户端）
pub async fn app_export_task_mapping(req_dao: &UserAuthQueryDao) -> JsonResult<JsonResponse> {
    Ok(JsonResponse::data(JsonData::body(json!({
        "export_task_status": vec![
            status_json_format!(req_dao, ExportTaskStatus::Pending),
            status_json_format!(req_dao, ExportTaskStatus::Running),
            status_json_format!(req_dao, ExportTaskStatus::Success),
            status_json_format!(req_dao, ExportTaskStatus::Failed),
            status_json_format!(req_dao, ExportTaskStatus::Deleted),
        ],
    }))))
}
