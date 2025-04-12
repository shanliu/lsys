use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use lsys_web::handler::api::user::account::{
    address_add, address_delete, address_edit, address_list_data, AddressAddParam,
    AddressDeleteParam, AddressEditParam,
};

#[post("/address/{method}")]
pub(crate) async fn address(
    jwt: JwtQuery,
    path: actix_web::web::Path<String>,
    json_param: JsonQuery,
    auth_dao: UserAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    Ok(match path.into_inner().as_str() {
        "add" => address_add(&json_param.param::<AddressAddParam>()?, &auth_dao).await,
        "edit" => address_edit(&json_param.param::<AddressEditParam>()?, &auth_dao).await,
        "list_data" => address_list_data(&auth_dao).await,
        "delete" => address_delete(&json_param.param::<AddressDeleteParam>()?, &auth_dao).await,
        name => handler_not_found!(name),
    }
    .map_err(|e| auth_dao.fluent_error_json_response(&e))?
    .into())
}
