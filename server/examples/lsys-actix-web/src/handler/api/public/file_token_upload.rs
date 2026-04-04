//! 独立令牌上传端点
//!
//! POST /api/file/upload_by_token
//!
//! 此端点与现有用户上传端点（POST /api/user/file/upload_data）完全独立：
//! - 认证方式：upload_token（从 Redis 获取绑定身份），无需 JWT
//! - 用途：SDK 创建上传任务后，客户端凭令牌直传文件到此端点
//! - 底层共享：验证通过后使用相同的 FileDao 上传逻辑

use actix_multipart::Multipart;
use actix_web::{post, web, HttpRequest};
use futures_util::StreamExt;
use lsys_web::common::{JsonData, JsonResponse, RequestDao};
use lsys_web::dao::WebDao;
use lsys_web::lsys_core::fluents::IntoFluentMessage;
use lsys_web::lsys_core::utils::RequestEnv;
use lsys_web::lsys_files::model::FileStatus;
use serde_json::json;
use std::sync::Arc;

use crate::common::handler::{ResponseJson, ResponseJsonResult};

/// 独立令牌上传端点
///
/// Multipart 表单字段：
/// - `upload_token`: 上传令牌（由 upload_create 返回）
/// - `id`: 上传任务的 file_user ID
/// - `chunk_index`: 分片索引（可选，默认 0）
/// - `file`: 二进制文件数据
#[post("/upload_by_token")]
pub async fn upload_by_token(
    req: HttpRequest,
    web_dao: web::Data<Arc<WebDao>>,
    mut payload: Multipart,
) -> ResponseJsonResult<ResponseJson> {
    // 构建 RequestDao（无需用户认证会话）
    let user_lang = req
        .headers()
        .get("Accept-Language")
        .and_then(|t| t.to_str().map(|s| s.split(',').next().unwrap_or(s)).ok())
        .unwrap_or("zh_CN")
        .replace('-', "_");
    let user_agent = req
        .headers()
        .get("User-Agent")
        .and_then(|e| e.to_str().ok());
    let request_id = req
        .headers()
        .get("X-Request-ID")
        .and_then(|e| e.to_str().ok());
    let device_id = req
        .headers()
        .get("X-Device-ID")
        .and_then(|e| e.to_str().ok());
    let ip: Option<String> = {
        let conn = req.connection_info();
        conn.realip_remote_addr().map(str::to_owned)
        // conn (Ref<ConnectionInfo>) is dropped here, before any await
    };
    let req_env = RequestEnv::new(
        Some(&user_lang),
        ip.as_deref(),
        request_id,
        user_agent,
        device_id,
    )
    .map_err(|verr| {
        JsonResponse::data(
            JsonData::default()
                .set_sub_code("env_valid_err")
                .set_code(400),
        )
        .set_message(verr.to_fluent_message().default_format())
    })?;
    let req_dao = RequestDao::new(web_dao.get_ref().clone(), req_env);

    // 解析 multipart
    let mut upload_token: Option<String> = None;
    let mut file_user_id: Option<u64> = None;
    let mut chunk_index: u32 = 0;
    let mut file_data: Vec<u8> = Vec::new();

    while let Some(item) = payload.next().await {
        let mut field = item.map_err(|e| {
            req_dao.fluent_error_json_response(&lsys_web::common::JsonError::Message(
                lsys_web::lsys_core::fluent_message!("multipart-error", e),
            ))
        })?;
        let field_name = field.name().unwrap_or("").to_string();
        match field_name.as_str() {
            "upload_token" => {
                let mut buf = Vec::new();
                while let Some(chunk) = field.next().await {
                    let chunk = chunk.map_err(|e| {
                        req_dao.fluent_error_json_response(&lsys_web::common::JsonError::Message(
                            lsys_web::lsys_core::fluent_message!("multipart-error", e),
                        ))
                    })?;
                    buf.extend_from_slice(&chunk);
                }
                upload_token = Some(String::from_utf8_lossy(&buf).trim().to_string());
            }
            "id" => {
                let mut buf = Vec::new();
                while let Some(chunk) = field.next().await {
                    let chunk = chunk.map_err(|e| {
                        req_dao.fluent_error_json_response(&lsys_web::common::JsonError::Message(
                            lsys_web::lsys_core::fluent_message!("multipart-error", e),
                        ))
                    })?;
                    buf.extend_from_slice(&chunk);
                }
                let val = String::from_utf8_lossy(&buf);
                file_user_id = Some(val.trim().parse::<u64>().map_err(|_| {
                    req_dao.fluent_error_json_response(&lsys_web::common::JsonError::Message(
                        lsys_web::lsys_core::fluent_message!("param-error"),
                    ))
                })?);
            }
            "chunk_index" => {
                let mut buf = Vec::new();
                while let Some(chunk) = field.next().await {
                    let chunk = chunk.map_err(|e| {
                        req_dao.fluent_error_json_response(&lsys_web::common::JsonError::Message(
                            lsys_web::lsys_core::fluent_message!("multipart-error", e),
                        ))
                    })?;
                    buf.extend_from_slice(&chunk);
                }
                let val = String::from_utf8_lossy(&buf);
                chunk_index = val.trim().parse::<u32>().unwrap_or(0);
            }
            "file" => {
                while let Some(chunk) = field.next().await {
                    let chunk = chunk.map_err(|e| {
                        req_dao.fluent_error_json_response(&lsys_web::common::JsonError::Message(
                            lsys_web::lsys_core::fluent_message!("multipart-error", e),
                        ))
                    })?;
                    file_data.extend_from_slice(&chunk);
                }
            }
            _ => {}
        }
    }

    let upload_token = upload_token.ok_or_else(|| {
        req_dao.fluent_error_json_response(&lsys_web::common::JsonError::Message(
            lsys_web::lsys_core::fluent_message!("upload-token-required"),
        ))
    })?;

    let file_user_id = file_user_id.ok_or_else(|| {
        req_dao.fluent_error_json_response(&lsys_web::common::JsonError::Message(
            lsys_web::lsys_core::fluent_message!("param-error"),
        ))
    })?;

    // 验证令牌（含自动续期）
    let _verify_result = req_dao
        .web_dao
        .web_files
        .upload_token
        .verify_upload_token(&upload_token, file_user_id)
        .await
        .map_err(|e| {
            req_dao.fluent_error_json_response(&lsys_web::common::JsonError::Message(
                e.to_fluent_message(),
            ))
        })?;

    // 执行上传逻辑
    let result: lsys_web::common::JsonResult<JsonResponse> = async {
        // 通过 file_user_id 获取上传句柄 (内部会查找 file_user → file，并从 file_user 中获取 app_id)
        let mut handle = req_dao
            .web_dao
            .web_files
            .file_dao
            .get_upload_handle_by_file_user_id(file_user_id, chunk_index)
            .await?;

        let file_id = handle.file.id;

        // 写入数据
        match req_dao
            .web_dao
            .web_files
            .file_dao
            .write_file(&mut handle, &file_data)
            .await
        {
            Ok(_) => {
                // complete_upload
                let completed_file = req_dao
                    .web_dao
                    .web_files
                    .file_dao
                    .complete_upload(handle, Some(&req_dao.req_env))
                    .await?;

                // 判断是否销毁令牌：整个文件 Normal 时销毁
                if completed_file.status == FileStatus::Normal as i8 {
                    let _ = req_dao
                        .web_dao
                        .web_files
                        .upload_token
                        .consume_upload_token(&upload_token)
                        .await;
                }

                Ok(JsonResponse::data(JsonData::body(json!({
                    "id": file_user_id,
                    "file_id": completed_file.id,
                    "chunk_index": chunk_index,
                    "file_status": completed_file.status,
                    "file_md5": completed_file.file_md5,
                    "file_name": completed_file.file_name,
                    "file_size": completed_file.file_size,
                }))))
            }
            Err(e) => {
                // fail_upload
                // 获取 file_local 来判断是否为单分片文件
                let file_local = req_dao
                    .web_dao
                    .web_files
                    .file_dao
                    .helper()
                    .find_file_local_by_file_id(file_id)
                    .await
                    .ok()
                    .flatten();

                let _ = req_dao
                    .web_dao
                    .web_files
                    .file_dao
                    .fail_upload(handle, Some(&req_dao.req_env))
                    .await;

                // 单分片文件 fail 时销毁令牌
                if let Some(local) = file_local
                    && local.file_chunk_total <= 1 {
                        let _ = req_dao
                            .web_dao
                            .web_files
                            .upload_token
                            .consume_upload_token(&upload_token)
                            .await;
                    }
                Err(e.into())
            }
        }
    }
    .await;

    Ok(result
        .map_err(|e| req_dao.fluent_error_json_response(&e))?
        .into())
}
