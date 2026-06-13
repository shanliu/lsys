use crate::common::handler::{ReqQuery, ResponseJson, ResponseJsonResult, RestQuery};
use actix_web::{post, web};
use lsys_web::dao::WebDao;
use lsys_web::handler::rest::file::{
    FileDeleteParam, FileInfoParam, FileListParam, FileUrlsParam, FromUrlParam, UploadByMd5Param,
    UploadCreateParam, UploadRetokenParam, file_delete, file_info, file_list, file_urls, from_url,
    mapping, upload_by_md5, upload_create, upload_retoken,
};

/// REST 文件接口分发
///
/// POST /rest/file  (method 由 RFC 元数据指定)
///
/// Methods:
/// - upload_create / upload_retoken / upload_by_md5 / from_url
/// - list / delete / urls / info / mapping
///
/// 令牌直传分片走独立端点 /rest/file/upload_by_token
#[post("")]
pub(crate) async fn file(
    rest: RestQuery,
    req_dao: ReqQuery,
    web_dao: web::Data<WebDao>,
) -> ResponseJsonResult<ResponseJson> {
    Ok(match rest.rfc.method.as_deref().unwrap_or_default() {
        "upload_create" => {
            let param = rest.param::<UploadCreateParam>()?;
            upload_create(&param, &rest.get_app().await?, &req_dao, web_dao.as_ref()).await
        }
        "upload_retoken" => {
            let param = rest.param::<UploadRetokenParam>()?;
            upload_retoken(&param, &rest.get_app().await?, &req_dao, web_dao.as_ref()).await
        }
        "upload_by_md5" => {
            let param = rest.param::<UploadByMd5Param>()?;
            upload_by_md5(&param, &rest.get_app().await?, &req_dao, web_dao.as_ref()).await
        }
        "from_url" => {
            let param = rest.param::<FromUrlParam>()?;
            from_url(&param, &rest.get_app().await?, &req_dao, web_dao.as_ref()).await
        }
        "list" => {
            let param = rest.param::<FileListParam>()?;
            file_list(&param, &rest.get_app().await?, &req_dao, web_dao.as_ref()).await
        }
        "delete" => {
            let param = rest.param::<FileDeleteParam>()?;
            file_delete(&param, &rest.get_app().await?, &req_dao, web_dao.as_ref()).await
        }
        "urls" => {
            let param = rest.param::<FileUrlsParam>()?;
            file_urls(&param, &rest.get_app().await?, &req_dao, web_dao.as_ref()).await
        }
        "info" => {
            let param = rest.param::<FileInfoParam>()?;
            file_info(&param, &rest.get_app().await?, &req_dao, web_dao.as_ref()).await
        }
        "mapping" => mapping(&rest.get_app().await?, &req_dao, web_dao.as_ref()).await,
        var => handler_not_found!(var),
    }
    .map_err(|e| req_dao.fluent_error_json_response(&e))?
    .into())
}
