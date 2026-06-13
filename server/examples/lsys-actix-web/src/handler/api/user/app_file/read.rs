use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;
use std::ops::Deref;

use crate::common::handler::{ReqQuery, ResponseJson, UserAuthQuery, parse_token};
use crate::common::util::handler::{create_file_stream_response, parse_range_header};

#[derive(Debug, Deserialize)]
pub struct AppFileAccessPathParam {
    pub key: String,
    /// 登录 token（用于无法设置 header 的下载场景，如浏览器直接打开）
    pub token: String,
}

/// 用户通过 key 访问应用文件
///
/// 逻辑：
/// - 公开文件：重定向到文件 URL（302）
/// - 私有文件：验证权限后流式输出文件内容
///
/// 支持 HTTP Range 请求头进行断点续传
#[post("/read", name = "app_file_access")]
pub async fn app_file_access(
    auth_dao: UserAuthQuery,
    req_query: ReqQuery,
    form: web::Form<AppFileAccessPathParam>,
    req: HttpRequest,
    web_dao: web::Data<lsys_web::dao::WebDao>,
) -> HttpResponse {
    let offset = parse_range_header(&req).unwrap_or(0);

    let param = form.into_inner();

    // 调用 lsys-web 的文件访问函数
    let result = match lsys_web::handler::api::user::app_file::app_file_access(
        &lsys_web::handler::api::user::app_file::AppFileAccessParam {
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
        lsys_web::handler::api::user::app_file::AppFileAccessData::Url(url) => {
            // 公开文件：重定向
            HttpResponse::Found()
                .insert_header(("Location", url.as_str()))
                .finish()
        }
        lsys_web::handler::api::user::app_file::AppFileAccessData::Stream {
            ref_model,
            file_model,
            stream,
        } => {
            // 私有文件：流式输出
            create_file_stream_response(*ref_model, *file_model, stream, offset)
        }
    }
}

/// GET 方式访问公开文件分享（无需认证）
///
/// 逻辑：
/// - 公开文件：重定向到文件 URL（302）
/// - 私有文件：返回纯文本"无权限访问"（403）
#[get("/share/{key}", name = "app_file_access_public")]
pub async fn app_file_access_public(
    path: web::Path<String>,
    req_query: ReqQuery,
    web_dao: web::Data<lsys_web::dao::WebDao>,
) -> HttpResponse {
    let key = path.into_inner();

    match lsys_web::handler::api::user::app_file::app_file_share(&key, web_dao.as_ref()).await {
        Ok(url) => HttpResponse::Found()
            .insert_header(("Location", url.as_str()))
            .finish(),
        Err(e) => HttpResponse::Forbidden()
            .content_type("text/plain; charset=utf-8")
            .body(req_query.fluent_error_string(&e)),
    }
}
