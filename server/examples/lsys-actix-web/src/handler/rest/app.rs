use crate::common::handler::{ResponseJson, ResponseJsonResult, RestQuery};
use actix_web::post;
use lsys_web::handler::rest::app::{
    subapp_info, subapp_oauth_scope, subapp_oauth_secret, subapp_user, SubAppInfoParam,
    SubAppOAuthScopeParam, SubAppOAuthSecretParam, SubAppUserParam,
};

#[post("")]
pub(crate) async fn app(rest: RestQuery) -> ResponseJsonResult<ResponseJson> {
    Ok(match rest.rfc.method.as_deref().unwrap_or_default() {
        "sub_app_info" => {
            let param = rest.param::<SubAppInfoParam>()?;
            subapp_info(&param, &rest.get_app().await?, &rest).await
        }
        "sub_app_user" => {
            let param = rest.param::<SubAppUserParam>()?;
            subapp_user(&param, &rest.get_app().await?, &rest).await
        }
        "sub_app_oauth_secret" => {
            let param = rest.param::<SubAppOAuthSecretParam>()?;
            subapp_oauth_secret(&param, &rest.get_app().await?, &rest).await
        }
        "sub_app_oauth_scope" => {
            let param = rest.param::<SubAppOAuthScopeParam>()?;
            subapp_oauth_scope(&param, &rest.get_app().await?, &rest).await
        }
        var => handler_not_found!(var),
    }
    .map_err(|e| rest.fluent_error_json_response(&e))?
    .into())
}
