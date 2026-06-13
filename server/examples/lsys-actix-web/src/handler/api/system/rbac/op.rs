use crate::common::handler::{
    BearerQuery, JsonQuery, ReqQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use actix_web::web::Data;
use lsys_web::dao::WebDao;
use lsys_web::handler::api::system::rbac::{
    OpAddParam, OpDataParam, OpDelParam, OpEditParam, op_add, op_data, op_del, op_edit,
};

#[post("/op/{method}")]
pub async fn op(
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
    let data = match path.into_inner().as_str() {
        "add" => op_add(&json_param.param::<OpAddParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "edit" => op_edit(&json_param.param::<OpEditParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "delete" => op_del(&json_param.param::<OpDelParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "list" => op_data(&json_param.param::<OpDataParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        name => handler_not_found!(name),
    };
    Ok(data
        .map_err(|e| req_query.fluent_error_json_response(&e))?
        .into())
}
