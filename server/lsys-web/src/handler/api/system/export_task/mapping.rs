use crate::common::{JsonData, JsonResponse, JsonResult, RequestDao};
use crate::dao::ExportTaskStatus;
use serde_json::json;


pub async fn admin_export_task_mapping(req_dao: &RequestDao) -> JsonResult<JsonResponse> {
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
