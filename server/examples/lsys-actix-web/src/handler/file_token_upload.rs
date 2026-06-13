//! 令牌上传端点（service 与 rest 两类场景共用同一套逻辑）
//!
//! 认证方式：仅凭 upload_token（令牌即 Redis Key，内部绑定 file_ref_id/user_id/app_id），
//! 不需要签名。客户端凭 upload_create 返回的令牌直传文件分片。
//!
//! - `/service/file/upload_by_token`：service 场景（令牌 app_id 恒为 0）
//! - `/rest/file/upload_by_token`：rest 场景（令牌 app_id 非 0，完成后触发应用回调）
//!
//! 两端点行为完全一致：底层均调用 `WebFile::finish_token_chunk_upload`，
//! 是否回调由令牌内的 app_id 决定，无需端点区分。

use actix_multipart::Multipart;
use actix_web::{post, web};
use futures_util::StreamExt;
use lsys_web::common::JsonData;
use lsys_web::dao::WebDao;

use crate::common::handler::{ReqQuery, ResponseJson, ResponseJsonResult};

/// 解析 multipart 表单，返回 (upload_token, chunk_index, file_data)
async fn parse_token_upload(
    req_dao: &ReqQuery,
    mut payload: Multipart,
) -> Result<(String, u32, Vec<u8>), ResponseJson> {
    let mut upload_token: Option<String> = None;
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

    Ok((upload_token, chunk_index, file_data))
}

/// 共用的令牌上传处理逻辑
async fn handle_token_upload(
    req_dao: &ReqQuery,
    web_dao: &WebDao,
    payload: Multipart,
) -> ResponseJsonResult<ResponseJson> {
    let (upload_token, chunk_index, file_data) = parse_token_upload(req_dao, payload).await?;

    let result = web_dao
        .web_file
        .finish_token_chunk_upload(&upload_token, chunk_index, &file_data, Some(&req_dao.req_env))
        .await
        .map_err(|e| {
            req_dao.fluent_error_json_response(&lsys_web::common::JsonError::from(e))
        })?;

    Ok(lsys_web::common::JsonResponse::data(JsonData::body(result)).into())
}

/// service 场景令牌上传端点
///
/// Multipart 字段：`upload_token`、`chunk_index`(可选,默认0)、`file`
#[post("/upload_by_token")]
pub async fn service_upload_by_token(
    req_dao: ReqQuery,
    web_dao: web::Data<WebDao>,
    payload: Multipart,
) -> ResponseJsonResult<ResponseJson> {
    handle_token_upload(&req_dao, web_dao.as_ref(), payload).await
}

/// rest 场景令牌上传端点
///
/// Multipart 字段：`upload_token`、`chunk_index`(可选,默认0)、`file`
#[post("/upload_by_token")]
pub async fn rest_upload_by_token(
    req_dao: ReqQuery,
    web_dao: web::Data<WebDao>,
    payload: Multipart,
) -> ResponseJsonResult<ResponseJson> {
    handle_token_upload(&req_dao, web_dao.as_ref(), payload).await
}
