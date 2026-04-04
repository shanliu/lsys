//! 服务间文件操作接口
//!
//! 提供 service 层文件操作业务逻辑（无框架依赖，纯 Rust 逻辑），
//! 供 actix-web 路由层调用。

use crate::common::{JsonData, JsonPageData, JsonResponse, JsonResult, RequestDao};
use lsys_core::api_utils::{PageCursorValue, PageTotalRowValue};
use lsys_core::db::TotalParam;
use lsys_files::dao::{ChunkInfo, FileDataListParam, FileListAttrParam, LocalFileMode};
use lsys_files::model::{FileModel, FileStatus, FileUserStatus};
use serde::Deserialize;
use serde_json::json;
use std::path::Path;

// ==================== 参数定义 ====================

/// 创建上传任务参数
#[derive(Debug, Deserialize)]
pub struct UploadCreateParam {
    pub user_id: u64,
    #[serde(default)]
    pub add_user_id: Option<u64>,
    pub app_id: u64,
    pub file_name: String,
    pub chunks: Vec<UploadChunkParam>,
    #[serde(default)]
    pub tag_names: Option<Vec<String>>,
}

/// 分片信息
#[derive(Debug, Deserialize)]
pub struct UploadChunkParam {
    pub offset: u64,
    pub len: u64,
    #[serde(default)]
    pub md5: Option<String>,
}

/// 重新签发令牌参数
#[derive(Debug, Deserialize)]
pub struct UploadRetokenParam {
    pub user_id: u64,
    pub app_id: u64,
    pub file_user_id: u64,
}

/// MD5 秒传参数
#[derive(Debug, Deserialize)]
pub struct UploadByMd5Param {
    pub user_id: u64,
    pub app_id: u64,
    pub file_md5: String,
    #[serde(default)]
    pub tag_names: Option<Vec<String>>,
}

/// 从 URL 创建文件参数
#[derive(Debug, Deserialize)]
pub struct FromUrlParam {
    pub user_id: u64,
    pub app_id: u64,
    pub source_url: String,
    #[serde(default = "default_max_concurrency")]
    pub max_concurrency: u32,
    #[serde(default)]
    pub tag_names: Option<Vec<String>>,
    /// 同步等待秒数：>0=等待指定秒 / 0=无限等 / 不传=异步
    #[serde(default)]
    pub wait_timeout: Option<u64>,
}

fn default_max_concurrency() -> u32 {
    10
}

/// 从本地文件导入参数
#[derive(Debug, Deserialize)]
pub struct FromLocalParam {
    pub user_id: u64,
    pub app_id: u64,
    pub local_file_path: String,
    #[serde(default)]
    pub file_name: Option<String>,
    /// "move" 或 "copy"，默认 "copy"
    #[serde(default = "default_mode")]
    pub mode: String,
    #[serde(default)]
    pub tag_names: Option<Vec<String>>,
}

fn default_mode() -> String {
    "copy".to_string()
}

/// 文件列表参数
#[derive(Debug, Deserialize)]
pub struct FileListParam {
    #[serde(default)]
    pub user_id: Option<u64>,
    #[serde(default)]
    pub app_id: Option<u64>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub source_url: Option<String>,
    #[serde(default)]
    pub add_time_start: Option<u64>,
    #[serde(default)]
    pub add_time_end: Option<u64>,
    #[serde(default)]
    pub status: Option<i8>,
    #[serde(default)]
    pub storage_type: Option<String>,
    #[serde(default)]
    pub file_md5: Option<String>,
    #[serde(default)]
    pub tag_names: Option<Vec<String>>,
    #[serde(default)]
    pub limit: Option<u64>,
    #[serde(default)]
    pub cursor: Option<u64>,
    #[serde(default)]
    pub count_num: Option<bool>,
}

/// 文件删除参数
#[derive(Debug, Deserialize)]
pub struct FileDeleteParam {
    pub file_user_id: u64,
}

/// 批量获取文件 URL 参数
#[derive(Debug, Deserialize)]
pub struct FileUrlsParam {
    pub file_ids: Vec<u64>,
}

/// 批量获取文件详情参数
#[derive(Debug, Deserialize)]
pub struct FileInfoParam {
    pub file_user_ids: Vec<u64>,
}

// ==================== 处理函数 ====================

