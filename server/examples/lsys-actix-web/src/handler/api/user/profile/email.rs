use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use lsys_web::handler::api::user::account::{
    email_add, email_confirm, email_delete, email_list_data, email_send_code, EmailAddParam,
    EmailConfirmParam, EmailDeleteParam, EmailListDataParam, EmailSendCodeParam,
};

#[post("/email/{method}")]
pub(crate) async fn email(
    jwt: JwtQuery,
    path: actix_web::web::Path<String>,
    json_param: JsonQuery,
    auth_dao: UserAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    Ok(match path.into_inner().as_str() {
        "add" => email_add(&json_param.param::<EmailAddParam>()?, &auth_dao).await,
        "send_code" => email_send_code(&json_param.param::<EmailSendCodeParam>()?, &auth_dao).await,
        "delete" => email_delete(&json_param.param::<EmailDeleteParam>()?, &auth_dao).await,
        "confirm" => email_confirm(&json_param.param::<EmailConfirmParam>()?, &auth_dao).await,
        "list_data" => email_list_data(&json_param.param::<EmailListDataParam>()?, &auth_dao).await,
        name => handler_not_found!(name),
    }
    .map_err(|e| auth_dao.fluent_error_json_response(&e))?
    .into())
}
