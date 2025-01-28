use crate::common::handler::{
    OauthAuthQuery, ReqQuery, ResponseJson, ResponseJsonResult, RestQuery,
};
use actix_web::{get, post, web::Query};
use lsys_access::dao::AccessSession;
use lsys_app::dao::RestAuthToken;
use lsys_web::handler::rest::{
    account_data_from_oauth, oauth_create_token, oauth_refresh_token, AccountOptionDataParam,
    OauthCodeParam, OauthRefreshCodeParam,
};

#[get("/token")]
pub(crate) async fn token(
    token_param: Query<OauthCodeParam>,
    req_dao: ReqQuery,
) -> ResponseJsonResult<ResponseJson> {
    Ok(
        oauth_create_token(&req_dao, token_param.into_inner()) //系统oauth
            .await
            .map_err(|e| req_dao.fluent_error_json_data(&e))?
            .into(),
    )
}

#[get("/refresh_token")]
pub(crate) async fn refresh(
    token_param: Query<OauthRefreshCodeParam>,
    oauth_param: OauthAuthQuery,
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
    Ok(oauth_refresh_token(&param, &oauth_param)
        .await
        .map_err(|e| oauth_param.fluent_error_json_data(&e))?
        .into())
}

#[post("user")]
pub(crate) async fn user_data(
    mut rest: RestQuery,
    oauth_param: OauthAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    oauth_param.set_request_token(&rest).await;
    Ok(match rest.rfc.method.as_deref() {
        Some("info") => {
            account_data_from_oauth(&rest.param::<AccountOptionDataParam>()?, &oauth_param).await
        }
        var => handler_not_found!(var.unwrap_or_default()),
    }
    .map_err(|e| oauth_param.fluent_error_json_data(&e))?
    .into())
}
