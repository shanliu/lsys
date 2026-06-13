//! REST 文件操作接口 — 对外应用调用
//!
//! 与 service 层一一对应，区别在于：
//! - 身份来自 rest 鉴权后的 `AppModel`（`app.id` 即 app_id，`app.user_id` 为归属用户）
//! - 统一做 `CheckRestApp` 权限校验
//! - 上传令牌（upload_create / upload_retoken）签发时 app_id 必为非 0（rest 场景），
//!   文件完成后将按 app_id 触发上传完成回调通知

use crate::common::{JsonData, JsonResponse, JsonResult, RequestDao};
use crate::dao::WebDao;
use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::rest::CheckRestApp;
use lsys_app::model::AppModel;
use lsys_file::dao::{ChunkInfo, FileDataListParam};
use lsys_file::model::{FileStatus, FileUserStatus};
use serde::Deserialize;
use serde_json::json;

// ==================== 参数定义 ====================

/// 创建上传任务参数
#[derive(Debug, Deserialize)]
pub struct UploadCreateParam {
    pub file_name: String,
    pub chunks: Vec<UploadChunkParam>,
    #[serde(default)]
    pub tag_names: Option<Vec<String>>,
    /// 存储类型, 默认 local_public
    #[serde(default = "default_storage_type")]
    pub storage_type: String,
}

/// 分片信息
#[derive(Debug, Deserialize)]
pub struct UploadChunkParam {
    pub offset: u64,
    pub len: u64,
    #[serde(default)]
    pub md5: Option<String>,
}

/// 重新签发单文件令牌参数（断点续传）
#[derive(Debug, Deserialize)]
pub struct UploadRetokenParam {
    pub file_ref_id: u64,
}

/// 创建分片上传会话参数
#[derive(Debug, Deserialize)]
pub struct UploadMultipartCreateParam {
    pub file_name: String,
    pub chunks: Vec<UploadChunkParam>,
    #[serde(default)]
    pub tag_names: Option<Vec<String>>,
    /// 存储类型，默认 local_public
    #[serde(default = "default_storage_type")]
    pub storage_type: String,
}

/// 为分片签发短时令牌参数
#[derive(Debug, Deserialize)]
pub struct UploadPartTokenParam {
    pub session_id: String,
    pub part_number: u32,
}

/// 中止分片上传参数
#[derive(Debug, Deserialize)]
pub struct UploadAbortMultipartParam {
    pub session_id: String,
}

/// MD5 秒传参数
#[derive(Debug, Deserialize)]
pub struct UploadByMd5Param {
    pub file_md5: String,
    pub file_name: String,
    #[serde(default)]
    pub tag_names: Option<Vec<String>>,
}

/// 从 URL 创建文件参数
#[derive(Debug, Deserialize)]
pub struct FromUrlParam {
    pub source_url: String,
    #[serde(default)]
    pub tag_names: Option<Vec<String>>,
    /// 同步等待秒数：>0=等待指定秒 / 0=无限等 / 不传=异步
    #[serde(default)]
    pub wait_timeout: Option<u64>,
    /// 存储类型, 默认 local_public
    #[serde(default = "default_storage_type")]
    pub storage_type: String,
}

fn default_storage_type() -> String {
    lsys_file::model::FileModel::STORAGE_TYPE_LOCAL_PUBLIC.to_string()
}

/// 文件列表参数
#[derive(Debug, Deserialize)]
pub struct FileListParam {
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
    pub file_ref_id: u64,
}

/// 批量获取文件 URL 参数
#[derive(Debug, Deserialize)]
pub struct FileUrlsParam {
    pub file_ids: Vec<u64>,
}

/// 批量获取文件详情参数
#[derive(Debug, Deserialize)]
pub struct FileInfoParam {
    pub file_ref_ids: Vec<u64>,
}

// ==================== 内部辅助 ====================

/// rest 应用权限校验
async fn check_rest_app(
    app: &AppModel,
    req_dao: &RequestDao,
    web_dao: &WebDao,
) -> JsonResult<()> {
    let app_user = web_dao
        .web_access
        .access_dao
        .user
        .cache()
        .find_by_id(&app.user_id)
        .await?;
    web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::user(&app_user, &req_dao.req_env),
            &CheckRestApp {},
        )
        .await?;
    Ok(())
}

// ==================== 处理函数 ====================

