//! 示例 handler：演示所有文件 SDK 操作
//!
//! 每个 handler 直接调用 `lsys-sdk` 的 `ServiceClient` 方法，
//! 展示 `lsys-app-barcode` 等专项节点如何通过 SDK 完成文件操作。

use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use lsys_core::api_utils::{JsonPageData, PageCursorValue, PageTotalRowValue};
use lsys_sdk::FileChunkParam;
use serde::Deserialize;
use serde_json::json;

use crate::server::AppState;

// ==================== 请求参数 ====================

#[derive(Debug, Deserialize)]
pub struct UploadCreateReq {
    pub user_id: u64,
    pub app_id: u64,
    pub file_name: String,
    pub chunks: Vec<ChunkReq>,
    #[serde(default)]
    pub tag_names: Option<Vec<String>>,
    #[serde(default)]
    pub storage_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChunkReq {
    pub offset: u64,
    pub len: u64,
    #[serde(default)]
    pub md5: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UploadRetokenReq {
    pub user_id: u64,
    pub app_id: u64,
    pub file_ref_id: u64,
}

#[derive(Debug, Deserialize)]
pub struct UploadByMd5Req {
    pub user_id: u64,
    pub app_id: u64,
    pub file_md5: String,
    pub file_name: String,
    #[serde(default)]
    pub tag_names: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct FromUrlReq {
    pub user_id: u64,
    pub app_id: u64,
    pub source_url: String,
    #[serde(default)]
    pub tag_names: Option<Vec<String>>,
    #[serde(default)]
    pub wait_timeout: Option<u64>,
    #[serde(default)]
    pub storage_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FileListReq {
    #[serde(default)]
    pub user_id: Option<u64>,
    #[serde(default)]
    pub app_id: Option<u64>,
    #[serde(default)]
    pub tag_names: Option<Vec<String>>,
    #[serde(default)]
    pub limit: Option<u64>,
    #[serde(default)]
    pub cursor: Option<u64>,
    #[serde(default)]
    pub count_num: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct FileDeleteReq {
    pub file_ref_id: u64,
}

#[derive(Debug, Deserialize)]
pub struct FromLocalReq {
    pub user_id: u64,
    pub app_id: u64,
    pub local_file_path: String,
    #[serde(default)]
    pub file_name: Option<String>,
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub tag_names: Option<Vec<String>>,
    #[serde(default)]
    pub storage_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FileUrlsReq {
    pub file_ids: Vec<u64>,
}

#[derive(Debug, Deserialize)]
pub struct FileInfoReq {
    pub file_ref_ids: Vec<u64>,
}

// ==================== 工具函数 ====================

fn ok_response(data: serde_json::Value) -> Json<serde_json::Value> {
    Json(json!({"result": {"state": "ok"}, "response": data}))
}

fn err_response(msg: &str) -> Json<serde_json::Value> {
    Json(json!({"result": {"state": "error", "message": msg}}))
}

// ==================== Handler 函数 ====================

/// 创建上传任务 + 获取 upload_token
///
/// 返回 file_id + upload_token，客户端凭此令牌直传到 lsys-actix-web 的
/// POST /api/file/upload_by_token 端点
pub async fn demo_upload_create(
    State(state): State<Arc<AppState>>,
    Json(req): Json<UploadCreateReq>,
) -> impl IntoResponse {
    let chunks: Vec<FileChunkParam> = req
        .chunks
        .iter()
        .map(|c| FileChunkParam {
            offset: c.offset,
            len: c.len,
            md5: c.md5.clone(),
        })
        .collect();

    let tag_refs: Option<Vec<&str>> = req
        .tag_names
        .as_ref()
        .map(|t| t.iter().map(String::as_str).collect());

    match state
        .upstream
        .file_upload_create(
            req.user_id,
            req.app_id,
            &req.file_name,
            &chunks,
            tag_refs.as_deref(),
            req.storage_type.as_deref(),
        )
        .await
    {
        Ok(resp) => ok_response(json!({
            "file_id": resp.file_id,
            "file_name": resp.file_name,
            "status": resp.status,
            "upload_token": resp.upload_token,
        })),
        Err(e) => err_response(&e.to_string()),
    }
}

/// 为未完成文件重新签发上传令牌（断点续传）
pub async fn demo_upload_retoken(
    State(state): State<Arc<AppState>>,
    Json(req): Json<UploadRetokenReq>,
) -> impl IntoResponse {
    match state
        .upstream
        .file_upload_retoken(req.user_id, req.app_id, req.file_ref_id)
        .await
    {
        Ok(resp) => ok_response(json!({
            "upload_token": resp.upload_token,
        })),
        Err(e) => err_response(&e.to_string()),
    }
}

/// MD5 秒传
pub async fn demo_upload_by_md5(
    State(state): State<Arc<AppState>>,
    Json(req): Json<UploadByMd5Req>,
) -> impl IntoResponse {
    let tag_refs: Option<Vec<&str>> = req
        .tag_names
        .as_ref()
        .map(|t| t.iter().map(String::as_str).collect());

    match state
        .upstream
        .file_upload_by_md5(req.user_id, req.app_id, &req.file_md5, &req.file_name, tag_refs.as_deref())
        .await
    {
        Ok(resp) => ok_response(json!({
            "matched": resp.matched,
            "id": resp.id,
        })),
        Err(e) => err_response(&e.to_string()),
    }
}

/// 从 URL 创建文件（支持同步/异步模式）
pub async fn demo_from_url(
    State(state): State<Arc<AppState>>,
    Json(req): Json<FromUrlReq>,
) -> impl IntoResponse {
    let tag_refs: Option<Vec<&str>> = req
        .tag_names
        .as_ref()
        .map(|t| t.iter().map(String::as_str).collect());

    match state
        .upstream
        .file_from_url(
            req.user_id,
            req.app_id,
            &req.source_url,
            tag_refs.as_deref(),
            req.wait_timeout,
            req.storage_type.as_deref(),
        )
        .await
    {
        Ok(resp) => ok_response(json!({
            "id": resp.id,
        })),
        Err(e) => err_response(&e.to_string()),
    }
}

/// 文件列表查询
pub async fn demo_file_list(
    State(state): State<Arc<AppState>>,
    Json(req): Json<FileListReq>,
) -> impl IntoResponse {
    let tag_refs: Option<Vec<&str>> = req
        .tag_names
        .as_ref()
        .map(|t| t.iter().map(String::as_str).collect());

    match state
        .upstream
        .file_list(
            req.user_id,
            req.app_id,
            tag_refs.as_deref(),
            req.limit,
            req.cursor,
            req.count_num,
        )
        .await
    {
        Ok(resp) => {
            let cursor = resp
                .cursor
                .map(PageCursorValue::from)
                .unwrap_or(PageCursorValue {
                    next: None,
                    prev: None,
                });
            let page_data = JsonPageData::cursor(
                serde_json::to_value(&resp.data).unwrap_or_default(),
                cursor,
                resp.total.map(PageTotalRowValue::from),
            );
            match serde_json::to_value(page_data) {
                Ok(val) => ok_response(val),
                Err(e) => err_response(&e.to_string()),
            }
        }
        Err(e) => err_response(&e.to_string()),
    }
}

/// 文件删除
pub async fn demo_file_delete(
    State(state): State<Arc<AppState>>,
    Json(req): Json<FileDeleteReq>,
) -> impl IntoResponse {
    match state
        .upstream
        .file_delete(req.file_ref_id)
        .await
    {
        Ok(_) => ok_response(json!({})),
        Err(e) => err_response(&e.to_string()),
    }
}

/// 批量获取文件 URL
pub async fn demo_file_urls(
    State(state): State<Arc<AppState>>,
    Json(req): Json<FileUrlsReq>,
) -> impl IntoResponse {
    match state.upstream.file_urls(&req.file_ids).await {
        Ok(resp) => ok_response(json!({
            "urls": resp.urls,
        })),
        Err(e) => err_response(&e.to_string()),
    }
}

/// 文件详情/状态查询
pub async fn demo_file_info(
    State(state): State<Arc<AppState>>,
    Json(req): Json<FileInfoReq>,
) -> impl IntoResponse {
    match state.upstream.file_info(&req.file_ref_ids).await {
        Ok(resp) => ok_response(json!({
            "data": serde_json::to_value(&resp.data).unwrap_or_default(),
        })),
        Err(e) => err_response(&e.to_string()),
    }
}

/// 获取文件配置映射
pub async fn demo_mapping(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.upstream.file_mapping().await {
        Ok(resp) => ok_response(json!({
            "max_upload_size": resp.max_upload_size,
            "upload_chunk_max": resp.upload_chunk_max,
        })),
        Err(e) => err_response(&e.to_string()),
    }
}

/// 从服务器本地文件导入
pub async fn demo_from_local(
    State(state): State<Arc<AppState>>,
    Json(req): Json<FromLocalReq>,
) -> impl IntoResponse {
    let tag_refs: Option<Vec<&str>> = req
        .tag_names
        .as_ref()
        .map(|t| t.iter().map(String::as_str).collect());

    match state
        .upstream
        .file_from_local(
            req.user_id,
            req.app_id,
            &req.local_file_path,
            req.file_name.as_deref(),
            req.mode.as_deref(),
            tag_refs.as_deref(),
            req.storage_type.as_deref(),
        )
        .await
    {
        Ok(resp) => ok_response(json!({
            "id": resp.id,
            "file_id": resp.file_id,
            "file_name": resp.file_name,
            "file_md5": resp.file_md5,
            "file_size": resp.file_size,
            "file_url": resp.file_url,
            "storage_type": resp.storage_type,
            "status": resp.status,
        })),
        Err(e) => err_response(&e.to_string()),
    }
}
