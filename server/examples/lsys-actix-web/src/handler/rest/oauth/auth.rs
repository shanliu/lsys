use crate::common::handler::{
    OauthAuthQuery, ReqQuery, ResponseJson, ResponseJsonResult, RestQuery,
};

use actix_web::{get, post, web::Query};
use lsys_app::dao::session::RestAuthTokenData;
use lsys_user::dao::auth::{SessionToken, UserSession};

use lsys_web::handler::oauth::{
    login_data_from_oauth, oauth_create_token, OauthCodeParam, OauthDataOptionParam,
};
use lsys_web::handler::oauth::{oauth_refresh_token, OauthRefreshCodeParam};

#[get("/token")]
pub(crate) async fn token(
    token_param: Query<OauthCodeParam>,
    req_dao: ReqQuery,
) -> ResponseJsonResult<ResponseJson> {
    Ok(oauth_create_token(&req_dao, token_param.into_inner())
        .await?
        .into())
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
        .set_session_token(SessionToken::from_data(Some(RestAuthTokenData {
            client_id: param.client_id.clone(),
            token: param.refresh_token.clone(),
        })));
    Ok(oauth_refresh_token(&oauth_param, param).await?.into())
}

#[post("user")]
pub(crate) async fn user_data(
    mut rest: RestQuery,
    oauth_param: OauthAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    oauth_param.set_request_token(&rest).await;
    Ok(match rest.rfc.method.as_deref() {
        Some("info") => {
            login_data_from_oauth(
                rest.param::<OauthDataOptionParam>()?,
                &rest.to_app_model().await?,
                &oauth_param,
            )
            .await
        }
        var => handler_not_found!(var.unwrap_or_default()),
    }?
    .into())
}
