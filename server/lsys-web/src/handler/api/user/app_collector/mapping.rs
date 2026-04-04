use crate::common::{JsonData, JsonResponse, JsonResult, UserAuthQueryDao};
use crate::model::{
    CollectorRecordStatus, CollectorScriptStatus,
    COLLECTOR_LOG_LEVEL_DEBUG, COLLECTOR_LOG_LEVEL_ERROR,
    COLLECTOR_LOG_LEVEL_INFO, COLLECTOR_LOG_LEVEL_SYSTEM, COLLECTOR_LOG_LEVEL_WARN,
};

use serde_json::json;

/// POST /api/user/file/collector/mapping — 采集字典映射
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
        "log_level": vec![
            var_json_format!(req_dao, &COLLECTOR_LOG_LEVEL_DEBUG.to_string()),
            var_json_format!(req_dao, &COLLECTOR_LOG_LEVEL_INFO.to_string()),
            var_json_format!(req_dao, &COLLECTOR_LOG_LEVEL_WARN.to_string()),
            var_json_format!(req_dao, &COLLECTOR_LOG_LEVEL_ERROR.to_string()),
            var_json_format!(req_dao, &COLLECTOR_LOG_LEVEL_SYSTEM.to_string()),
        ],
    }))))
}