/// 创建上传任务 + 签发上传令牌
pub async fn upload_create(
    param: &UploadCreateParam,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    let chunks: Vec<ChunkInfo> = param
        .chunks
        .iter()
        .map(|c| ChunkInfo {
            offset: c.offset,
            len: c.len,
            md5: c.md5.clone(),
        })
        .collect();

    // 上传规则校验
    let upload_config = &req_dao.web_dao.web_files.upload_config;
    let total_size: u64 = chunks.iter().map(|c| c.len).sum();
    if total_size > upload_config.max_upload_size {
        return Err(crate::common::JsonError::Message(
            lsys_core::fluent_message!("file-size-too-large",
                {"size": total_size, "max": upload_config.max_upload_size}
            ),
        ));
    }
    if total_size > upload_config.chunk_threshold && chunks.len() <= 1 {
        return Err(crate::common::JsonError::Message(
            lsys_core::fluent_message!("file-chunk-required",
                {"size": total_size, "threshold": upload_config.chunk_threshold}
            ),
        ));
    }

    let tag_refs: Vec<&str> = param
        .tag_names
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .map(String::as_str)
        .collect();

    // 创建上传任务
    let (file_id, file_user_id) = req_dao
        .web_dao
        .web_files
        .file_dao
        .create_upload(
            param.user_id,
            param.add_user_id.unwrap_or(param.user_id),
            param.app_id,
            &chunks,
            &param.file_name,
            &tag_refs,
            Some(&req_dao.req_env),
        )
        .await?;

    // 签发上传令牌
    let upload_token = req_dao
        .web_dao
        .web_files
        .upload_token
        .create_upload_token(file_user_id, param.user_id, param.app_id, None)
        .await?;

    Ok(JsonResponse::data(JsonData::body(json!({
        "id": file_user_id,
        "file_id": file_id,
        "file_name": param.file_name,
        "status": FileStatus::Unfinished as i8,
        "upload_token": upload_token,
    }))))
}

/// 为未完成文件重新签发上传令牌（断点续传）
pub async fn upload_retoken(
    param: &UploadRetokenParam,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    // 查询 file_user 记录
    let file_user = req_dao
        .web_dao
        .web_files
        .file_dao
        .helper()
        .find_file_user_by_id(param.file_user_id)
        .await?
        .ok_or_else(|| {
            crate::common::JsonError::Message(lsys_core::fluent_message!("file-user-not-found"))
        })?;

    // 查询文件
    let file = req_dao
        .web_dao
        .web_files
        .file_dao
        .helper()
        .find_file_by_id(file_user.file_id)
        .await?
        .ok_or_else(|| {
            crate::common::JsonError::Message(lsys_core::fluent_message!("file-not-found"))
        })?;

    // 校验文件状态为 Unfinished
    if file.status != FileStatus::Unfinished as i8 {
        return Err(crate::common::JsonError::Message(
            lsys_core::fluent_message!("file-status-not-unfinished"),
        ));
    }

    // 校验 user_id 匹配（from_user_id 是创建者）
    if file.from_user_id != param.user_id {
        return Err(crate::common::JsonError::Message(
            lsys_core::fluent_message!("file-user-mismatch"),
        ));
    }

    // 签发新令牌
    let upload_token = req_dao
        .web_dao
        .web_files
        .upload_token
        .retoken_upload(param.file_user_id, param.user_id, param.app_id, None, None)
        .await?;

    Ok(JsonResponse::data(JsonData::body(json!({
        "upload_token": upload_token,
    }))))
}

/// MD5 秒传
pub async fn upload_by_md5(
    param: &UploadByMd5Param,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    let tag_refs: Vec<&str> = param
        .tag_names
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .map(String::as_str)
        .collect();

    let result = req_dao
        .web_dao
        .web_files
        .file_dao
        .create_from_md5(
            &param.file_md5,
            param.user_id,
            param.user_id,
            param.app_id,
            &tag_refs,
            Some(&req_dao.req_env),
        )
        .await?;

    match result {
        Some(file_user_id) => Ok(JsonResponse::data(JsonData::body(json!({
            "matched": true,
            "id": file_user_id,
        })))),
        None => Ok(JsonResponse::data(JsonData::body(json!({
            "matched": false,
        })))),
    }
}

