use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;
use std::ops::Deref;

use crate::common::handler::{ReqQuery, ResponseJson, UserAuthQuery, parse_token};
use crate::common::util::handler::{create_file_stream_response, parse_range_header};

#[derive(Debug, Deserialize)]
pub struct AdminFileAccessPathParam {
    pub key: String,
    /// 登录 token（用于无法设置 header 的下载场景，如浏览器直接打开）
    pub token: String,
}

/// 管理员通过 key 访问文件
///
/// 逻辑：
/// - 公开文件：重定向到文件 URL（302）
/// - 私有文件：验证管理员权限后流式输出文件内容
///
/// 支持 HTTP Range 请求头进行断点续传
#[post("/read", name = "admin_file_access")]
pub async fn admin_file_access(
    auth_dao: UserAuthQuery,
    req_query: ReqQuery,
    form: web::Form<AdminFileAccessPathParam>,
    req: HttpRequest,
    web_dao: web::Data<lsys_web::dao::WebDao>,
) -> HttpResponse {
    let offset = parse_range_header(&req).unwrap_or(0);

    let param = form.into_inner();

    // 调用 lsys-web 的文件访问函数
    let result = match lsys_web::handler::api::system::file::admin_file_access(
        &lsys_web::handler::api::system::file::AdminFileAccessParam {
            key: param.key,
        },
        offset,
        &req_query,
        || async {
            let token = parse_token(&req, param.token.clone())
                .await
                .map_err(|e| e.into_json_error())?;
            auth_dao.set_request_token(&token).await?;
            Ok(auth_dao.deref())
        },
        web_dao.as_ref(),
    )
    .await
    {
        Ok(result) => result,
        Err(e) => {
            return ResponseJson::from(req_query.fluent_error_json_response(&e)).respond_to(&req);
        }
    };

    match result {
        lsys_web::handler::api::system::file::AdminFileAccessData::Url(url) => {
            // 公开文件：重定向
            HttpResponse::Found()
                .insert_header(("Location", url.as_str()))
                .finish()
        }
        lsys_web::handler::api::system::file::AdminFileAccessData::Stream {
            ref_model,
            file_model,
            stream,
        } => {
            // 私有文件：流式输出
            create_file_stream_response(*ref_model, *file_model, stream, offset)
        }
    }
}
