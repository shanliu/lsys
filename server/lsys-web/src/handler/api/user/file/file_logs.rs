//用户文件日志接口

use crate::common::{
    JsonData, JsonResponse, JsonResult, PageParam, ToOffsetPageParam, UserAuthQueryDao,
};
use lsys_access::dao::AccessSession;
use lsys_core::db::OffsetPageParam;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct FileLogsParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub file_id: u64,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
    pub page: Option<PageParam>,
}

/// 文件日志列表
pub async fn file_logs(
    param: &FileLogsParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    // 校验用户对 app 的查看权限
    let _app = super::app_check_get(param.app_id, false, &auth_data, req_dao).await?;

    let page: OffsetPageParam = param.page.to_offset_page_param();

    let data = req_dao
        .web_dao
        .web_files
        .file_dao
        .helper()
        .log_dao()
        .list_by_file_id(param.file_id, &page)
        .await?;

    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .web_files
                .file_dao
                .helper()
                .log_dao()
                .count_by_file_id(param.file_id)
                .await?,
        )
    } else {
        None
    };

    // 批量获取用户信息
    let user_ids: Vec<u64> = data.iter().map(|item| item.user_id).collect();
    let user_data = req_dao
        .web_dao
        .web_access
        .access_dao
        .user
        .cache()
        .find_users_by_ids(&user_ids)
        .await?;

    let items: Vec<serde_json::Value> = data
        .iter()
        .map(|item| {
            json!({
                "id": item.id,
                "file_chunk_id": item.file_chunk_id,
                "message": item.message,
                "user_data": user_data.get(item.user_id),
                "add_time": item.add_time,
            })
        })
        .collect();

    Ok(JsonResponse::data(JsonData::body(json!({
        "data": items,
        "total": count,
    }))))
}
