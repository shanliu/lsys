use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use lsys_web::handler::api::user::{
    user_address_add, user_address_delete, user_address_edit, user_address_list_data,
    AddressAddParam, AddressDeleteParam, AddressEditParam,
};

#[post("address/{method}")]
pub(crate) async fn address<'t>(
    jwt: JwtQuery,
    path: actix_web::web::Path<(String,)>,
    json_param: JsonQuery,
    auth_dao: UserAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    Ok(match path.0.to_string().as_str() {
        "add" => user_address_add(json_param.param::<AddressAddParam>()?, &auth_dao).await,
        "edit" => user_address_edit(json_param.param::<AddressEditParam>()?, &auth_dao).await,
        "list_data" => user_address_list_data(&auth_dao).await,
        "delete" => user_address_delete(json_param.param::<AddressDeleteParam>()?, &auth_dao).await,
        name => handler_not_found!(name),
    }?
    .into())
}
