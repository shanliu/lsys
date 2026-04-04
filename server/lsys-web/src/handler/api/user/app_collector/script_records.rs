use crate::common::{JsonData, JsonResponse, JsonResult, UserAuthQueryDao};
use crate::handler::api::user::app_collector::app_check_get;
use lsys_access::dao::AccessSession;
use lsys_core::api_utils::{JsonPageData, PageCursorValue, PageTotalRowValue};
use lsys_core::db::{CursorPageSort, TotalParam};
use serde::Deserialize;
use serde_json::json;
#[derive(Debug, Deserialize)]
pub struct ScriptRecordsParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub script_id: u64,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_i8")]
    pub status: Option<i8>,
    pub limit: Option<crate::common::LimitParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
}

/// GET /api/user/collector/script_records — 按脚本查记录
pub async fn script_records(
    param: &ScriptRecordsParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let _app = app_check_get(param.app_id, false, &auth_data, req_dao).await?;

    use crate::common::ToCursorPageParam;
    let page = param.limit.to_u64_cursor_page_param(CursorPageSort::Desc);

    let (record_list, page_data) = req_dao
        .web_dao
        .web_files
        .collector
        .list_records(param.script_id, None, param.status, &page)
        .await?;

    let total = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .web_files
                .collector
                .count_records(param.script_id, None, param.status, &TotalParam::default())
                .await
                .map(PageTotalRowValue::from)?,
        )
    } else {
        None
    };

    let record_items: Vec<serde_json::Value> = record_list
        .iter()
        .map(|rec| {
            json!({
                "id": rec.id,
                "request_id": rec.request_id,
                "script_id": rec.script_id,
                "add_user_id": rec.add_user_id,
                "app_id": rec.app_id,
                "task_id": rec.task_id,
                "exec_params": rec.exec_params,
                "status": rec.status,
                "elapsed_ms": rec.elapsed_ms,
                "error_message": rec.error_message,
                "add_time": rec.add_time,
                "start_time": rec.start_time,
                "finish_time": rec.finish_time,
            })
        })
        .collect();

    let cursor = PageCursorValue::from(&page_data);
    Ok(JsonResponse::data(JsonData::body(
        JsonPageData::cursor(record_items, cursor, total),
    )))
}
