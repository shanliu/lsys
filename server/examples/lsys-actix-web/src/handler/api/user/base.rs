use crate::common::handler::{
    JsonQuery, BearerQuery, ReqQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};

use actix_web::post;
use actix_web::web::Data;
use lsys_web::dao::WebDao;

use lsys_web::handler::api::user::account::DeleteParam;
use lsys_web::handler::api::user::account::InfoCheckUserNameParam;
use lsys_web::handler::api::user::account::InfoSetUserInfoParam;
use lsys_web::handler::api::user::account::InfoSetUserNameParam;
use lsys_web::handler::api::user::account::LoginHistoryParam;
use lsys_web::handler::api::user::account::delete;
use lsys_web::handler::api::user::account::info_check_username;
use lsys_web::handler::api::user::account::info_set_data;
use lsys_web::handler::api::user::account::info_set_username;
use lsys_web::handler::api::user::account::login_history;
use lsys_web::handler::api::user::account::mapping_data;
use lsys_web::handler::api::user::account::password_last_modify;
use lsys_web::handler::api::user::account::{SetPasswordParam, set_password};

#[post("/{type}")]
pub async fn base(
    auth_dao: UserAuthQuery,
    path: actix_web::web::Path<String>,
    json_param: JsonQuery,
    bearer: BearerQuery,
    req_query: ReqQuery,
    web_dao: Data<WebDao>,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao
        .set_request_token(&bearer)
        .await
        .map_err(|e| req_query.fluent_error_json_response(&e))?;
    Ok(match path.into_inner().as_str() {
        "mapping" => mapping_data(&req_query).await,
        "login_history" => {
            login_history(&json_param.param::<LoginHistoryParam>()?, &auth_dao, web_dao.as_ref()).await
        }
        "set_password" => set_password(&json_param.param::<SetPasswordParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "delete" => delete(&json_param.param::<DeleteParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,

        "set_username" => {
            info_set_username(&json_param.param::<InfoSetUserNameParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "check_username" => {
            info_check_username(&json_param.param::<InfoCheckUserNameParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "password_modify" => password_last_modify(&auth_dao, web_dao.as_ref()).await,
        "set_info" => info_set_data(&json_param.param::<InfoSetUserInfoParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        name => handler_not_found!(name),
    }
    .map_err(|e| req_query.fluent_error_json_response(&e))?
    .into())
}
