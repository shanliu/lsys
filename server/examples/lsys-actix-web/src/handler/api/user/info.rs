use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use lsys_web::handler::api::user::{
    password_last_modify, user_info_check_username, user_info_set_data, user_info_set_username,
    InfoCheckUserNameParam, InfoSetUserInfoParam, InfoSetUserNameParam,
};

#[post("info/{method}")]
pub(crate) async fn set_info(
    jwt: JwtQuery,
    json_param: JsonQuery,
    auth_dao: UserAuthQuery,
    path: actix_web::web::Path<String>,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    Ok(match path.into_inner().as_str() {
        "set_username" => {
            user_info_set_username(&json_param.param::<InfoSetUserNameParam>()?, &auth_dao).await
        }
        "check_username" => {
            user_info_check_username(&json_param.param::<InfoCheckUserNameParam>()?, &auth_dao)
                .await
        }
        "password_modify" => password_last_modify(&auth_dao).await,
        "set_info" => {
            user_info_set_data(&json_param.param::<InfoSetUserInfoParam>()?, &auth_dao).await
        }
        name => handler_not_found!(name),
    }
    .map_err(|e| auth_dao.fluent_error_json_data(&e))?
    .into())
}
