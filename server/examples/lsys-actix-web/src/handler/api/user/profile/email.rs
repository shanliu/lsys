use crate::common::handler::{
    BearerQuery, JsonQuery, ReqQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use lsys_web::dao::WebDao;
use lsys_web::handler::api::user::account::{
    EmailAddParam, EmailConfirmParam, EmailDeleteParam, EmailListDataParam, EmailSendCodeParam,
    email_add, email_confirm, email_delete, email_list_data, email_send_code,
};

#[post("/email/{method}")]
pub(crate) async fn email(
    bearer: BearerQuery,
    path: actix_web::web::Path<String>,
    json_param: JsonQuery,
    auth_dao: UserAuthQuery,
    req_query: ReqQuery,
    web_dao: actix_web::web::Data<WebDao>,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao
        .set_request_token(&bearer)
        .await
        .map_err(|e| req_query.fluent_error_json_response(&e))?;
    Ok(match path.into_inner().as_str() {
        "add" => email_add(&json_param.param::<EmailAddParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "send_code" => email_send_code(&json_param.param::<EmailSendCodeParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "delete" => email_delete(&json_param.param::<EmailDeleteParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "confirm" => email_confirm(&json_param.param::<EmailConfirmParam>()?, &req_query, web_dao.as_ref()).await,
        "list_data" => email_list_data(&json_param.param::<EmailListDataParam>()?, &auth_dao, web_dao.as_ref()).await,
        name => handler_not_found!(name),
    }
    .map_err(|e| req_query.fluent_error_json_response(&e))?
    .into())
}
