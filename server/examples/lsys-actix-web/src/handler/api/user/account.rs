use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};

use actix_web::post;

use lsys_web::handler::api::user::account::delete;
use lsys_web::handler::api::user::account::info_check_username;
use lsys_web::handler::api::user::account::info_set_data;
use lsys_web::handler::api::user::account::info_set_username;
use lsys_web::handler::api::user::account::login_history;
use lsys_web::handler::api::user::account::password_last_modify;
use lsys_web::handler::api::user::account::DeleteParam;
use lsys_web::handler::api::user::account::InfoCheckUserNameParam;
use lsys_web::handler::api::user::account::InfoSetUserInfoParam;
use lsys_web::handler::api::user::account::InfoSetUserNameParam;
use lsys_web::handler::api::user::account::LoginHistoryParam;
use lsys_web::handler::api::user::account::{set_password, SetPasswordParam};

#[post("/{type}")]
pub async fn account(
    auth_dao: UserAuthQuery,
    path: actix_web::web::Path<String>,
    json_param: JsonQuery,
    jwt: JwtQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    Ok(match path.into_inner().as_str() {
        "login_history" => {
            login_history(&json_param.param::<LoginHistoryParam>()?, &auth_dao).await
        }
        "set_password" => set_password(&json_param.param::<SetPasswordParam>()?, &auth_dao).await,
        "delete" => delete(&json_param.param::<DeleteParam>()?, &auth_dao).await,

        "set_username" => {
            info_set_username(&json_param.param::<InfoSetUserNameParam>()?, &auth_dao).await
        }
        "check_username" => {
            info_check_username(&json_param.param::<InfoCheckUserNameParam>()?, &auth_dao).await
        }
        "password_modify" => password_last_modify(&auth_dao).await,
        "set_info" => info_set_data(&json_param.param::<InfoSetUserInfoParam>()?, &auth_dao).await,
        name => handler_not_found!(name),
    }
    .map_err(|e| auth_dao.fluent_error_json_response(&e))?
    .into())
}
