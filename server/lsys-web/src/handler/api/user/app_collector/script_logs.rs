use crate::common::{JsonData, JsonResponse, JsonResult, RequestDao, UserAuthQueryDao};
use crate::dao::WebDao;
use crate::handler::api::user::app_collector::app_check_get;
use lsys_access::dao::AccessSession;
use lsys_core::api_utils::{JsonPageData, PageCursorValue, PageTotalRowValue};
use lsys_core::db::{CursorPageSort, TotalParam};
use serde::Deserialize;
use serde_json::json;
#[derive(Debug, Deserialize)]
pub struct ScriptLogsParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub script_id: u64,
    #[serde(default)]
    pub request_id: Option<String>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u8")]
    pub level: Option<u8>,
    pub limit: Option<crate::common::LimitParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
}

pub async fn script_logs(
    param: &ScriptLogsParam,
    req_dao: &RequestDao,
    auth_dao: &UserAuthQueryDao,
    web_dao: &WebDao,
) -> JsonResult<JsonResponse> {
    let auth_data = auth_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await?;

    let script = web_dao
        .web_collector.collector
        .find_script_by_id(param.script_id)
        .await?
        .ok_or_else(|| {
            crate::common::JsonError::Message(lsys_core::fluent_message!(
                "collector-script-not-found"
            ))
        })?;
    app_check_get(script.app_id, false, &auth_data, req_dao, web_dao).await?;

    use crate::common::ToCursorPageParam;
    let page = param.limit.to_u64_cursor_page_param(CursorPageSort::Desc);

    let (log_list, page_data) = web_dao
        .web_collector.collector
        .list_logs(
            &script,
            param.request_id.as_deref(),
            param.level,
            &page,
        )
        .await?;

    let items: Vec<serde_json::Value> = log_list
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
            web_dao
                .web_collector.collector
                .count_logs(
                    &script,
                    param.request_id.as_deref(),
                    param.level,
                    &TotalParam::default(),
                )
                .await
                .map(PageTotalRowValue::from)?,
        )
    } else {
        None
    };

    let cursor = PageCursorValue::from(&page_data);
    Ok(JsonResponse::data(JsonData::body(JsonPageData::cursor(
        items, cursor, total,
    ))))
}
