use crate::common::{JsonData, JsonResponse, JsonResult, UserAuthQueryDao};
use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::admin::CheckAdminFileManage;
use lsys_access::dao::AccessSession;
use lsys_core::api_utils::{JsonPageData, PageCursorValue, PageTotalRowValue};
use lsys_core::db::{CursorPageSort, TotalParam};
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct RecordFilesParam {
    pub request_id: String,
    pub limit: Option<crate::common::LimitParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
}

/// GET /api/system/collector/record_files — 记录关联文件列表
pub async fn record_files(
    param: &RecordFilesParam,
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

    use crate::common::ToCursorPageParam;
    let page = param.limit.to_u64_cursor_page_param(CursorPageSort::Desc);

    let (files, page_data) = req_dao
        .web_dao
        .web_files
        .collector
        .list_record_files(&record, &page, None)
        .await?;

    let items: Vec<serde_json::Value> = files
        .iter()
        .map(|item| {
            let tags: Vec<serde_json::Value> = item
                .tags
                .iter()
                .map(|tag| {
                    json!({
                        "tag_name": tag.tag_name,
                        "add_time": tag.add_time,
                    })
                })
                .collect();
            json!({
                "file_id": item.file_id,
                "file_name": item.file_name,
                "file_md5": item.file_md5,
                "file_size": item.file_size,
                "storage_type": item.storage_type,
                "content_type": item.content_type,
                "file_url": item.file_url,
                "add_time": item.add_time,
                "tags": tags,
            })
        })
        .collect();

    let total = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .web_files
                .collector
                .count_record_files(&record, None, &TotalParam::default())
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
