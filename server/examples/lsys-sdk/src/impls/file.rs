//! SDK 文件操作方法
//!
//! 提供文件上传、秒传、从 URL/本地导入、列表、删除、URL 获取、详情等方法。

use lsys_core::api_utils::{PageCursorValue, PageTotalRowValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::client::ServiceClient;
use crate::result::ServiceResult;

// ==================== 请求参数类型（内部） ====================

/// 分片参数（请求用，公开类型供外部构造）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChunkParam {
    pub offset: u64,
    pub len: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub md5: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FileUploadCreateParam {
    pub user_id: u64,
    pub app_id: u64,
    pub file_name: String,
    pub chunks: Vec<FileChunkParam>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag_names: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_type: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FileUploadRetokenParam {
    pub user_id: u64,
    pub app_id: u64,
    pub file_ref_id: u64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FileUploadByMd5Param {
    pub user_id: u64,
    pub app_id: u64,
    pub file_md5: String,
    pub file_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag_names: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FileFromUrlParam {
    pub user_id: u64,
    pub app_id: u64,
    pub source_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag_names: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wait_timeout: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_type: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FileListParam {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub add_time_start: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub add_time_end: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<i8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_md5: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag_names: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count_num: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FileFromLocalParam {
    pub user_id: u64,
    pub app_id: u64,
    pub local_file_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag_names: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_type: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FileDeleteParam {
    pub file_ref_id: u64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FileUrlsParam {
    pub file_ids: Vec<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FileInfoParam {
    pub file_ref_ids: Vec<u64>,
}

// ==================== 响应类型（公开） ====================

/// 创建上传任务响应
#[derive(Debug, Clone, Deserialize)]
pub struct FileUploadCreateResponse {
    #[serde(deserialize_with = "crate::utils::deserialize_u64_from_string")]
    pub id: u64,
    #[serde(deserialize_with = "crate::utils::deserialize_u64_from_string")]
    pub file_id: u64,
    pub file_name: String,
    #[serde(deserialize_with = "crate::utils::deserialize_i8_from_string")]
    pub status: i8,
    pub upload_token: String,
}

/// 重新签发令牌响应
#[derive(Debug, Clone, Deserialize)]
pub struct FileUploadRetokenResponse {
    pub upload_token: String,
}

/// MD5 秒传响应
#[derive(Debug, Clone, Deserialize)]
pub struct FileUploadByMd5Response {
    #[serde(deserialize_with = "crate::utils::deserialize_bool_from_string")]
    pub matched: bool,
    #[serde(
        default,
        deserialize_with = "crate::utils::deserialize_option_u64_from_string"
    )]
    pub id: Option<u64>,
}

/// 从 URL 创建文件响应
#[derive(Debug, Clone, Deserialize)]
pub struct FileFromUrlResponse {
    #[serde(deserialize_with = "crate::utils::deserialize_u64_from_string")]
    pub id: u64,
}

/// 从本地文件导入响应
#[derive(Debug, Clone, Deserialize)]
pub struct FileFromLocalResponse {
    #[serde(deserialize_with = "crate::utils::deserialize_u64_from_string")]
    pub id: u64,
    #[serde(deserialize_with = "crate::utils::deserialize_u64_from_string")]
    pub file_id: u64,
    pub file_name: String,
    pub file_md5: String,
    #[serde(deserialize_with = "crate::utils::deserialize_u64_from_string")]
    pub file_size: u64,
    pub file_url: String,
    pub storage_type: String,
    #[serde(deserialize_with = "crate::utils::deserialize_i8_from_string")]
    pub status: i8,
}

/// 文件标签项
#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct FileTagItem {
    pub tag_name: String,
    #[serde(deserialize_with = "crate::utils::deserialize_u64_from_string")]
    pub add_time: u64,
}

/// 文件列表项
#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct FileListItem {
    #[serde(deserialize_with = "crate::utils::deserialize_u64_from_string")]
    pub id: u64,
    #[serde(deserialize_with = "crate::utils::deserialize_u64_from_string")]
    pub file_id: u64,
    pub file_name: String,
    pub file_md5: String,
    #[serde(deserialize_with = "crate::utils::deserialize_u64_from_string")]
    pub file_size: u64,
    pub storage_type: String,
    #[serde(deserialize_with = "crate::utils::deserialize_i8_from_string")]
    pub status: i8,
    pub content_type: String,
    #[serde(default)]
    pub source_url: String,
    #[serde(default)]
    pub file_url: Option<String>,
    #[serde(deserialize_with = "crate::utils::deserialize_u64_from_string")]
    pub add_time: u64,
    #[serde(deserialize_with = "crate::utils::deserialize_u64_from_string")]
    pub user_id: u64,
    #[serde(default)]
    pub local_path: Option<String>,
    #[serde(default)]
    pub tags: Option<Vec<FileTagItem>>,
    // 下载状态（服务端新增）
    #[serde(default)]
    pub is_downloading: bool,
    // 本地文件属性（local attr）
    #[serde(default)]
    pub source_type: Option<String>,
    #[serde(default, deserialize_with = "crate::utils::deserialize_option_u64_from_string")]
    pub file_chunk_total: Option<u64>,
    #[serde(default, deserialize_with = "crate::utils::deserialize_option_u64_from_string")]
    pub file_chunk_succ: Option<u64>,
    // OSS 属性（oss attr）
    #[serde(default)]
    pub object_url: Option<String>,
    #[serde(default)]
    pub bucket: Option<String>,
    #[serde(default)]
    pub region: Option<String>,
}

/// 游标分页响应
#[derive(Debug, Clone, Deserialize)]
pub struct CursorResp {
    #[serde(
        default,
        deserialize_with = "crate::utils::deserialize_option_u64_from_string"
    )]
    pub next: Option<u64>,
    #[serde(
        default,
        deserialize_with = "crate::utils::deserialize_option_u64_from_string"
    )]
    pub prev: Option<u64>,
}

/// 总数响应（精确或近似）
#[derive(Debug, Clone, Deserialize)]
pub struct TotalResp {
    #[serde(
        default,
        deserialize_with = "crate::utils::deserialize_option_u64_from_string"
    )]
    pub exact: Option<u64>,
    #[serde(
        default,
        deserialize_with = "crate::utils::deserialize_option_u64_from_string"
    )]
    pub over: Option<u64>,
}

