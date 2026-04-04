//用户文件列表接口

use crate::common::LimitParam;
use crate::common::{JsonData, JsonResponse, JsonResult, UserAuthQueryDao};
use crate::dao::access::api::system::user::{CheckUserFileUpload, CheckUserFileView};
use crate::dao::access::RbacAccessCheckEnv;
use lsys_access::dao::AccessSession;
use lsys_core::api_utils::{JsonPageData, PageCursorValue, PageTotalRowValue};
use lsys_core::db::{CursorPageSort, TotalParam};
use lsys_files::common::FileError;
use lsys_files::dao::{FileDataListParam, FileListAttrParam};
use lsys_files::model::{FileModel, FileUserModel};
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
    #[serde(default)]
    pub status: Option<i8>,
    pub limit: Option<LimitParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
    /// 按标签名过滤
    #[serde(default)]
    pub tag_names: Option<Vec<String>>,
    /// 是否返回完整标签数据（默认 false，只返回 tag_count）
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub attr_tag: Option<bool>,
}

/// 标签名列表查询参数
#[derive(Debug, Deserialize)]
pub struct FileTagNamesParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    /// 标签名前缀过滤
    #[serde(default)]
    pub tag_name_prefix: Option<String>,
    /// 最大返回条数，默认 50
    #[serde(default)]
    pub limit: Option<u32>,
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

    let tag_refs: Option<Vec<&str>> = param
        .tag_names
        .as_ref()
        .map(|v| v.iter().map(String::as_str).collect());
    let filter = FileDataListParam {
        local_url: param.url.as_deref(),
        source_url: param.source_url.as_deref(),
        user_id: Some(param.user_id.unwrap_or(user_id)),
        app_id: Some(app.id),
        add_time_start: param.add_time_start,
        add_time_end: param.add_time_end,
        storage_type: param.storage_type.as_deref(),
        file_md5: param.file_md5.as_deref(),
        status: param.status,
        tag_names: tag_refs.as_deref(),
        tag_any_names: None,
    };

    let need_full_tag = param.attr_tag.unwrap_or(false);
    let attr_param = FileListAttrParam {
        attr_local: Some(true),
        attr_oss: Some(true),
        attr_tag: Some(need_full_tag),
    };

    let (data, page_data) = req_dao
        .web_dao
        .web_files
        .file_dao
        .data_dao()
        .list_files(&filter, &page, &attr_param)
        .await?;

    let mut items: Vec<serde_json::Value> = Vec::with_capacity(data.len());

    // 批量获取文件 URL
    let file_models: Vec<FileModel> = data
        .iter()
        .map(|item| FileModel {
            id: item.item.file_id,
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

    // 批量查询标签计数和第一个标签（仅在不需要完整标签数据时）
    let mut tag_count_map: std::collections::HashMap<u64, i64> = std::collections::HashMap::new();
    let mut first_tag_map: std::collections::HashMap<u64, lsys_files::model::FileTagModel> =
        std::collections::HashMap::new();
    if !need_full_tag && !data.is_empty() {
        let file_ids: Vec<u64> = data.iter().map(|item| item.item.file_id).collect();
        for &fid in &file_ids {
            let count = req_dao
                .web_dao
                .web_files
                .file_dao
                .data_dao()
                .count_tags_by_file(fid, user_id, app.id)
                .await
                .unwrap_or(0);
            tag_count_map.insert(fid, count);
        }
        // 批量获取每个文件的第一个标签
        let tags_with_count: Vec<u64> = tag_count_map
            .iter()
            .filter(|&(_, &count)| count > 0)
            .map(|(&fid, _)| fid)
            .collect();
        for fid in tags_with_count {
            if let Ok(tags) = req_dao
                .web_dao
                .web_files
                .file_dao
                .data_dao()
                .list_tags_by_file(fid, user_id, app.id)
                .await
                && let Some(first) = tags.into_iter().next() {
                    first_tag_map.insert(fid, first);
                }
        }
    }

    for item in &data {
        let url = url_map.get(&item.item.file_id).cloned();
        let mut obj = json!({
            "id": item.item.id,
            "file_id": item.item.file_id,
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

        // 摊平 attr_tag 数据
        if let Some(tag_attr) = &item.attr_tag {
            let tags: Vec<serde_json::Value> = tag_attr
                .tags
                .iter()
                .map(|t| {
                    json!({
                        "tag_name": t.tag_name,
                        "add_time": t.add_time,
                    })
                })
                .collect();
            obj["tags"] = json!(tags);
            obj["tag_count"] = json!(tag_attr.tags.len());
        }

        // 如果不需要完整标签数据，使用预查询的标签数量
        if !need_full_tag {
            let tag_count = tag_count_map.get(&item.item.file_id).copied().unwrap_or(0);
            obj["tag_count"] = json!(tag_count);
            // 仅返回第一个标签用于列表预览
            if tag_count > 0 {
                let first_tag = first_tag_map.get(&item.item.file_id);
                if let Some(first) = first_tag {
                    obj["first_tag"] = json!({
                        "tag_name": first.tag_name,
                        "add_time": first.add_time,
                    });
                }
            }
        }

        items.push(obj);
    }

    let total = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .web_files
                .file_dao
                .data_dao()
                .count_files(&filter, &TotalParam::default())
                .await
                .map(PageTotalRowValue::from)?,
        )
    } else {
        None
    };

    let cursor = PageCursorValue::from(&page_data);
    Ok(JsonResponse::data(JsonData::body(
        JsonPageData::cursor(items, cursor, total),
    )))
}

/// 查询当前用户某应用下的标签名列表（去重，支持前缀过滤）
pub async fn file_tag_names(
    param: &FileTagNamesParam,
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

    let limit = param.limit.unwrap_or(50).min(200);
    let tag_names = req_dao
        .web_dao
        .web_files
        .file_dao
        .data_dao()
        .list_tag_names_by_user(user_id, app.id, param.tag_name_prefix.as_deref(), limit)
        .await?;

    Ok(JsonResponse::data(JsonData::body(json!({
        "data": tag_names,
    }))))
}

// ==================== 文件标签管理接口 ====================

/// 通过 file_user_id 查找文件归属记录，返回 FileUserModel
async fn resolve_file_user(
    file_user_id: u64,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<FileUserModel> {
    let file_user = req_dao
        .web_dao
        .web_files
        .file_dao
        .helper()
        .find_file_user_by_id(file_user_id)
        .await?;
    file_user.ok_or_else(|| FileError::Param(lsys_core::fluent_message!("param-error")).into())
}

/// 查询单个文件的标签列表参数
#[derive(Debug, Deserialize)]
pub struct FileTagsParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub id: u64,
}

/// 查询单个文件的所有标签
pub async fn file_tags(
    param: &FileTagsParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let file_user = resolve_file_user(param.id, req_dao).await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserFileView {
                res_user_id: file_user.user_id,
            },
        )
        .await?;

    let tags = req_dao
        .web_dao
        .web_files
        .file_dao
        .data_dao()
        .list_tags_by_file(file_user.file_id, file_user.user_id, file_user.app_id)
        .await?;

    let tag_items: Vec<serde_json::Value> = tags
        .iter()
        .map(|t| {
            json!({
                "id": t.id,
                "tag_name": t.tag_name,
                "add_time": t.add_time,
            })
        })
        .collect();

    Ok(JsonResponse::data(JsonData::body(json!({
        "data": tag_items,
    }))))
}

