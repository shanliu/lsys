use crate::common::handler::{
    BearerQuery, JsonQuery, ReqQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use actix_web::web::Data;
use lsys_web::dao::WebDao;
use lsys_web::handler::api::auth::{RbacAccessMenuParam, perm_check_list};

#[post("/perm/{method}")]
pub(crate) async fn perm(
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
        "check" => perm_check_list(&json_param.param::<RbacAccessMenuParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        name => handler_not_found!(name),
    }
    .map_err(|e| req_query.fluent_error_json_response(&e))?
    .into())
}
