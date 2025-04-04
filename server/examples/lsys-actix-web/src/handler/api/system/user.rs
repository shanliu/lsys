use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use lsys_web::handler::api::system::user::{
    account_id_search, account_search, app_logout, change_logs_list, login_history, user_logout,
    AccountIdSearchParam, AccountSearchParam, AppLogoutParam, ChangeLogsListParam,
    LoginHistoryParam, UserLogoutParam,
};

#[post("/{method}")]
pub(crate) async fn user(
    jwt: JwtQuery,
    json_param: JsonQuery,
    auth_dao: UserAuthQuery,
    path: actix_web::web::Path<String>,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    Ok(match path.into_inner().as_str() {
        "login_history" => {
            login_history(&json_param.param::<LoginHistoryParam>()?, &auth_dao).await
        }
        "user_logout" => user_logout(&json_param.param::<UserLogoutParam>()?, &auth_dao).await,
        "app_logout" => app_logout(&json_param.param::<AppLogoutParam>()?, &auth_dao).await,
        "account_search" => {
            account_search(&json_param.param::<AccountSearchParam>()?, &auth_dao).await
        }
        "account_id_search" => {
            account_id_search(&json_param.param::<AccountIdSearchParam>()?, &auth_dao).await
        }
        "change_logs" => {
            change_logs_list(&json_param.param::<ChangeLogsListParam>()?, &auth_dao).await
        }
        name => handler_not_found!(name),
    }
    .map_err(|e| auth_dao.fluent_error_json_data(&e))?
    .into())
}
