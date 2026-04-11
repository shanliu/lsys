use crate::common::{JsonData, JsonResponse, JsonResult, UserAuthQueryDao};

use crate::handler::api::user::app_collector::app_check_get;
use lsys_access::dao::AccessSession;
use lsys_core::api_utils::JsonPageData;
use serde::Deserialize;
use serde_json::json;
#[derive(Debug, Deserialize)]
pub struct ScriptListParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_i8")]
    pub status: Option<i8>,
    pub page: Option<crate::common::PageParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
}

/// GET /api/user/collector/scripts — 用户应用脚本列表
pub async fn scripts(
    param: &ScriptListParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let _app = app_check_get(param.app_id, false, &auth_data, req_dao).await?;

    use crate::common::ToOffsetPageParam;
    let page = param.page.to_offset_page_param();

    let data = req_dao
        .web_dao
        .web_files
        .collector
        .list_scripts(param.app_id, param.status, &page)
        .await?;

    let total = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .web_files
                .collector
                .count_scripts(param.app_id, param.status)
                .await?,
        )
    } else {
        None
    };

    let items: Vec<serde_json::Value> = data
        .iter()
        .map(|s| {
            json!({
                "id": s.id,
                "add_user_id": s.add_user_id,
                "app_id": s.app_id,
                "name": s.name,
                "script_md5": s.script_md5,
                "timeout_secs": s.timeout_secs,
                "memory_limit": s.memory_limit,
                "status": s.status,
                "add_time": s.add_time,
                "change_time": s.change_time,
            })
        })
        .collect();

    Ok(JsonResponse::data(JsonData::body(JsonPageData::total(
        items, total,
    ))))
}
