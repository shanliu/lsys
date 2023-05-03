use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use lsys_web::handler::api::user::{
    user_email_add, user_email_confirm, user_email_delete, user_email_list_data,
    user_email_send_code, EmailAddParam, EmailConfirmParam, EmailDeleteParam, EmailListDataParam,
    EmailSendCodeParam,
};

#[post("email_confirm")]
pub(crate) async fn email_confirm<'t>(
    json_param: JsonQuery,
    auth_dao: UserAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    Ok(
        user_email_confirm(json_param.param::<EmailConfirmParam>()?, &auth_dao)
            .await?
            .into(),
    )
}

#[post("email/{method}")]
pub(crate) async fn email<'t>(
    jwt: JwtQuery,
    path: actix_web::web::Path<(String,)>,
    json_param: JsonQuery,
    auth_dao: UserAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    Ok(match path.0.to_string().as_str() {
        "add" => user_email_add(json_param.param::<EmailAddParam>()?, &auth_dao).await,
        "send_code" => {
            user_email_send_code(json_param.param::<EmailSendCodeParam>()?, &auth_dao).await
        }
        "delete" => user_email_delete(json_param.param::<EmailDeleteParam>()?, &auth_dao).await,
        "list_data" => {
            user_email_list_data(json_param.param::<EmailListDataParam>()?, &auth_dao).await
        }
        name => handler_not_found!(name),
    }?
    .into())
}
