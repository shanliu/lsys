use crate::common::handler::{
    BearerQuery, JsonQuery, ReqQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use actix_web::web::Data;
use lsys_web::dao::WebDao;
use lsys_web::handler::rest::oauth::{AuthorizeDoParam, ScopeGetParam, create_code, scope_get};

#[post("/{method}")]
pub(crate) async fn oauth(
    bearer: BearerQuery,
    path: actix_web::web::Path<String>,
    json_param: JsonQuery,
    auth_dao: UserAuthQuery,
    req_query: ReqQuery,
    web_dao: Data<WebDao>,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao
        .set_request_token(&bearer)
        .await
        .map_err(|e| req_query.fluent_error_json_response(&e))?;
    Ok(match path.into_inner().as_str() {
        "scope" => scope_get(&json_param.param::<ScopeGetParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "do" => create_code(&json_param.param::<AuthorizeDoParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        name => handler_not_found!(name),
    }
    .map_err(|e| req_query.fluent_error_json_response(&e))?
    .into())
}
