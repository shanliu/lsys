//用户文件分片接口

use crate::common::{
    JsonData, JsonResponse, JsonResult, PageParam, ToOffsetPageParam, UserAuthQueryDao,
};
use lsys_access::dao::AccessSession;
use lsys_core::api_utils::JsonPageData;
use lsys_core::db::OffsetPageParam;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct FileChunksParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub file_id: u64,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
    pub page: Option<PageParam>,
}

/// 文件分片列表
pub async fn file_chunks(
    param: &FileChunksParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    // 校验用户对 app 的查看权限
    let _app = super::app_check_get(param.app_id, false, &auth_data, req_dao).await?;

    let page: OffsetPageParam = param.page.to_offset_page_param();

    let data_dao = req_dao.web_dao.web_files.file_dao.data_dao();

    let data = data_dao
        .list_chunks_by_file_id(param.file_id, &page)
        .await?;

    let count = if param.count_num.unwrap_or(false) {
        Some(data_dao.count_chunks_by_file_id(param.file_id).await?)
    } else {
        None
    };

    let items: Vec<serde_json::Value> = data
        .iter()
        .map(|item| {
            json!({
                "id": item.id,
                "file_id": item.file_id,
                "chunk_index": item.chunk_index,
                "start_offset": item.start_offset,
                "chunk_md5": item.chunk_md5,
                "upload_md5": item.upload_md5,
                "chunk_path": item.chunk_path,
                "file_size": item.file_size,
                "complete_size": item.complete_size,
                "status": item.status,
                "add_time": item.add_time,
                "change_time": item.change_time,
            })
        })
        .collect();

    Ok(JsonResponse::data(JsonData::body(JsonPageData::total(
        items, count,
    ))))
}