/// 添加标签参数
#[derive(Debug, Deserialize)]
pub struct FileTagAddParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub id: u64,
    pub tag_name: String,
}

/// 为文件添加标签
pub async fn file_tag_add(
    param: &FileTagAddParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let file_user = resolve_file_user(param.id, req_dao).await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserFileUpload {
                res_user_id: file_user.user_id,
            },
        )
        .await?;

    let tag_id = req_dao
        .web_dao
        .web_files
        .file_dao
        .tag_dao()
        .add_tag(
            file_user.file_id,
            file_user.user_id,
            file_user.app_id,
            &param.tag_name,
            None,
        )
        .await?;

    Ok(JsonResponse::data(JsonData::body(json!({
        "id": tag_id,
    }))))
}

/// 删除标签参数
#[derive(Debug, Deserialize)]
pub struct FileTagRemoveParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub id: u64,
    pub tag_name: String,
}

/// 移除文件标签
pub async fn file_tag_remove(
    param: &FileTagRemoveParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let file_user = resolve_file_user(param.id, req_dao).await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserFileUpload {
                res_user_id: file_user.user_id,
            },
        )
        .await?;

    let affected = req_dao
        .web_dao
        .web_files
        .file_dao
        .tag_dao()
        .remove_tag(
            file_user.file_id,
            file_user.user_id,
            file_user.app_id,
            &param.tag_name,
            None,
        )
        .await?;

    Ok(JsonResponse::data(JsonData::body(json!({
        "affected": affected,
    }))))
}
