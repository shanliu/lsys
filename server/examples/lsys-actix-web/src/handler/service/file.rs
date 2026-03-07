use actix_web::post;
use lsys_web::handler::service::file::{
    self as service_file, FileDeleteParam, FileInfoParam, FileListParam, FileUrlsParam,
    FromLocalParam, FromUrlParam, UploadByMd5Param, UploadCreateParam, UploadRetokenParam,
};

use crate::common::handler::{JsonQuery, ResponseJson, ResponseJsonResult, ServiceQuery};

/// File service endpoints
///
/// POST /service/file/{method}
///
/// Headers:
/// - X-Timestamp: required
/// - X-Signature: required
///
/// Methods:
/// - upload_create: Create upload task + issue token
/// - upload_retoken: Re-issue token for unfinished file
/// - upload_by_md5: Instant upload by MD5
/// - from_url: Create file from URL (sync/async)
/// - from_local: Import from local file on shared disk
/// - list: Query file list
/// - delete: Delete file
/// - urls: Batch get file URLs
/// - info: Get file details by file_user_id
/// - mapping: Get file config mapping
#[post("/file/{method}")]
pub async fn file(
    service: ServiceQuery,
    path: actix_web::web::Path<String>,
    json_param: JsonQuery,
) -> ResponseJsonResult<ResponseJson> {
    let method = path.into_inner();
    let result = match method.as_str() {
        "upload_create" => {
            service_file::upload_create(
                &json_param.param::<UploadCreateParam>()?,
                &service.inner,
            )
            .await
        }
        "upload_retoken" => {
            service_file::upload_retoken(
                &json_param.param::<UploadRetokenParam>()?,
                &service.inner,
            )
            .await
        }
        "upload_by_md5" => {
            service_file::upload_by_md5(
                &json_param.param::<UploadByMd5Param>()?,
                &service.inner,
            )
            .await
        }
        "from_url" => {
            service_file::from_url(&json_param.param::<FromUrlParam>()?, &service.inner).await
        }
        "from_local" => {
            service_file::from_local(&json_param.param::<FromLocalParam>()?, &service.inner).await
        }
        "list" => {
            service_file::file_list(&json_param.param::<FileListParam>()?, &service.inner).await
        }
        "delete" => {
            service_file::file_delete(&json_param.param::<FileDeleteParam>()?, &service.inner)
                .await
        }
        "urls" => {
            service_file::file_urls(&json_param.param::<FileUrlsParam>()?, &service.inner).await
        }
        "info" => {
            service_file::file_info(&json_param.param::<FileInfoParam>()?, &service.inner).await
        }
        "mapping" => service_file::mapping(&(), &service.inner).await,
        _ => handler_not_found!(method),
    };
    result
        .map(|r| r.into())
        .map_err(|e| service.fluent_error_json_response(&e).into())
}
