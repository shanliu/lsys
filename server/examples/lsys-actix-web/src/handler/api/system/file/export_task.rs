use crate::common::handler::{
    BearerQuery, JsonQuery, ReqQuery, ResponseJson, ResponseJsonResult, UserAuthQuery, parse_token,
};
use crate::common::util::handler::{create_download_response, parse_range_header};

use actix_web::{post, HttpRequest, HttpResponse};
use actix_web::web::Data;
use lsys_web::dao::WebDao;

use lsys_web::handler::api::system::export_task::AdminExportActiveCountParam;
use lsys_web::handler::api::system::export_task::AdminExportDownloadParam;
use lsys_web::handler::api::system::export_task::AdminExportListParam;
use lsys_web::handler::api::system::export_task::AdminExportSubmitParam;
use lsys_web::handler::api::system::export_task::admin_export_active_count;
use lsys_web::handler::api::system::export_task::admin_export_download;
use lsys_web::handler::api::system::export_task::admin_export_list;
use lsys_web::handler::api::system::export_task::admin_export_submit;
use lsys_web::handler::api::system::export_task::admin_export_task_mapping;

#[post("/export_task/{type}")]
pub async fn export_task(
    auth_dao: UserAuthQuery,
    path: actix_web::web::Path<String>,
    json_param: JsonQuery,
    bearer: BearerQuery,
    req_query: ReqQuery,
    web_dao: Data<WebDao>,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao
        .set_request_token(&bearer)
        .await
        .map_err(|e| req_query.fluent_error_json_response(&e))?;
    Ok(match path.into_inner().as_str() {
        "mapping" => admin_export_task_mapping(&req_query).await,
        "active_count" => {
            admin_export_active_count(
                &json_param.param::<AdminExportActiveCountParam>()?,
                &req_query,
                &auth_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "submit" => {
            admin_export_submit(&json_param.param::<AdminExportSubmitParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "list" => {
            admin_export_list(&json_param.param::<AdminExportListParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        name => handler_not_found!(name),
    }
    .map_err(|e| req_query.fluent_error_json_response(&e))?
    .into())
}

#[derive(serde::Deserialize)]
struct DownloadRequest {
    token: String,
    task_id: u64,
}

/// 系统端导出任务文件下载接口
///
/// 支持 HTTP Range 请求实现断点续传
#[post("/export_download")]
pub async fn export_task_download(
    auth_dao: UserAuthQuery,
    form: actix_web::web::Form<DownloadRequest>,
    req: HttpRequest,
    req_query: ReqQuery,
    web_dao: Data<WebDao>,
) -> ResponseJsonResult<HttpResponse> {
    let down_req = form.into_inner();
    let token = parse_token(&req, down_req.token)
        .await
        .map_err(|e| req_query.fluent_error_json_response(&e.into_json_error()))?;
    let param = AdminExportDownloadParam { task_id: down_req.task_id };
    auth_dao
        .set_request_token(&token)
        .await
        .map_err(|e| req_query.fluent_error_json_response(&e))?;

    // 解析 Range 请求头
    let offset =parse_range_header(&req).unwrap_or(0);

    // 调用下载函数
    let response = admin_export_download(param, offset, &req_query, &auth_dao, web_dao.as_ref())
        .await
        .map_err(|e| req_query.fluent_error_json_response(&e))?;

    // 创建流式响应
    let mut iter = response.iter;
    let file_size = response.file_size;
    let file_name = response.file_name;
    let content_type = response.content_type;

    let stream = async_stream::stream! {
        while let Some(result) = iter.next_chunk().await {
            match result {
                Ok(chunk) => yield Ok::<_, std::io::Error>(actix_web::web::Bytes::from(chunk.data)),
                Err(e) => {
                    yield Err(std::io::Error::other(req_query.fluent_error_string(&e.into())));
                    break;
                }
            }
        }
    };

    Ok(create_download_response(
        file_name,
        content_type,
        file_size,
        stream,
        offset,
    ))
}
