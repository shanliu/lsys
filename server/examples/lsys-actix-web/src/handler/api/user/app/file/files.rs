use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};

use actix_web::post;

use lsys_web::handler::api::user::app_file::FileChunksParam;
use lsys_web::handler::api::user::app_file::FileDeleteParam;
use lsys_web::handler::api::user::app_file::FileFromUrlParam;
use lsys_web::handler::api::user::app_file::FileListParam;
use lsys_web::handler::api::user::app_file::FileLogsParam;
use lsys_web::handler::api::user::app_file::FileTagAddParam;
use lsys_web::handler::api::user::app_file::FileTagNamesParam;
use lsys_web::handler::api::user::app_file::FileTagRemoveParam;
use lsys_web::handler::api::user::app_file::FileTagsParam;
use lsys_web::handler::api::user::app_file::FileUploadByMd5Param;
use lsys_web::handler::api::user::app_file::FileUploadCreateParam;
use lsys_web::handler::api::user::app_file::file_chunks;
use lsys_web::handler::api::user::app_file::file_delete;
use lsys_web::handler::api::user::app_file::file_from_url;
use lsys_web::handler::api::user::app_file::file_list;
use lsys_web::handler::api::user::app_file::file_logs;
use lsys_web::handler::api::user::app_file::file_tag_add;
use lsys_web::handler::api::user::app_file::file_tag_names;
use lsys_web::handler::api::user::app_file::file_tag_remove;
use lsys_web::handler::api::user::app_file::file_tags;
use lsys_web::handler::api::user::app_file::file_upload_by_md5;
use lsys_web::handler::api::user::app_file::file_upload_create;
use lsys_web::handler::api::user::app_file::mapping_data;

#[post("/{type}")]
pub async fn file(
    auth_dao: UserAuthQuery,
    path: actix_web::web::Path<String>,
    json_param: JsonQuery,
    jwt: JwtQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao
        .set_request_token(&jwt)
        .await
        .map_err(|e| auth_dao.fluent_error_json_response(&e))?;
    Ok(match path.into_inner().as_str() {
        "mapping" => mapping_data(&auth_dao).await,
        "list" => file_list(&json_param.param::<FileListParam>()?, &auth_dao).await,
        "tag_names" => file_tag_names(&json_param.param::<FileTagNamesParam>()?, &auth_dao).await,
        "tags" => file_tags(&json_param.param::<FileTagsParam>()?, &auth_dao).await,
        "tag_add" => file_tag_add(&json_param.param::<FileTagAddParam>()?, &auth_dao).await,
        "tag_remove" => {
            file_tag_remove(&json_param.param::<FileTagRemoveParam>()?, &auth_dao).await
        }
        "upload_create" => {
            file_upload_create(&json_param.param::<FileUploadCreateParam>()?, &auth_dao).await
        }
        "upload_by_md5" => {
            file_upload_by_md5(&json_param.param::<FileUploadByMd5Param>()?, &auth_dao).await
        }
        "delete" => file_delete(&json_param.param::<FileDeleteParam>()?, &auth_dao).await,
        "from_url" => file_from_url(&json_param.param::<FileFromUrlParam>()?, &auth_dao).await,
        "logs" => file_logs(&json_param.param::<FileLogsParam>()?, &auth_dao).await,
        "chunks" => file_chunks(&json_param.param::<FileChunksParam>()?, &auth_dao).await,
        name => handler_not_found!(name),
    }
    .map_err(|e| auth_dao.fluent_error_json_response(&e))?
    .into())
}
