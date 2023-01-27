use crate::common::handler::{OauthAuthQuery, ResponseJson, ResponseJsonResult, RestQuery};

use actix_web::web::Data;
use actix_web::{get, post, web::Query};
use lsys_app::dao::session::RestAuthTokenData;
use lsys_user::dao::auth::{SessionToken, UserSession};

use lsys_web::dao::WebDao;
use lsys_web::handler::oauth::{
    login_data_from_oauth, oauth_create_token, OauthCodeParam, OauthDataOptionParam,
};
use lsys_web::handler::oauth::{oauth_refresh_token, OauthRefreshCodeParam};

#[get("/token")]
pub(crate) async fn token(
    token_param: Query<OauthCodeParam>,
    web_dao: Data<WebDao>,
) -> ResponseJsonResult<ResponseJson> {
    Ok(oauth_create_token(&web_dao, token_param.into_inner())
        .await?
        .into())
}

#[get("/refresh_token")]
pub(crate) async fn refresh(
    token_param: Query<OauthRefreshCodeParam>,
    auth_dao: OauthAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    let param = token_param.into_inner();
    auth_dao
        .user_session
        .write()
        .await
        .set_session_token(SessionToken::from_data(Some(RestAuthTokenData {
            client_id: param.client_id,
            token: param.refresh_token,
        })));
    Ok(oauth_refresh_token(&auth_dao).await?.into())
}

#[post("/user_info")]
pub(crate) async fn user_info(
    rest: RestQuery,
    auth_dao: OauthAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&rest).await;
    Ok(
        login_data_from_oauth(rest.param::<OauthDataOptionParam>()?, &auth_dao)
            .await?
            .into(),
    )
}
