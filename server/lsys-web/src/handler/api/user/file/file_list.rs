//用户文件列表接口

use crate::common::{JsonData, JsonResponse, JsonResult, UserAuthQueryDao};
use crate::dao::access::api::system::user::CheckUserFileView;
use crate::dao::access::RbacAccessCheckEnv;
use lsys_access::dao::AccessSession;
use lsys_core::db::CursorPageSort;
use lsys_files::dao::{FileListFilter, FileListAttrParam};
use lsys_files::model::FileModel;
use serde::Deserialize;
use serde_json::json;
use crate::common::LimitParam;

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
    pub limit: Option<LimitParam>,
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
    let page = param.limit.to_u64_cursor_page_param(CursorPageSort::Desc);

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

    let attr_param = FileListAttrParam {
        attr_local: Some(true),
        attr_oss: Some(true),
    };

    let (data, page_data) = req_dao
        .web_dao
        .web_files
        .file_dao
        .list_files(&filter, &page, &attr_param)
        .await?;

    let mut items: Vec<serde_json::Value> = Vec::with_capacity(data.len());

    // 批量获取文件 URL
    let file_models: Vec<FileModel> = data
        .iter()
        .map(|item| FileModel {
            id: item.item.id,
            storage_type: item.item.storage_type.clone(),
            status: item.item.status,
            file_name: item.item.file_name.clone(),
            file_md5: item.item.file_md5.clone(),
            file_size: item.item.file_size,
            content_type: item.item.content_type.clone(),
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
        let url = url_map.get(&item.item.id).cloned();
        let mut obj = json!({
            "id": item.item.id,
            "file_user_id": item.item.file_user_id,
            "file_name": item.item.file_name,
            "file_md5": item.item.file_md5,
            "file_size": item.item.file_size,
            "storage_type": item.item.storage_type,
            "status": item.item.status,
            "content_type": item.item.content_type,
            "source_url": item.item.source_url,
            "url": url,
            "add_time": item.item.file_user_add_time,
            "user_id": item.item.user_id,
        });
        
        // 摊平 attr_local 数据
        if let Some(local) = &item.attr_local {
            obj["local_id"] = json!(local.id);
            obj["source_type"] = json!(local.source_type);
            obj["local_path"] = json!(local.local_path);
            obj["file_chunk_total"] = json!(local.file_chunk_total);
            obj["file_chunk_succ"] = json!(local.file_chunk_succ);
            obj["file_chunk_size"] = json!(local.file_chunk_size);
        }
        
        // 摊平 attr_oss 数据
        if let Some(oss) = &item.attr_oss {
            obj["oss_id"] = json!(oss.id);
            obj["object_key"] = json!(oss.object_key);
            obj["object_url"] = json!(oss.object_url);
            obj["bucket"] = json!(oss.bucket);
            obj["region"] = json!(oss.region);
            obj["oss_size"] = json!(oss.size);
        }
        
        items.push(obj);
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
