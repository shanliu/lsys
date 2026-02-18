use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};

use actix_web::post;

use lsys_web::handler::api::system::file::admin_file_delete;
use lsys_web::handler::api::system::file::admin_file_list;
use lsys_web::handler::api::system::file::AdminFileDeleteParam;
use lsys_web::handler::api::system::file::AdminFileListParam;

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
        "list" => admin_file_list(&json_param.param::<AdminFileListParam>()?, &auth_dao).await,
        "delete" => {
            admin_file_delete(&json_param.param::<AdminFileDeleteParam>()?, &auth_dao).await
        }
        name => handler_not_found!(name),
    }
    .map_err(|e| auth_dao.fluent_error_json_response(&e))?
    .into())
}
