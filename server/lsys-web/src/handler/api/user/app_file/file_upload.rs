//用户文件上传流接口

use crate::common::{JsonData, JsonResponse, JsonResult, RequestDao, UserAuthQueryDao};
use crate::dao::WebDao;
use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::user::CheckUserFileUpload;
use lsys_access::dao::AccessSession;
use lsys_file::dao::{ChunkInfo, FileWriteHandle};
use lsys_file::model::{FileModel, FileStatus};
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
    pub file_name: String,
    #[serde(default)]
    pub tag_names: Option<Vec<String>>,
}

// ==================== 处理函数 ====================

/// 根据文件 MD5 秒传
///
/// 客户端先计算文件 MD5，调用此方法判断服务端是否已存在相同文件。
/// 若存在则直接创建关联记录，返回 `file_ref_id`，无需上传。
/// 若不存在则返回 `matched: false`，客户端需走正常上传流程。
pub async fn file_upload_by_md5(
    param: &FileUploadByMd5Param,
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
    let app = super::app_check_get(param.app_id, true, &auth_data, req_dao, web_dao).await?;

    web_dao
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
    let result = web_dao
        .web_file.file_dao
        .create_from_md5(
            &param.file_md5,
            user_id,
            user_id,
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

/// 创建上传任务
pub async fn file_upload_create(
    param: &FileUploadCreateParam,
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
    let app = super::app_check_get(param.app_id, true, &auth_data, req_dao, web_dao).await?;

    web_dao
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
    let total_size: u64 = chunks.iter().map(|c| c.len).sum();
    let max_upload_size = web_dao.web_file.file_dao.runtime_setting().get_upload_max_file_size().await.unwrap_or(0);
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
        .web_file.file_dao
        .create_upload(
            user_id,
            user_id,
            app.id,
            &param.storage_type,
            &chunks,
            &param.file_name,
            &tag_refs,
            None, // expire_time
            Some(&req_dao.req_env),
        )
        .await?;

    let mut body = json!({
        "id": file_ref_id,
        "file_id": file_id,
        "file_name": param.file_name,
        "status": FileStatus::Unfinished as i8,
    });
    let file_model = web_dao.web_file.file_dao.data_dao().find_file_by_id(file_id).await?;
    if file_model.storage_type ==FileModel::STORAGE_TYPE_LOCAL_PUBLIC {
        let key = web_dao
            .web_file.file_dao
            .file_key_encoder()
            .encode(file_ref_id, None);
        body["file_key"] = json!(key);
    }
    Ok(JsonResponse::data(JsonData::body(body)))
}

/// 获取上传写句柄（通过 file_ref_id，app_id 自动从 file_ref 记录获取）
pub async fn file_upload_handle(
    file_ref_id: u64,
    chunk_index: u32,
    web_dao: &WebDao,
) -> JsonResult<FileWriteHandle> {
    let handle = web_dao
        .web_file.file_dao
        .get_upload_handle_by_file_ref_id(file_ref_id, chunk_index)
        .await?;
    Ok(handle)
}

/// 写入上传数据（代理函数，供外部组合调用，可多次调用）
pub async fn file_upload_write(
    handle: &mut FileWriteHandle,
    data: &[u8],
    web_dao: &WebDao,
) -> JsonResult<usize> {
    let written = web_dao.web_file.file_dao.write_file(handle, data).await?;
    Ok(written)
}

/// 完成上传（上传流最终步骤：成功完结）
pub async fn file_upload_complete(
    handle: FileWriteHandle,
    req_dao: &RequestDao,
    web_dao: &WebDao,
) -> JsonResult<JsonResponse> {
    web_dao
        .web_file
        .file_dao
        .complete_upload(handle, Some(&req_dao.req_env))
        .await?;
    Ok(JsonResponse::default())
}

/// 上传失败（上传流最终步骤：错误完结）
pub async fn file_upload_fail(
    handle: FileWriteHandle,
    req_dao: &RequestDao,
    web_dao: &WebDao,
) -> JsonResult<JsonResponse> {
    web_dao
        .web_file.file_dao
        .fail_upload(handle, Some(&req_dao.req_env))
        .await?;

    Ok(JsonResponse::default())
}
