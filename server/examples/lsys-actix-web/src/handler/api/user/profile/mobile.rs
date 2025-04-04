use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use lsys_web::handler::api::user::account::{
    mobile_add, mobile_confirm, mobile_delete, mobile_list_data, mobile_send_code, MobileAddParam,
    MobileConfirmParam, MobileDeleteParam, MobileListDataParam, MobileSendCodeParam,
};

#[post("/mobile/{method}")]
pub(crate) async fn mobile(
    jwt: JwtQuery,
    path: actix_web::web::Path<String>,
    json_param: JsonQuery,
    auth_dao: UserAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    Ok(match path.into_inner().as_str() {
        "add" => mobile_add(&json_param.param::<MobileAddParam>()?, &auth_dao).await,
        "send_code" => {
            mobile_send_code(&json_param.param::<MobileSendCodeParam>()?, &auth_dao).await
        }
        "delete" => mobile_delete(&json_param.param::<MobileDeleteParam>()?, &auth_dao).await,
        "confirm" => mobile_confirm(&json_param.param::<MobileConfirmParam>()?, &auth_dao).await,
        "list_data" => {
            mobile_list_data(&json_param.param::<MobileListDataParam>()?, &auth_dao).await
        }
        name => handler_not_found!(name),
    }
    .map_err(|e| auth_dao.fluent_error_json_data(&e))?
    .into())
}
