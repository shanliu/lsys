use crate::common::handler::JsonQuery;
use crate::common::handler::{BearerQuery, ReqQuery, ResponseJson,  UserAuthQuery};

use actix_web::{HttpRequest, HttpResponse, Responder};
use actix_web::post;
use tokio::sync::mpsc::Receiver;
use tokio::time::{Duration, timeout};
use lsys_web::lsys_file::dao::FileProgressInfo;
use actix_web::web::Data;
use lsys_web::dao::WebDao;

use lsys_web::handler::api::user::app_file::{FileDownloadProgressSseParam, file_download_progress_sse};
use async_stream::stream;
/// 上传文件数据（multipart 表单上传）
///
/// 表单字段：
/// - `id`: 上传任务的 file_user ID（由 upload_create 返回）
/// - `chunk_index`: 分片索引（可选，默认 0）
/// - `file`: 二进制文件数据
///
/// 流程：查找文件 → 获取句柄 → 流式写入 → 成功则 complete，失败则 fail
#[post("/download_progress_sse")]
pub async fn file_download_progress(
   auth_dao: UserAuthQuery,
    json_param: JsonQuery,
    bearer: BearerQuery,
    req:HttpRequest,
    req_query: ReqQuery,
    web_dao: Data<WebDao>,
) -> HttpResponse {
    if let Err(e) = auth_dao.set_request_token(&bearer).await {
        return ResponseJson::from(e.to_json_response(&req_query.fluent)).respond_to(&req)
    }
    let file_ref_ids = match json_param.param::<FileDownloadProgressSseParam>(){
            Ok(e) => e,
            Err(e) =>  return ResponseJson::from(e).respond_to(&req),
        };
    let rx: Receiver<FileProgressInfo> = match file_download_progress_sse(
        &file_ref_ids,
        &req_query,
        &auth_dao,
        web_dao.as_ref(),
    )
    .await
    {
        Ok(rx) => rx,
        Err(e) => {
            return ResponseJson::from(e.to_json_response(&req_query.fluent)).respond_to(&req)
        }
    };

    let sse_stream = stream! {
        let mut rx = rx;
        loop {
            match timeout(Duration::from_secs(15), rx.recv()).await {
                Ok(Some(info)) => {
                    match serde_json::to_string(&info) {
                        Ok(json) => yield Ok::<actix_web::web::Bytes, std::io::Error>(
                            actix_web::web::Bytes::from(format!("data: {}\n\n", json))
                        ),
                        Err(_) => continue,
                    }
                }
                Ok(None) => break,
                Err(_) => {
                    yield Ok::<actix_web::web::Bytes, std::io::Error>(
                        actix_web::web::Bytes::from(": ping\n\n")
                    );
                }
            }
        }
    };

    HttpResponse::Ok()
        .content_type("text/event-stream")
        .insert_header(("Content-Encoding", "identity"))
        .insert_header(("Cache-Control", "no-cache"))
        .insert_header(("X-Accel-Buffering", "no"))
        .streaming(Box::pin(sse_stream))
}
