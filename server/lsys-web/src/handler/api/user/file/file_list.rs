//用户文件列表接口

use crate::common::LimitParam;
use crate::common::{JsonData, JsonResponse, JsonResult, RequestDao, UserAuthQueryDao};
use crate::dao::WebDao;
use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::user::{CheckUserFileUpload, CheckUserFileView};
use lsys_access::dao::AccessSession;
use lsys_core::api_utils::{JsonPageData, PageCursorValue, PageTotalRowValue};
use lsys_core::db::{CursorPageSort, TotalParam};
use lsys_file::common::FileError;
use lsys_file::dao::{FileDataListParam, FileListAttrParam};
use lsys_file::model::FileRefModel;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct FileListParam {
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
    /// 是否返回关联（lineage）统计数据
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub attr_lineage: Option<bool>,
}

/// 标签名列表查询参数
#[derive(Debug, Deserialize)]
pub struct FileTagNamesParam {
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
    req_dao: &RequestDao,
    auth_dao: &UserAuthQueryDao,
    web_dao: &WebDao,
) -> JsonResult<JsonResponse>
{
    let auth_data = auth_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await?;
    let user_id = auth_data.user_id();

    web_dao
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
        user_id: Some(user_id),
        app_id: Some(0),
        add_time_start: param.add_time_start,
        add_time_end: param.add_time_end,
        storage_type: param.storage_type.as_deref(),
        file_md5: param.file_md5.as_deref(),
        status: param.status,
        tag_names: tag_refs.as_deref(),
    };

    let need_full_tag = param.attr_tag.unwrap_or(false);
    let need_lineage = param.attr_lineage.unwrap_or(false);
    let attr_param = FileListAttrParam {
        attr_local: Some(true),
        attr_oss: Some(true),
        attr_tag_list: Some(need_full_tag as u32 * 3),
        attr_tag_count: Some(need_full_tag),
        attr_lineage: Some(need_lineage),
        attr_url_downloading: Some(true),
    };

    let (data, page_data) = web_dao
        .web_file.file_dao
        .data_dao()
        .list_files(&filter, &page, &attr_param)
        .await?;

    let mut items: Vec<serde_json::Value> = Vec::with_capacity(data.len());

    // 标签数据已通过 attr_tag_count 包含在结果中
    let mut tag_count_map: std::collections::HashMap<u64, i64> = std::collections::HashMap::new();
    let mut first_tag_map: std::collections::HashMap<u64, lsys_file::model::FileTagModel> =
        std::collections::HashMap::new();

    // 从返回的数据中提取标签计数和第一个标签
    for item in &data {
        if let Some(attr_tag) = &item.attr_tag {
            // 获取标签计数（来自 attr_tag_count）
            if let Some(count) = attr_tag.count {
                tag_count_map.insert(item.item.file_id, count);
            }
            // 获取第一个标签（来自 attr_tag_list）
            if let Some(first_tag_item) = attr_tag.tags.first() {
                let tag_model: lsys_file::model::FileTagModel = lsys_file::model::FileTagModel {
                    id: 0,
                    file_id: item.item.file_id,
                    tag_name: first_tag_item.tag_name.clone(),
                    user_id: item.item.user_id,
                    app_id: item.item.app_id,
                    add_time: first_tag_item.add_time,
                    change_time: first_tag_item.add_time,
                    status: 1,
                };
                first_tag_map.insert(item.item.file_id, tag_model);
            }
        }
    }

    for item in &data {
        let mut obj = json!({
            "id": item.item.id,
            "file_id": item.item.file_id,
            "file_key": item.file_key,
            "file_name": item.item.file_name,
            "file_md5": item.item.file_md5,
            "file_size": item.item.file_size,
            "storage_type": item.item.storage_type,
            "status": item.item.status,
            "content_type": item.item.content_type,
            "source_url": item.item.source_url,
            "is_downloading": item.attr_url_downloading,
            "add_time": item.item.file_ref_add_time,
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

        // 摊平 attr_lineage 统计数据
        if let Some(lineage_attr) = &item.attr_lineage {
            let lineage_counts: Vec<serde_json::Value> = lineage_attr
                .counts
                .iter()
                .map(|c| {
                    json!({
                        "rel_type": c.rel_type,
                        "storage_type": c.storage_type,
                        "count": c.count,
                    })
                })
                .collect();
            obj["lineage_counts"] = json!(lineage_counts);
        }

        items.push(obj);
    }

    let total = if param.count_num.unwrap_or(false) {
        Some(
            web_dao
                .web_file.file_dao
                .data_dao()
                .count_files(&filter, &TotalParam::default())
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

/// 查询当前用户某应用下的标签名列表（去重，支持前缀过滤）
pub async fn file_tag_names(
    param: &FileTagNamesParam,
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
    let user_id = auth_data.user_id();
 
    web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserFileView {
                res_user_id: user_id,
            },
        )
        .await?;

    let limit = param.limit.unwrap_or(50).min(200);
    let tag_names = web_dao
        .web_file.file_dao
        .data_dao()
        .list_tag_names_by_user(user_id,0, param.tag_name_prefix.as_deref(), limit)
        .await?;

    Ok(JsonResponse::data(JsonData::body(json!({
        "data": tag_names,
    }))))
}

// ==================== 文件标签管理接口 ====================

/// 通过 file_ref_id 查找文件归属记录，返回 FileRefModel
async fn resolve_file_user(file_ref_id: u64, web_dao: &WebDao) -> JsonResult<FileRefModel> {
    let file_user = web_dao
        .web_file.file_dao
        .helper()
        .find_file_ref_by_id(file_ref_id)
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
    let file_user = resolve_file_user(param.id, web_dao).await?;

    web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserFileView {
                res_user_id: file_user.user_id,
            },
        )
        .await?;

    let tags = web_dao
        .web_file.file_dao
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
    let file_user = resolve_file_user(param.id, web_dao).await?;

    web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserFileUpload {
                res_user_id: file_user.user_id,
            },
        )
        .await?;

    let tag_id = web_dao
        .web_file.file_dao
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
    let file_user = resolve_file_user(param.id, web_dao).await?;

    web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserFileUpload {
                res_user_id: file_user.user_id,
            },
        )
        .await?;

    let affected = web_dao
        .web_file.file_dao
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

// ==================== 文件关联（lineage）查询接口 ====================

/// 查询文件关联详细列表参数（包含完整文件信息）
#[derive(Debug, Deserialize)]
pub struct FileLineageRelatedListParam {
    /// file_ref_id
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub id: u64,
    /// 关联类型（可选）：1=拷贝, 2=转换, 3=OSS同步。为空则返回全部
    #[serde(default)]
    pub rel_type: Option<i8>,
    /// 按存储类型过滤（可选）
    #[serde(default)]
    pub storage_type: Option<String>,
    pub limit: Option<LimitParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
}

/// 查询下载中文件列表参数
#[derive(Debug, Deserialize)]
pub struct FileDownloadingListParam {
    /// 是否正在下载：true=仅下载中, false=仅排队中, 空=全部
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub is_downloading: Option<bool>,
    pub limit: Option<LimitParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
}

/// 查询文件关联详细列表（包含完整文件信息和 file_ref 数据）
pub async fn file_lineage_related_list(
    param: &FileLineageRelatedListParam,
    req_dao: &RequestDao,
    auth_dao: &UserAuthQueryDao,
    web_dao: &WebDao,
) -> JsonResult<JsonResponse>
{
    let auth_data = auth_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await?;
    let file_user = resolve_file_user(param.id, web_dao).await?;

    web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserFileView {
                res_user_id: file_user.user_id,
            },
        )
        .await?;

    use crate::common::ToCursorPageParam;
    use lsys_file::dao::LineageRelatedListParam;

    let page = param.limit.to_u64_cursor_page_param(CursorPageSort::Desc);

    let filter = LineageRelatedListParam {
        rel_type: param.rel_type,
        storage_type: param.storage_type.clone(),
    };

    let attr_param = FileListAttrParam {
        attr_local: Some(true),
        attr_oss: Some(true),
        attr_tag_list: None,
        attr_tag_count: Some(true),
        attr_lineage: None,
        attr_url_downloading: None,
    };

    let (data, page_data) = web_dao
        .web_file.file_dao
        .data_dao()
        .list_lineage_related_files(&file_user, &filter, &page, &attr_param)
        .await?;

    let mut items: Vec<serde_json::Value> = Vec::with_capacity(data.len());
    for item in &data {
        let mut obj = json!({
            "id": item.item.id,
            "file_id": item.item.file_id,
            "file_key": item.file_key,
            "file_name": item.item.file_name,
            "file_md5": item.item.file_md5,
            "file_size": item.item.file_size,
            "storage_type": item.item.storage_type,
            "status": item.item.status,
            "content_type": item.item.content_type,
            "source_url": item.item.source_url,
            "add_time": item.item.file_ref_add_time,
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

        // 标签数量
        if let Some(tag_attr) = &item.attr_tag
            && let Some(count) = tag_attr.count {
                obj["tag_count"] = json!(count);
            }

        items.push(obj);
    }

    let total = if param.count_num.unwrap_or(false) {
        let count = web_dao
            .web_file.file_dao
            .data_dao()
            .count_lineage_related_files(&file_user, &filter)
            .await?;
        Some(PageTotalRowValue {
            exact: Some(count as u64),
            over: None,
        })
    } else {
        None
    };

    let cursor = PageCursorValue::from(&page_data);
    Ok(JsonResponse::data(JsonData::body(JsonPageData::cursor(
        items, cursor, total,
    ))))
}