impl From<CursorResp> for PageCursorValue {
    fn from(value: CursorResp) -> Self {
        Self {
            next: value.next,
            prev: value.prev,
        }
    }
}

impl From<TotalResp> for PageTotalRowValue {
    fn from(value: TotalResp) -> Self {
        Self {
            exact: value.exact,
            over: value.over,
        }
    }
}

/// 文件列表响应
#[derive(Debug, Clone, Deserialize)]
pub struct FileListResponse {
    pub data: Vec<FileListItem>,
    #[serde(default)]
    pub cursor: Option<CursorResp>,
    #[serde(default)]
    pub total: Option<TotalResp>,
}

/// 文件 URL 映射响应
#[derive(Debug, Clone, Deserialize)]
pub struct FileUrlsResponse {
    pub urls: HashMap<String, String>,
}

/// 文件详情项
#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct FileInfoItem {
    #[serde(deserialize_with = "crate::utils::deserialize_u64_from_string")]
    pub id: u64,
    #[serde(deserialize_with = "crate::utils::deserialize_u64_from_string")]
    pub file_id: u64,
    pub file_name: String,
    pub file_md5: String,
    #[serde(deserialize_with = "crate::utils::deserialize_u64_from_string")]
    pub file_size: u64,
    #[serde(deserialize_with = "crate::utils::deserialize_i8_from_string")]
    pub status: i8,
    pub file_url: String,
    pub storage_type: String,
    pub content_type: String,
}

/// 文件详情响应
#[derive(Debug, Clone, Deserialize)]
pub struct FileInfoResponse {
    pub data: Vec<FileInfoItem>,
}

