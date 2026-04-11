use crate::common::{JsonData, JsonResponse, JsonResult, UserAuthQueryDao};
use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::admin::CheckAdminFileManage;
use lsys_access::dao::AccessSession;
use lsys_core::api_utils::JsonPageData;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct ScriptListParam {
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u64")]
    pub user_id: Option<u64>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_i8")]
    pub status: Option<i8>,
    pub page: Option<crate::common::PageParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
}

/// GET /api/system/collector/scripts — 系统脚本列表 (app_id=0)
pub async fn scripts(
    param: &ScriptListParam,
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

    use crate::common::ToOffsetPageParam;
    let page = param.page.to_offset_page_param();

    let data = req_dao
        .web_dao
        .web_files
        .collector
        .list_scripts(0, param.status, &page)
        .await?;

    let total = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .web_files
                .collector
                .count_scripts(0, param.status)
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
                "app_user_id": s.app_user_id,
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
