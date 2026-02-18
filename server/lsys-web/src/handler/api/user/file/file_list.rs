//用户文件列表接口

use crate::common::{JsonData, JsonResponse, JsonResult, UserAuthQueryDao};
use crate::dao::access::api::system::user::CheckUserFileView;
use crate::dao::access::RbacAccessCheckEnv;
use lsys_access::dao::AccessSession;
use lsys_core::db::CursorPageSort;
use lsys_files::dao::FileListFilter;
use lsys_files::model::FileModel;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct FileListParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u64")]
    pub user_id: Option<u64>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub source_url: Option<String>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u64")]
    pub add_time_start: Option<u64>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u64")]
    pub add_time_end: Option<u64>,
    #[serde(default)]
    pub storage_type: Option<String>,
    #[serde(default)]
    pub file_md5: Option<String>,
    pub page: Option<crate::common::LimitParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
}

/// 文件列表
pub async fn file_list(
    param: &FileListParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let user_id = auth_data.user_id();
    let app = super::app_check_get(param.app_id, false, &auth_data, req_dao).await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserFileView {
                res_user_id: user_id,
            },
        )
        .await?;

    use crate::common::ToCursorPageParam;
    let page = param.page.to_u64_cursor_page_param(CursorPageSort::Desc);

    let filter = FileListFilter {
        url: param.url.clone(),
        source_url: param.source_url.clone(),
        user_id: Some(param.user_id.unwrap_or(user_id)),
        app_id: Some(app.id),
        add_time_start: param.add_time_start,
        add_time_end: param.add_time_end,
        status: None,
        storage_type: param.storage_type.clone(),
        file_md5: param.file_md5.clone(),
    };

    let (data, page_data) = req_dao
        .web_dao
        .web_files
        .file_dao
        .list_files(&filter, &page)
        .await?;

    let mut items: Vec<serde_json::Value> = Vec::with_capacity(data.len());

    // 批量获取文件 URL
    let file_models: Vec<FileModel> = data
        .iter()
        .map(|item| FileModel {
            id: item.id,
            storage_type: item.storage_type.clone(),
            status: item.status,
            file_name: item.file_name.clone(),
            file_md5: item.file_md5.clone(),
            file_size: item.file_size,
            content_type: item.content_type.clone(),
            ..Default::default()
        })
        .collect();
    let url_map = req_dao
        .web_dao
        .web_files
        .file_dao
        .get_file_urls(&file_models)
        .await
        .unwrap_or_default();

    for item in &data {
        let url = url_map.get(&item.id).cloned();
        items.push(json!({
            "id": item.id,
            "file_user_id": item.file_user_id,
            "file_name": item.file_name,
            "file_md5": item.file_md5,
            "file_size": item.file_size,
            "storage_type": item.storage_type,
            "status": item.status,
            "content_type": item.content_type,
            "source_url": item.source_url,
            "url": url,
            "add_time": item.file_user_add_time,
            "user_id": item.user_id,
        }));
    }

    let total = if param.count_num.unwrap_or(false) {
        Some(req_dao.web_dao.web_files.file_dao.count_files(&filter).await?)
    } else {
        None
    };

    Ok(JsonResponse::data(JsonData::body(json!({
        "data": items,
        "next_cursor": page_data.next_cursor,
        "prev_cursor": page_data.prev_cursor,
        "total": total,
    }))))
}
