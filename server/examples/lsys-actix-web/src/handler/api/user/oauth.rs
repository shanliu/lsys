use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use lsys_web::handler::oauth::{
    oauth_create_code, oauth_scope_get, OauthAuthorizeDoParam, OauthScopeGetParam,
};

#[post("oauth/{method}")]
pub(crate) async fn oauth<'t>(
    jwt: JwtQuery,
    path: actix_web::web::Path<(String,)>,
    json_param: JsonQuery,
    auth_dao: UserAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    Ok(match path.0.to_string().as_str() {
        "scope" => oauth_scope_get(json_param.param::<OauthScopeGetParam>()?, &auth_dao).await,
        "do" => oauth_create_code(json_param.param::<OauthAuthorizeDoParam>()?, &auth_dao).await,
        name => handler_not_found!(name),
    }?
    .into())
}