/// 从 URL 创建文件（支持同步/异步模式）
pub async fn from_url(param: &FromUrlParam, req_dao: &RequestDao) -> JsonResult<JsonResponse> {
    // 探测 URL 信息
    let url_info = req_dao
        .web_dao
        .web_files
        .file_dao
        .helper()
        .get_url_file_info(&param.source_url, param.max_concurrency as usize)
        .await?;

    // 文件大小校验
    let upload_config = &req_dao.web_dao.web_files.upload_config;
    if let Some(file_size) = url_info.file_size
        && file_size > upload_config.max_upload_size {
            return Err(crate::common::JsonError::Message(
                lsys_core::fluent_message!("file-size-too-large",
                    {"size": file_size, "max": upload_config.max_upload_size}
                ),
            ));
        }

    // 根据探测信息构建分片参数
    let chunks = if let Some(file_size) = url_info.file_size {
        req_dao
            .web_dao
            .web_files
            .file_dao
            .helper()
            .create_concurrent_chunks(file_size, url_info.max_concurrency)?
    } else {
        vec![ChunkInfo {
            offset: 0,
            len: 0,
            md5: None,
        }]
    };

    let tag_refs: Vec<&str> = param
        .tag_names
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .map(String::as_str)
        .collect();

    let file_user_id = req_dao
        .web_dao
        .web_files
        .file_dao
        .create_from_url(
            &param.source_url,
            param.user_id,
            param.user_id,
            param.app_id,
            &chunks,
            url_info.content_type.as_deref(),
            &tag_refs,
            param.wait_timeout,
            Some(&req_dao.req_env),
        )
        .await?;

    Ok(JsonResponse::data(JsonData::body(json!({
        "id": file_user_id,
    }))))
}

/// 从本地文件导入
pub async fn from_local(param: &FromLocalParam, req_dao: &RequestDao) -> JsonResult<JsonResponse> {
    // 安全校验：路径规范化，防止路径遍历攻击
    let local_path = Path::new(&param.local_file_path);
    let canonical_path = local_path.canonicalize().map_err(|e| {
        crate::common::JsonError::Message(lsys_core::fluent_message!("file-local-path-invalid", e))
    })?;

    // 校验路径在 storage_base_path（网盘挂载目录）下
    let base_path = Path::new(
        &req_dao
            .web_dao
            .web_files
            .file_dao
            .config()
            .storage_base_path,
    );
    let canonical_base = base_path.canonicalize().map_err(|e| {
        crate::common::JsonError::Message(lsys_core::fluent_message!(
            "file-storage-path-invalid",
            e
        ))
    })?;

    if !canonical_path.starts_with(&canonical_base) {
        return Err(crate::common::JsonError::Message(
            lsys_core::fluent_message!("file-path-outside-storage"),
        ));
    }

    // 校验文件存在
    if !canonical_path.exists() {
        return Err(crate::common::JsonError::Message(
            lsys_core::fluent_message!("file-local-not-exists"),
        ));
    }

    // 解析 mode
    let mode = match param.mode.as_str() {
        "move" => LocalFileMode::Move,
        _ => LocalFileMode::Copy,
    };

    let tag_refs: Vec<&str> = param
        .tag_names
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .map(String::as_str)
        .collect();

    let (file, _file_user) = req_dao
        .web_dao
        .web_files
        .file_dao
        .create_from_local_file(
            canonical_path.to_str().unwrap_or(&param.local_file_path),
            param.user_id,
            param.user_id,
            param.app_id,
            param.file_name.as_deref(),
            mode,
            None,
            &tag_refs,
            Some(&req_dao.req_env),
        )
        .await?;

    // 获取文件 URL
    let file_url = req_dao
        .web_dao
        .web_files
        .file_dao
        .get_file_url(&file)
        .await?
        .unwrap_or_default();

    Ok(JsonResponse::data(JsonData::body(json!({
        "file_id": file.id,
        "file_name": file.file_name,
        "file_md5": file.file_md5,
        "file_size": file.file_size,
        "file_url": file_url,
        "status": file.status,
    }))))
}

/// 文件列表查询
pub async fn file_list(param: &FileListParam, req_dao: &RequestDao) -> JsonResult<JsonResponse> {
    use lsys_core::db::{
        CursorConfig, CursorLimit, CursorPageDir, CursorPageParam, CursorPageSort,
    };

    let limit_val = param.limit.unwrap_or(20).min(100);
    let page = CursorPageParam::new(
        CursorPageDir::Next,
        CursorConfig::primary(CursorPageSort::Desc),
        param.cursor,
        CursorLimit::Limit {
            limit: limit_val,
            more: false,
        },
    );

    let tag_refs: Option<Vec<&str>> = param
        .tag_names
        .as_ref()
        .map(|v| v.iter().map(String::as_str).collect());

    let filter = FileDataListParam {
        local_url: param.url.as_deref(),
        source_url: param.source_url.as_deref(),
        user_id: param.user_id,
        app_id: param.app_id,
        add_time_start: param.add_time_start,
        add_time_end: param.add_time_end,
        status: param.status,
        storage_type: param.storage_type.as_deref(),
        file_md5: param.file_md5.as_deref(),
        tag_names: tag_refs.as_deref(),
        tag_any_names: None,
    };

    let attr_param = FileListAttrParam {
        attr_local: Some(true),
        attr_oss: Some(true),
        attr_tag: Some(true),
    };

    let (data, page_data) = req_dao
        .web_dao
        .web_files
        .file_dao
        .data_dao()
        .list_files(&filter, &page, &attr_param)
        .await?;

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

    let mut items: Vec<serde_json::Value> = Vec::with_capacity(data.len());
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
            "file_url": url,
            "add_time": item.item.file_user_add_time,
            "user_id": item.item.user_id,
        });

        if let Some(local) = &item.attr_local {
            obj["local_path"] = json!(local.local_path);
            obj["source_type"] = json!(local.source_type);
            obj["file_chunk_total"] = json!(local.file_chunk_total);
            obj["file_chunk_succ"] = json!(local.file_chunk_succ);
        }

        if let Some(oss) = &item.attr_oss {
            obj["object_url"] = json!(oss.object_url);
            obj["bucket"] = json!(oss.bucket);
            obj["region"] = json!(oss.region);
        }

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

