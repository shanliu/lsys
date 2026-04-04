use crate::common::handler::{JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery};

use actix_multipart::Multipart;
use actix_web::post;
use futures_util::StreamExt;

use lsys_web::handler::api::user::app_file::file_upload_complete;
use lsys_web::handler::api::user::app_file::file_upload_fail;
use lsys_web::handler::api::user::app_file::file_upload_handle;
use lsys_web::handler::api::user::app_file::file_upload_write;

/// 上传文件数据（multipart 表单上传）
///
/// 表单字段：
/// - `id`: 上传任务的 file_user ID（由 upload_create 返回）
/// - `chunk_index`: 分片索引（可选，默认 0）
/// - `file`: 二进制文件数据
///
/// 流程：查找文件 → 获取句柄 → 流式写入 → 成功则 complete，失败则 fail
#[post("/upload_data")]
pub async fn file_upload_data(
    auth_dao: UserAuthQuery,
    jwt: JwtQuery,
    mut payload: Multipart,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao
        .set_request_token(&jwt)
        .await
        .map_err(|e| auth_dao.fluent_error_json_response(&e))?;

    let mut file_user_id: Option<u64> = None;
    let mut chunk_index: u32 = 0;
    let mut file_data: Vec<u8> = Vec::new();

    // 解析 multipart 表单字段
    while let Some(item) = payload.next().await {
        let mut field = item.map_err(|e| {
            auth_dao.fluent_error_json_response(&lsys_web::common::JsonError::Message(
                lsys_web::lsys_core::fluent_message!("multipart-error", e),
            ))
        })?;
        let field_name = field.name().unwrap_or("").to_string();
        match field_name.as_str() {
            "id" => {
                let mut buf = Vec::new();
                while let Some(chunk) = field.next().await {
                    let chunk = chunk.map_err(|e| {
                        auth_dao.fluent_error_json_response(&lsys_web::common::JsonError::Message(
                            lsys_web::lsys_core::fluent_message!("multipart-error", e),
                        ))
                    })?;
                    buf.extend_from_slice(&chunk);
                }
                let val = String::from_utf8_lossy(&buf);
                file_user_id = Some(val.trim().parse::<u64>().map_err(|_| {
                    auth_dao.fluent_error_json_response(&lsys_web::common::JsonError::Message(
                        lsys_web::lsys_core::fluent_message!("param-error"),
                    ))
                })?);
            }
            "chunk_index" => {
                let mut buf = Vec::new();
                while let Some(chunk) = field.next().await {
                    let chunk = chunk.map_err(|e| {
                        auth_dao.fluent_error_json_response(&lsys_web::common::JsonError::Message(
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
                        auth_dao.fluent_error_json_response(&lsys_web::common::JsonError::Message(
                            lsys_web::lsys_core::fluent_message!("multipart-error", e),
                        ))
                    })?;
                    file_data.extend_from_slice(&chunk);
                }
            }
            _ => {}
        }
    }

    let file_user_id = file_user_id.ok_or_else(|| {
        auth_dao.fluent_error_json_response(&lsys_web::common::JsonError::Message(
            lsys_web::lsys_core::fluent_message!("param-error"),
        ))
    })?;

    let result: lsys_web::common::JsonResult<lsys_web::common::JsonResponse> = async {
        let mut handle = file_upload_handle(file_user_id, chunk_index, &auth_dao).await?;

        // 写入数据，成功则 complete，失败则 fail
        match file_upload_write(&mut handle, &file_data, &auth_dao).await {
            Ok(_) => file_upload_complete(handle, &auth_dao).await,
            Err(e) => {
                let _ = file_upload_fail(handle, &auth_dao).await;
                Err(e)
            }
        }
    }
    .await;

    Ok(result
        .map_err(|e| auth_dao.fluent_error_json_response(&e))?
        .into())
}
