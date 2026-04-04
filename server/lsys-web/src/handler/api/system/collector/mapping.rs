use crate::common::{JsonData, JsonResponse, JsonResult, UserAuthQueryDao};
use crate::model::{CollectorRecordStatus, CollectorScriptStatus};
use serde_json::json;

/// POST /api/system/collector/mapping — 采集字典映射
pub async fn mapping_data(req_dao: &UserAuthQueryDao) -> JsonResult<JsonResponse> {
    Ok(JsonResponse::data(JsonData::body(json!({
        "script_status": vec![
            status_json_format!(req_dao, CollectorScriptStatus::Enable),
            status_json_format!(req_dao, CollectorScriptStatus::Disable),
        ],
        "record_status": vec![
            status_json_format!(req_dao, CollectorRecordStatus::Pending),
            status_json_format!(req_dao, CollectorRecordStatus::Running),
            status_json_format!(req_dao, CollectorRecordStatus::Success),
            status_json_format!(req_dao, CollectorRecordStatus::Failed),
            status_json_format!(req_dao, CollectorRecordStatus::Timeout),
        ],
    }))))
}