/// 创建上传任务 + 签发上传令牌
pub async fn upload_create(
    param: &UploadCreateParam,
    app: &AppModel,
    req_dao: &RequestDao,
    web_dao: &WebDao,
) -> JsonResult<JsonResponse> {
    check_rest_app(app, req_dao, web_dao).await?;

    let chunks: Vec<ChunkInfo> = param
        .chunks
        .iter()
        .map(|c| ChunkInfo {
            offset: c.offset,
            len: c.len,
            md5: c.md5.clone(),
        })
        .collect();

    // 上传规则校验（与 api / service 通道保持一致的友好提示）
    let total_size: u64 = chunks.iter().map(|c| c.len).sum();
    let max_upload_size = web_dao
        .web_file
        .file_dao
        .runtime_setting()
        .get_upload_max_file_size()
        .await
        .unwrap_or(0);
    if max_upload_size > 0 && total_size > max_upload_size {
        return Err(crate::common::JsonError::Message(
            lsys_core::fluent_message!("file-size-too-large",
                {"size": total_size, "max": max_upload_size}
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
    let (file_id, file_ref_id) = web_dao
        .web_file
        .file_dao
        .create_upload(
            app.user_id,
            app.user_id,
            app.id,
            &param.storage_type,
            &chunks,
            &param.file_name,
            &tag_refs,
            None, // expire_time
            Some(&req_dao.req_env),
        )
        .await?;

    // 签发上传令牌（rest 场景 app_id 非 0）
    let upload_token = web_dao
        .web_file
        .upload_token
        .create_token(file_ref_id, app.user_id, app.id, None)
        .await?;

    Ok(JsonResponse::data(JsonData::body(json!({
        "id": file_ref_id,
        "file_id": file_id,
        "file_name": param.file_name,
        "status": FileStatus::Unfinished as i8,
        "upload_token": upload_token,
    }))))
}

/// 为未完成文件重新签发上传令牌（断点续传）
pub async fn upload_retoken(
    param: &UploadRetokenParam,
    app: &AppModel,
    req_dao: &RequestDao,
    web_dao: &WebDao,
) -> JsonResult<JsonResponse> {
    check_rest_app(app, req_dao, web_dao).await?;

    // 查询 file_ref 记录
    let file_ref = web_dao
        .web_file
        .file_dao
        .helper()
        .find_file_ref_by_id(param.file_ref_id)
        .await?
        .ok_or_else(|| {
            crate::common::JsonError::Message(lsys_core::fluent_message!("file-user-not-found"))
        })?;

    // 查询文件
    let file = web_dao
        .web_file
        .file_dao
        .helper()
        .find_file_by_id(file_ref.file_id)
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

    // 校验归属应用
    if file_ref.app_id != app.id {
        return Err(crate::common::JsonError::Message(
            lsys_core::fluent_message!("file-user-mismatch"),
        ));
    }

    // 重新签发令牌（覆盖旧令牌）
    let upload_token = web_dao
        .web_file
        .upload_token
        .create_token(param.file_ref_id, app.user_id, app.id, None)
        .await?;

    Ok(JsonResponse::data(JsonData::body(json!({
        "upload_token": upload_token,
    }))))
}

/// MD5 秒传
pub async fn upload_by_md5(
    param: &UploadByMd5Param,
    app: &AppModel,
    req_dao: &RequestDao,
    web_dao: &WebDao,
) -> JsonResult<JsonResponse> {
    check_rest_app(app, req_dao, web_dao).await?;

    let tag_refs: Vec<&str> = param
        .tag_names
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .map(String::as_str)
        .collect();

    let result = web_dao
        .web_file
        .file_dao
        .create_from_md5(
            &param.file_md5,
            app.user_id,
            app.user_id,
            app.id,
            &param.file_name,
            &tag_refs,
            Some(&req_dao.req_env),
        )
        .await?;

    match result {
        Some(file_ref_id) => Ok(JsonResponse::data(JsonData::body(json!({
            "matched": true,
            "id": file_ref_id,
        })))),
        None => Ok(JsonResponse::data(JsonData::body(json!({
            "matched": false,
        })))),
    }
}

/// 从 URL 创建文件（支持同步/异步模式）
pub async fn from_url(
    param: &FromUrlParam,
    app: &AppModel,
    req_dao: &RequestDao,
    web_dao: &WebDao,
) -> JsonResult<JsonResponse> {
    check_rest_app(app, req_dao, web_dao).await?;

    let tag_refs: Vec<&str> = param
        .tag_names
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .map(String::as_str)
        .collect();

    let file_ref_id = web_dao
        .web_file
        .file_dao
        .create_from_url_auto(
            &param.source_url,
            app.user_id,
            app.user_id,
            app.id,
            &param.storage_type,
            &tag_refs,
            None, // expire_time
            param.wait_timeout,
            Some(&req_dao.req_env),
        )
        .await?;

    Ok(JsonResponse::data(JsonData::body(json!({
        "id": file_ref_id,
    }))))
}

/// 文件列表查询（限定当前应用）
pub async fn file_list(
    param: &FileListParam,
    app: &AppModel,
    req_dao: &RequestDao,
    web_dao: &WebDao,
) -> JsonResult<JsonResponse> {
    check_rest_app(app, req_dao, web_dao).await?;

    let tag_refs: Option<Vec<&str>> = param
        .tag_names
        .as_ref()
        .map(|v| v.iter().map(String::as_str).collect());

    let filter = FileDataListParam {
        local_url: param.url.as_deref(),
        source_url: param.source_url.as_deref(),
        user_id: None,
        // rest 场景固定限定为当前应用
        app_id: Some(app.id),
        add_time_start: param.add_time_start,
        add_time_end: param.add_time_end,
        status: param.status,
        storage_type: param.storage_type.as_deref(),
        file_md5: param.file_md5.as_deref(),
        tag_names: tag_refs.as_deref(),
    };

    crate::handler::shared::file_view::file_list_response(
        &filter,
        param.cursor,
        param.limit,
        param.count_num.unwrap_or(false),
        web_dao,
    )
    .await
}

/// 文件删除
pub async fn file_delete(
    param: &FileDeleteParam,
    app: &AppModel,
    req_dao: &RequestDao,
    web_dao: &WebDao,
) -> JsonResult<JsonResponse> {
    check_rest_app(app, req_dao, web_dao).await?;

    let file_ref = web_dao
        .web_file
        .file_dao
        .helper()
        .find_file_ref_by_id(param.file_ref_id)
        .await?
        .ok_or_else(|| {
            lsys_file::dao::FileError::Param(lsys_core::fluent_message!("file-not-found"))
        })?;

    // 仅允许删除归属当前应用的文件
    if file_ref.app_id != app.id {
        return Err(crate::common::JsonError::Message(
            lsys_core::fluent_message!("file-user-mismatch"),
        ));
    }

    let file = web_dao
        .web_file
        .file_dao
        .helper()
        .find_file_by_id(file_ref.file_id)
        .await?
        .ok_or_else(|| {
            lsys_file::dao::FileError::Param(lsys_core::fluent_message!("file-not-found"))
        })?;

    let ctx = web_dao
        .web_file
        .file_dao
        .file_ops()
        .create_context(&file_ref)
        .with_file(&file)?;
    web_dao
        .web_file
        .file_dao
        .file_ops()
        .delete_file(ctx, Some(&req_dao.req_env))
        .await?;

    Ok(JsonResponse::default())
}

/// 批量获取文件 URL
pub async fn file_urls(
    param: &FileUrlsParam,
    app: &AppModel,
    req_dao: &RequestDao,
    web_dao: &WebDao,
) -> JsonResult<JsonResponse> {
    check_rest_app(app, req_dao, web_dao).await?;

    let mut files = Vec::with_capacity(param.file_ids.len());
    for &fid in &param.file_ids {
        if let Some(file) = web_dao
            .web_file
            .file_dao
            .helper()
            .find_file_by_id(fid)
            .await?
        {
            files.push(file);
        }
    }

    let url_map = web_dao
        .web_file
        .file_dao
        .data_dao()
        .get_file_urls(&files)
        .await?;

    // 转换 key 为字符串，过滤掉 None 值
    let urls: std::collections::HashMap<String, String> = url_map
        .into_iter()
        .filter_map(|(k, v)| v.map(|url| (k.to_string(), url)))
        .collect();

    Ok(JsonResponse::data(JsonData::body(json!({
        "urls": urls,
    }))))
}

/// 文件详情查询（按 file_ref_id）
pub async fn file_info(
    param: &FileInfoParam,
    app: &AppModel,
    req_dao: &RequestDao,
    web_dao: &WebDao,
) -> JsonResult<JsonResponse> {
    check_rest_app(app, req_dao, web_dao).await?;

    let mut items: Vec<serde_json::Value> = Vec::with_capacity(param.file_ref_ids.len());

    for &fuid in &param.file_ref_ids {
        let file_ref = web_dao
            .web_file
            .file_dao
            .helper()
            .find_file_ref_by_id(fuid)
            .await?;

        if let Some(fu) = file_ref {
            if fu.status == FileUserStatus::Deleted as i8 {
                continue;
            }
            // 仅返回归属当前应用的文件
            if fu.app_id != app.id {
                continue;
            }
            let file = web_dao
                .web_file
                .file_dao
                .helper()
                .find_file_by_id(fu.file_id)
                .await?;

            if let Some(f) = file {
                let file_url = web_dao
                    .web_file
                    .file_dao
                    .data_dao()
                    .get_file_url(&f)
                    .await?
                    .unwrap_or_default();

                items.push(json!({
                    "id": fu.id,
                    "file_id": f.id,
                    "file_name": f.origin_name,
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
pub async fn mapping(
    app: &AppModel,
    req_dao: &RequestDao,
    web_dao: &WebDao,
) -> JsonResult<JsonResponse> {
    check_rest_app(app, req_dao, web_dao).await?;

    let max_upload_size = web_dao
        .web_file
        .file_dao
        .runtime_setting()
        .get_upload_max_file_size()
        .await
        .unwrap_or(0);
    let upload_chunk_max = web_dao.web_file.file_dao.config().upload_chunk_max;

    Ok(JsonResponse::data(JsonData::body(json!({
        "max_upload_size": max_upload_size,
        "upload_chunk_max": upload_chunk_max,
    }))))
}

// ==================== 分片上传 ====================

/// 创建分片上传会话（对应 S3 CreateMultipartUpload）
///
/// 适用于大文件：服务端创建长时会话（默认 12h），返回 `session_id`；
/// 客户端随后为每个分片调用 `upload_part_token` 获取短时令牌后上传。
pub async fn upload_multipart_create(
    param: &UploadMultipartCreateParam,
    app: &AppModel,
    req_dao: &RequestDao,
    web_dao: &WebDao,
) -> JsonResult<JsonResponse> {
    check_rest_app(app, req_dao, web_dao).await?;

    let chunks: Vec<lsys_file::dao::ChunkInfo> = param
        .chunks
        .iter()
        .map(|c| lsys_file::dao::ChunkInfo {
            offset: c.offset,
            len: c.len,
            md5: c.md5.clone(),
        })
        .collect();

    let total_size: u64 = chunks.iter().map(|c| c.len).sum();
    let max_upload_size = web_dao
        .web_file
        .file_dao
        .runtime_setting()
        .get_upload_max_file_size()
        .await
        .unwrap_or(0);
    if max_upload_size > 0 && total_size > max_upload_size {
        return Err(crate::common::JsonError::Message(
            lsys_core::fluent_message!("file-size-too-large",
                {"size": total_size, "max": max_upload_size}
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

    let (file_id, file_ref_id) = web_dao
        .web_file
        .file_dao
        .create_upload(
            app.user_id,
            app.user_id,
            app.id,
            &param.storage_type,
            &chunks,
            &param.file_name,
            &tag_refs,
            None,
            Some(&req_dao.req_env),
        )
        .await?;

    let total_parts = chunks.len() as u32;
    let session_id = web_dao
        .web_file
        .upload_token
        .create_session(file_ref_id, app.user_id, app.id, total_parts, None)
        .await?;

    Ok(JsonResponse::data(JsonData::body(json!({
        "id": file_ref_id,
        "file_id": file_id,
        "file_name": param.file_name,
        "status": lsys_file::model::FileStatus::Unfinished as i8,
        "session_id": session_id,
        "total_parts": total_parts,
    }))))
}

/// 为分片签发短时上传令牌（对应 S3 UploadPart Presigned URL）
pub async fn upload_part_token(
    param: &UploadPartTokenParam,
    app: &AppModel,
    req_dao: &RequestDao,
    web_dao: &WebDao,
) -> JsonResult<JsonResponse> {
    check_rest_app(app, req_dao, web_dao).await?;
    let _ = req_dao;

    // 校验会话归属
    let session = web_dao
        .web_file
        .upload_token
        .resolve_session(&param.session_id)
        .await?;
    if session.app_id != app.id {
        return Err(crate::common::JsonError::Message(
            lsys_core::fluent_message!("file-user-mismatch"),
        ));
    }

    let part_token = web_dao
        .web_file
        .upload_token
        .create_part_token(&param.session_id, param.part_number, None)
        .await?;

    Ok(JsonResponse::data(JsonData::body(json!({
        "part_token": part_token,
        "part_number": param.part_number,
    }))))
}

/// 中止分片上传，清理会话及所有分片令牌
pub async fn upload_abort_multipart(
    param: &UploadAbortMultipartParam,
    app: &AppModel,
    req_dao: &RequestDao,
    web_dao: &WebDao,
) -> JsonResult<JsonResponse> {
    check_rest_app(app, req_dao, web_dao).await?;
    let _ = req_dao;

    // 校验会话归属
    let session = web_dao
        .web_file
        .upload_token
        .resolve_session(&param.session_id)
        .await?;
    if session.app_id != app.id {
        return Err(crate::common::JsonError::Message(
            lsys_core::fluent_message!("file-user-mismatch"),
        ));
    }

    web_dao
        .web_file
        .upload_token
        .remove_session(&param.session_id)
        .await?;

    Ok(JsonResponse::default())
}

