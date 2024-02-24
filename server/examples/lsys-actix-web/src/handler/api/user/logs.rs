use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use lsys_web::handler::api::user::{change_logs_list, ChangeLogsListParam};

#[post("logs/{method}")]
pub(crate) async fn user_logs<'t>(
    jwt: JwtQuery,
    json_param: JsonQuery,
    auth_dao: UserAuthQuery,
    path: actix_web::web::Path<String>,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    Ok(match path.into_inner().as_str() {
        "change" => change_logs_list(json_param.param::<ChangeLogsListParam>()?, &auth_dao).await,
        name => handler_not_found!(name),
    }?
    .into())
}