/// 文件配置映射响应
#[derive(Debug, Clone, Deserialize)]
pub struct FileMappingResponse {
    #[serde(deserialize_with = "crate::utils::deserialize_u64_from_string")]
    pub max_upload_size: u64,
    #[serde(deserialize_with = "crate::utils::deserialize_u64_from_string")]
    pub upload_chunk_max: u64,
}

/// 令牌上传分片响应（OSS 风格）
#[derive(Debug, Clone, Deserialize)]
pub struct FileUploadByTokenResponse {
    #[serde(deserialize_with = "crate::utils::deserialize_u64_from_string")]
    pub id: u64,
    #[serde(deserialize_with = "crate::utils::deserialize_u64_from_string")]
    pub file_id: u64,
    #[serde(default, deserialize_with = "crate::utils::deserialize_option_u64_from_string")]
    pub chunk_index: Option<u64>,
    #[serde(deserialize_with = "crate::utils::deserialize_i8_from_string")]
    pub status: i8,
    #[serde(default)]
    pub file_md5: Option<String>,
    #[serde(default)]
    pub file_name: Option<String>,
    #[serde(default, deserialize_with = "crate::utils::deserialize_option_u64_from_string")]
    pub file_size: Option<u64>,
    #[serde(default)]
    pub content_type: Option<String>,
    #[serde(default)]
    pub storage_type: Option<String>,
    #[serde(default)]
    pub file_url: Option<String>,
}

// ==================== ServiceClient 方法实现 ====================

impl ServiceClient {
    /// 创建上传任务，返回含 upload_token
    pub async fn file_upload_create(
        &self,
        user_id: u64,
        app_id: u64,
        file_name: &str,
        chunks: &[FileChunkParam],
        tag_names: Option<&[&str]>,
        storage_type: Option<&str>,
    ) -> ServiceResult<FileUploadCreateResponse> {
        let param = FileUploadCreateParam {
            user_id,
            app_id,
            file_name: file_name.to_string(),
            chunks: chunks.to_vec(),
            tag_names: tag_names.map(|t| t.iter().map(|s| s.to_string()).collect()),
            storage_type: storage_type.map(|s| s.to_string()),
        };

        self.post("/service/file/upload_create")?
            .json(&param)
            .send_json()
            .await
    }

    /// 为未完成文件重新签发上传令牌（断点续传）
    pub async fn file_upload_retoken(
        &self,
        user_id: u64,
        app_id: u64,
        file_ref_id: u64,
    ) -> ServiceResult<FileUploadRetokenResponse> {
        let param = FileUploadRetokenParam {
            user_id,
            app_id,
            file_ref_id,
        };

        self.post("/service/file/upload_retoken")?
            .json(&param)
            .send_json()
            .await
    }

    /// MD5 秒传
    pub async fn file_upload_by_md5(
        &self,
        user_id: u64,
        app_id: u64,
        file_md5: &str,
        file_name: &str,
        tag_names: Option<&[&str]>,
    ) -> ServiceResult<FileUploadByMd5Response> {
        let param = FileUploadByMd5Param {
            user_id,
            app_id,
            file_md5: file_md5.to_string(),
            file_name: file_name.to_string(),
            tag_names: tag_names.map(|t| t.iter().map(|s| s.to_string()).collect()),
        };

        self.post("/service/file/upload_by_md5")?
            .json(&param)
            .send_json()
            .await
    }

    /// 从 URL 创建文件
    pub async fn file_from_url(
        &self,
        user_id: u64,
        app_id: u64,
        source_url: &str,
        tag_names: Option<&[&str]>,
        wait_timeout: Option<u64>,
        storage_type: Option<&str>,
    ) -> ServiceResult<FileFromUrlResponse> {
        let param = FileFromUrlParam {
            user_id,
            app_id,
            source_url: source_url.to_string(),
            tag_names: tag_names.map(|t| t.iter().map(|s| s.to_string()).collect()),
            wait_timeout,
            storage_type: storage_type.map(|s| s.to_string()),
        };

        self.post("/service/file/from_url")?
            .json(&param)
            .send_json()
            .await
    }

