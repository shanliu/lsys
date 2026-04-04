use crate::common::{JsonData, JsonResponse, JsonResult, UserAuthQueryDao};
use crate::handler::api::user::app_collector::app_check_get;
use lsys_access::dao::AccessSession;
use lsys_core::api_utils::JsonPageData;
use serde::Deserialize;
use serde_json::json;
#[derive(Debug, Deserialize)]
pub struct RecordLogsParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    pub request_id: String,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u8")]
    pub level: Option<u8>,
    pub page: Option<crate::common::PageParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
}

/// GET /api/user/collector/record_logs — 记录关联日志列表
pub async fn record_logs(
    param: &RecordLogsParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let _app = app_check_get(param.app_id, false, &auth_data, req_dao).await?;

    let record = req_dao
        .web_dao
        .web_files
        .collector
        .find_record_by_request_id(&param.request_id)
        .await?
        .ok_or_else(|| {
            crate::common::JsonError::Message(lsys_core::fluent_message!(
                "collector-record-not-found"
            ))
        })?;

    use crate::common::ToOffsetPageParam;
    let page = param.page.to_offset_page_param();

    let logs = req_dao
        .web_dao
        .web_files
        .collector
        .list_record_logs(&record, param.level, &page)
        .await?;

    let items: Vec<serde_json::Value> = logs
        .iter()
        .map(|log| {
            json!({
                "id": log.id,
                "request_id": log.request_id,
                "script_id": log.script_id,
                "user_id": log.user_id,
                "app_id": log.app_id,
                "level": log.level,
                "message": log.message,
                "add_time": log.add_time,
            })
        })
        .collect();

    let total = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .web_files
                .collector
                .count_record_logs(&record, param.level)
                .await?,
        )
    } else {
        None
    };

    Ok(JsonResponse::data(JsonData::body(
        JsonPageData::total(items, total),
    )))
}
