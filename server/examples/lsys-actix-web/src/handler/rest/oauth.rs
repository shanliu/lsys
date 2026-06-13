use crate::common::handler::{
    OauthAuthQuery, ReqQuery, ResponseJson, ResponseJsonResult, RestQuery,
};
use actix_web::{get, post, web, web::Query};
use lsys_web::dao::WebDao;
use lsys_web::handler::rest::oauth::{
    AccountOptionDataParam, CodeParam, RefreshCodeParam, account_data_from_oauth, create_token,
    refresh_token,
};
use lsys_web::lsys_access::dao::AccessSession;
use lsys_web::lsys_app::dao::RestAuthToken;

#[get("/token")]
pub(crate) async fn token(
    token_param: Query<CodeParam>,
    req_dao: ReqQuery,
    web_dao: web::Data<WebDao>,
) -> ResponseJsonResult<ResponseJson> {
    Ok(
        create_token(&req_dao, web_dao.as_ref(), &token_param.into_inner()) //系统oauth
            .await
            .map_err(|e| req_dao.fluent_error_json_response(&e))?
            .into(),
    )
}

#[get("/refresh_token")]
pub(crate) async fn refresh(
    token_param: Query<RefreshCodeParam>,
    oauth_param: OauthAuthQuery,
    req_query: ReqQuery,
    web_dao: web::Data<WebDao>,
) -> ResponseJsonResult<ResponseJson> {
    let param = token_param.into_inner();
    oauth_param
        .user_session
        .write()
        .await
        .set_session_token(RestAuthToken {
            client_id: param.client_id.clone(),
            token: param.refresh_token.clone(),
        });
    Ok(refresh_token(&param, &req_query, web_dao.as_ref())
        .await
        .map_err(|e| req_query.fluent_error_json_response(&e))?
        .into())
}

#[post("/user")]
pub(crate) async fn user_data(
    rest: RestQuery,
    oauth_param: OauthAuthQuery,
    req_query: ReqQuery,
    web_dao: web::Data<WebDao>,
) -> ResponseJsonResult<ResponseJson> {
    oauth_param
        .set_request_token(&rest)
        .await
        .map_err(|e| req_query.fluent_error_json_response(&e))?;
    Ok(match rest.rfc.method.as_deref() {
        Some("info") => {
            account_data_from_oauth(&rest.param::<AccountOptionDataParam>()?, &oauth_param, web_dao.as_ref()).await
        }
        var => handler_not_found!(var.unwrap_or_default()),
    }
    .map_err(|e| req_query.fluent_error_json_response(&e))?
    .into())
}
