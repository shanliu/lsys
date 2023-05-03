use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use lsys_web::handler::api::user::{
    user_mobile_add, user_mobile_confirm, user_mobile_delete, user_mobile_list_data,
    user_mobile_send_code, MobileAddParam, MobileConfirmParam, MobileDeleteParam,
    MobileListDataParam, MobileSendCodeParam,
};

#[post("mobile/{method}")]
pub(crate) async fn mobile<'t>(
    jwt: JwtQuery,
    path: actix_web::web::Path<(String,)>,
    json_param: JsonQuery,
    auth_dao: UserAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    Ok(match path.0.to_string().as_str() {
        "add" => user_mobile_add(json_param.param::<MobileAddParam>()?, &auth_dao).await,
        "send_code" => {
            user_mobile_send_code(json_param.param::<MobileSendCodeParam>()?, &auth_dao).await
        }
        "delete" => user_mobile_delete(json_param.param::<MobileDeleteParam>()?, &auth_dao).await,
        "confirm" => {
            user_mobile_confirm(json_param.param::<MobileConfirmParam>()?, &auth_dao).await
        }
        "list_data" => {
            user_mobile_list_data(json_param.param::<MobileListDataParam>()?, &auth_dao).await
        }
        name => handler_not_found!(name),
    }?
    .into())
}
