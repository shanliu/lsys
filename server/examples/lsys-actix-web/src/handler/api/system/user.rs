use crate::common::handler::{
    BearerQuery, JsonQuery, ReqQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use actix_web::web::Data;
use lsys_web::dao::WebDao;
use lsys_web::handler::api::system::user::{
    AccountDetailParam, AccountSearchParam, ChangeLogsListParam, LoginHistoryParam,
    UserLogoutParam, account_detail, account_search, change_logs_list, login_history, mapping_data,
    user_logout,
};

#[post("/{method}")]
pub(crate) async fn user(
    bearer: BearerQuery,
    json_param: JsonQuery,
    auth_dao: UserAuthQuery,
    path: actix_web::web::Path<String>,
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
            login_history(&json_param.param::<LoginHistoryParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "user_logout" => user_logout(&json_param.param::<UserLogoutParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "account_search" => {
            account_search(&json_param.param::<AccountSearchParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "account_detail" => {
            account_detail(&json_param.param::<AccountDetailParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "change_logs" => {
            change_logs_list(&json_param.param::<ChangeLogsListParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        name => handler_not_found!(name),
    }
    .map_err(|e| req_query.fluent_error_json_response(&e))?
    .into())
}
