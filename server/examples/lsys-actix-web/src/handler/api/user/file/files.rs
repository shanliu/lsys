use crate::common::handler::{
    JsonQuery, BearerQuery, ReqQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};

use actix_web:: post;
use actix_web::web::Data;
use lsys_web::dao::WebDao;

use lsys_web::handler::api::user::app_file::file_chunks;
use lsys_web::handler::api::user::file::file_delete;
use lsys_web::handler::api::user::app_file::file_download_progress;
use lsys_web::handler::api::user::app_file::file_downloading_list;
use lsys_web::handler::api::user::app_file::file_from_url;
use lsys_web::handler::api::user::file::file_lineage_related_list;
use lsys_web::handler::api::user::file::file_list;
use lsys_web::handler::api::user::file::file_logs;
use lsys_web::handler::api::user::file::file_tag_add;
use lsys_web::handler::api::user::file::file_tag_names;
use lsys_web::handler::api::user::file::file_tag_remove;
use lsys_web::handler::api::user::file::file_tags;
use lsys_web::handler::api::user::file::file_upload_by_md5;
use lsys_web::handler::api::user::file::file_upload_create;
use lsys_web::handler::api::user::file::mapping_data;
use lsys_web::handler::api::user::app_file::FileChunksParam;
use lsys_web::handler::api::user::file::FileDeleteParam;
use lsys_web::handler::api::user::app_file::FileDownloadProgressParam;
use lsys_web::handler::api::user::app_file::FileDownloadingListParam;
use lsys_web::handler::api::user::app_file::FileFromUrlParam;
use lsys_web::handler::api::user::file::FileLineageRelatedListParam;
use lsys_web::handler::api::user::file::FileListParam;
use lsys_web::handler::api::user::file::FileLogsParam;
use lsys_web::handler::api::user::file::FileTagAddParam;
use lsys_web::handler::api::user::file::FileTagNamesParam;
use lsys_web::handler::api::user::file::FileTagRemoveParam;
use lsys_web::handler::api::user::file::FileTagsParam;
use lsys_web::handler::api::user::file::FileUploadByMd5Param;
use lsys_web::handler::api::user::file::FileUploadCreateParam;

#[post("/{type}")]
pub async fn file(
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
        "mapping" => mapping_data(&req_query, web_dao.as_ref()).await,
        "list" => {
            file_list(
                &json_param.param::<FileListParam>()?,
                &req_query,
                &auth_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "tag_names" => file_tag_names(&json_param.param::<FileTagNamesParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "tags" => file_tags(&json_param.param::<FileTagsParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "tag_add" => file_tag_add(&json_param.param::<FileTagAddParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "tag_remove" => {
            file_tag_remove(&json_param.param::<FileTagRemoveParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "upload_create" => {
            file_upload_create(&json_param.param::<FileUploadCreateParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "upload_by_md5" => {
            file_upload_by_md5(&json_param.param::<FileUploadByMd5Param>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "delete" => file_delete(&json_param.param::<FileDeleteParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "from_url" => file_from_url(&json_param.param::<FileFromUrlParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "download_progress" => {
            file_download_progress(
                &json_param.param::<FileDownloadProgressParam>()?,
                &req_query,
                &auth_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "logs" => file_logs(&json_param.param::<FileLogsParam>()?, &auth_dao, web_dao.as_ref(), &req_query).await,
        "chunks" => file_chunks(&json_param.param::<FileChunksParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "lineage_related_list" => {
            file_lineage_related_list(
                &json_param.param::<FileLineageRelatedListParam>()?,
                &req_query,
                &auth_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "downloading_list" => {
            file_downloading_list(
                &json_param.param::<FileDownloadingListParam>()?,
                &req_query,
                &auth_dao,
                web_dao.as_ref(),
            )
            .await
        }
        name => handler_not_found!(name),
    }
    .map_err(|e| req_query.fluent_error_json_response(&e))?
    .into())
}