/// 文件删除
pub async fn file_delete(
    param: &FileDeleteParam,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    let file_user = req_dao
        .web_dao
        .web_files
        .file_dao
        .helper()
        .find_file_user_by_id(param.file_user_id)
        .await?
        .ok_or_else(|| lsys_files::dao::FileError::Param(lsys_core::fluent_message!("file-not-found")))?;

    let file = req_dao
        .web_dao
        .web_files
        .file_dao
        .helper()
        .find_file_by_id(file_user.file_id)
        .await?
        .ok_or_else(|| lsys_files::dao::FileError::Param(lsys_core::fluent_message!("file-not-found")))?;

    let ctx = req_dao
        .web_dao
        .web_files
        .file_dao
        .create_context(&file_user)
        .with_file(&file)?;
    req_dao
        .web_dao
        .web_files
        .file_dao
        .delete_file(
            ctx,
            Some(&req_dao.req_env),
        )
        .await?;

    Ok(JsonResponse::default())
}

/// 批量获取文件 URL
pub async fn file_urls(param: &FileUrlsParam, req_dao: &RequestDao) -> JsonResult<JsonResponse> {
    let mut files = Vec::with_capacity(param.file_ids.len());
    for &fid in &param.file_ids {
        if let Some(file) = req_dao
            .web_dao
            .web_files
            .file_dao
            .helper()
            .find_file_by_id(fid)
            .await?
        {
            files.push(file);
        }
    }

    let url_map = req_dao
        .web_dao
        .web_files
        .file_dao
        .get_file_urls(&files)
        .await?;

    // 转换 key 为字符串
    let urls: std::collections::HashMap<String, String> = url_map
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect();

    Ok(JsonResponse::data(JsonData::body(json!({
        "urls": urls,
    }))))
}

/// 文件详情查询（按 file_user_id）
pub async fn file_info(param: &FileInfoParam, req_dao: &RequestDao) -> JsonResult<JsonResponse> {
    let mut items: Vec<serde_json::Value> = Vec::with_capacity(param.file_user_ids.len());

    for &fuid in &param.file_user_ids {
        let file_user = req_dao
            .web_dao
            .web_files
            .file_dao
            .helper()
            .find_file_user_by_id(fuid)
            .await?;

        if let Some(fu) = file_user {
            if fu.status == FileUserStatus::Deleted as i8 {
                continue;
            }
            let file = req_dao
                .web_dao
                .web_files
                .file_dao
                .helper()
                .find_file_by_id(fu.file_id)
                .await?;

            if let Some(f) = file {
                let file_url = req_dao
                    .web_dao
                    .web_files
                    .file_dao
                    .get_file_url(&f)
                    .await?
                    .unwrap_or_default();

                items.push(json!({
                    "id": fu.id,
                    "file_id": f.id,
                    "file_name": f.file_name,
                    "file_md5": f.file_md5,
                    "file_size": f.file_size,
                    "status": f.status,
                    "file_url": file_url,
                    "storage_type": f.storage_type,
                    "content_type": f.content_type,
                }));
            }
        }
    }

    Ok(JsonResponse::data(JsonData::body(json!({
        "data": items,
    }))))
}

/// 获取文件配置映射
pub async fn mapping(_param: &(), req_dao: &RequestDao) -> JsonResult<JsonResponse> {
    let config = req_dao.web_dao.web_files.file_dao.config();
    let upload_config = &req_dao.web_dao.web_files.upload_config;

    Ok(JsonResponse::data(JsonData::body(json!({
        "min_chunk_size": config.min_chunk_size,
        "max_upload_size": upload_config.max_upload_size,
        "chunk_threshold": upload_config.chunk_threshold,
        "default_chunk_size": upload_config.default_chunk_size,
    }))))
}