    /// 从服务器本地文件导入（服务内部共享磁盘）
    pub async fn file_from_local(
        &self,
        user_id: u64,
        app_id: u64,
        local_file_path: &str,
        file_name: Option<&str>,
        mode: Option<&str>,
        tag_names: Option<&[&str]>,
        storage_type: Option<&str>,
    ) -> ServiceResult<FileFromLocalResponse> {
        let param = FileFromLocalParam {
            user_id,
            app_id,
            local_file_path: local_file_path.to_string(),
            file_name: file_name.map(|s| s.to_string()),
            mode: mode.map(|s| s.to_string()),
            tag_names: tag_names.map(|t| t.iter().map(|s| s.to_string()).collect()),
            storage_type: storage_type.map(|s| s.to_string()),
        };

        self.post("/service/file/from_local")?
            .json(&param)
            .send_json()
            .await
    }

    /// 文件列表查询
    pub async fn file_list(
        &self,
        user_id: Option<u64>,
        app_id: Option<u64>,
        tag_names: Option<&[&str]>,
        limit: Option<u64>,
        cursor: Option<u64>,
        count_num: Option<bool>,
    ) -> ServiceResult<FileListResponse> {
        let param = FileListParam {
            user_id,
            app_id,
            url: None,
            source_url: None,
            add_time_start: None,
            add_time_end: None,
            status: None,
            storage_type: None,
            file_md5: None,
            tag_names: tag_names.map(|t| t.iter().map(|s| s.to_string()).collect()),
            limit,
            cursor,
            count_num,
        };

        self.post("/service/file/list")?
            .json(&param)
            .send_json()
            .await
    }

    /// 文件删除
    pub async fn file_delete(
        &self,
        file_ref_id: u64,
    ) -> ServiceResult<serde_json::Value> {
        let param = FileDeleteParam {
            file_ref_id,
        };

        self.post("/service/file/delete")?
            .json(&param)
            .send_json()
            .await
    }

    /// 批量获取文件 URL
    pub async fn file_urls(&self, file_ids: &[u64]) -> ServiceResult<FileUrlsResponse> {
        let param = FileUrlsParam {
            file_ids: file_ids.to_vec(),
        };

        self.post("/service/file/urls")?
            .json(&param)
            .send_json()
            .await
    }

    /// 文件详情/状态查询（按 file_ref_id）
    pub async fn file_info(&self, file_ref_ids: &[u64]) -> ServiceResult<FileInfoResponse> {
        let param = FileInfoParam {
            file_ref_ids: file_ref_ids.to_vec(),
        };

        self.post("/service/file/info")?
            .json(&param)
            .send_json()
            .await
    }

    /// 获取文件配置映射
    pub async fn file_mapping(&self) -> ServiceResult<FileMappingResponse> {
        self.post("/service/file/mapping")?
            .json(&serde_json::json!({}))
            .send_json()
            .await
    }

    /// service 场景：通过上传令牌直传一个文件分片
    ///
    /// 对应端点 `/service/file/upload_by_token`，仅凭令牌鉴权。
    pub async fn file_upload_by_token(
        &self,
        upload_token: &str,
        chunk_index: u32,
        data: Vec<u8>,
    ) -> ServiceResult<FileUploadByTokenResponse> {
        self.upload_by_token(
            "/service/file/upload_by_token",
            upload_token,
            chunk_index,
            data,
        )
        .await
    }

    /// rest 场景：通过上传令牌直传一个文件分片
    ///
    /// 对应端点 `/rest/file/upload_by_token`，仅凭令牌鉴权。
    pub async fn file_rest_upload_by_token(
        &self,
        upload_token: &str,
        chunk_index: u32,
        data: Vec<u8>,
    ) -> ServiceResult<FileUploadByTokenResponse> {
        self.upload_by_token(
            "/rest/file/upload_by_token",
            upload_token,
            chunk_index,
            data,
        )
        .await
    }
}
