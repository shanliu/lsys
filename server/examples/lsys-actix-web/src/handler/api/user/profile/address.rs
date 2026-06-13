use crate::common::handler::{
    JsonQuery, BearerQuery, ReqQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use actix_web::web::Data;
use lsys_web::dao::WebDao;
use lsys_web::handler::api::user::account::{
    AddressAddParam, AddressDeleteParam, AddressEditParam, address_add, address_delete,
    address_edit, address_list_data,
};

#[post("/address/{method}")]
pub(crate) async fn address(
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
        "add" => address_add(&json_param.param::<AddressAddParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "edit" => address_edit(&json_param.param::<AddressEditParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "list_data" => address_list_data(&auth_dao, web_dao.as_ref()).await,
        "delete" => address_delete(&json_param.param::<AddressDeleteParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        name => handler_not_found!(name),
    }
    .map_err(|e| req_query.fluent_error_json_response(&e))?
    .into())
}
