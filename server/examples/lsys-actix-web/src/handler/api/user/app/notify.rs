use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};

use actix_web::post;
use lsys_web::handler::api::user::app::{
    notify_data_del, notify_data_list, NotifyDataDelParam, NotifyDataListParam,
};
#[post("/{type}")]
pub async fn notify(
    path: actix_web::web::Path<String>,
    auth_dao: UserAuthQuery,
    json_param: JsonQuery,
    jwt: JwtQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    Ok(match path.into_inner().as_str() {
        "list" => notify_data_list(&json_param.param::<NotifyDataListParam>()?, &auth_dao).await,
        "del" => notify_data_del(&json_param.param::<NotifyDataDelParam>()?, &auth_dao).await,
        name => handler_not_found!(name),
    }
    .map_err(|e| auth_dao.fluent_error_json_response(&e))?
    .into())
}
