//用户文件上传流接口

use crate::common::{JsonData, JsonResponse, JsonResult, UserAuthQueryDao};
use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::user::CheckUserFileUpload;
use lsys_access::dao::AccessSession;
use lsys_file::dao::{ChunkInfo, FileWriteHandle};
use lsys_file::model::FileStatus;
use serde::Deserialize;
use serde_json::json;

// ==================== 参数定义 ====================

#[derive(Debug, Deserialize)]
pub struct FileUploadCreateParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    pub file_name: String,
    pub chunks: Vec<FileUploadChunkParam>,
    #[serde(default)]
    pub tag_names: Option<Vec<String>>,
    /// 存储类型: local_public / local_private / local_crypto，默认 local_public
    #[serde(default = "default_storage_type")]
    pub storage_type: String,
}

fn default_storage_type() -> String {
    lsys_file::model::FileModel::STORAGE_TYPE_LOCAL_PUBLIC.to_string()
}

#[derive(Debug, Deserialize)]
pub struct FileUploadChunkParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub offset: u64,
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub len: u64,
    #[serde(default)]
    pub md5: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FileUploadByMd5Param {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    pub file_md5: String,
    #[serde(default)]
    pub tag_names: Option<Vec<String>>,
}

// ==================== 处理函数 ====================

/// 根据文件 MD5 秒传
///
/// 客户端先计算文件 MD5，调用此方法判断服务端是否已存在相同文件。
/// 若存在则直接创建关联记录，返回 `file_user_id`，无需上传。
/// 若不存在则返回 `matched: false`，客户端需走正常上传流程。
pub async fn file_upload_by_md5(
    param: &FileUploadByMd5Param,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let user_id = auth_data.user_id();
    let app = super::app_check_get(param.app_id, true, &auth_data, req_dao).await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserFileUpload {
                res_user_id: user_id,
            },
        )
        .await?;

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
            user_id,
            user_id,
            app.id,
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

/// 创建上传任务
pub async fn file_upload_create(
    param: &FileUploadCreateParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let user_id = auth_data.user_id();
    let app = super::app_check_get(param.app_id, true, &auth_data, req_dao).await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserFileUpload {
                res_user_id: user_id,
            },
        )
        .await?;

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
    let (file_id, file_user_id) = req_dao
        .web_dao
        .web_files
        .file_dao
        .create_upload(
            user_id,
            user_id,
            app.id,
            &param.storage_type,
            &chunks,
            &param.file_name,
            &tag_refs,
            Some(&req_dao.req_env),
        )
        .await?;

    Ok(JsonResponse::data(JsonData::body(json!({
        "id": file_user_id,
        "file_id": file_id,
        "file_name": param.file_name,
        "status": FileStatus::Unfinished as i8,
    }))))
}

/// 获取上传写句柄（通过 file_user_id，app_id 自动从 file_user 记录获取）
pub async fn file_upload_handle(
    file_user_id: u64,
    chunk_index: u32,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<FileWriteHandle> {
    let handle = req_dao
        .web_dao
        .web_files
        .file_dao
        .get_upload_handle_by_file_user_id(file_user_id, chunk_index)
        .await?;
    Ok(handle)
}

/// 写入上传数据（代理函数，供外部组合调用，可多次调用）
pub async fn file_upload_write(
    handle: &mut FileWriteHandle,
    data: &[u8],
    req_dao: &UserAuthQueryDao,
) -> JsonResult<usize> {
    let written = req_dao
        .web_dao
        .web_files
        .file_dao
        .write_file(handle, data)
        .await?;
    Ok(written)
}

/// 完成上传（上传流最终步骤：成功完结）
pub async fn file_upload_complete(
    handle: FileWriteHandle,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    req_dao
        .web_dao
        .web_files
        .file_dao
        .complete_upload(handle, Some(&req_dao.req_env))
        .await?;

    Ok(JsonResponse::default())
}

/// 上传失败（上传流最终步骤：错误完结）
pub async fn file_upload_fail(
    handle: FileWriteHandle,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    req_dao
        .web_dao
        .web_files
        .file_dao
        .fail_upload(handle, Some(&req_dao.req_env))
        .await?;

    Ok(JsonResponse::default())
}
