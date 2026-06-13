use crate::common::handler::{ResponseJson, ResponseJsonResult, RestQuery, ReqQuery};
use actix_web::{post, web};
use lsys_web::dao::WebDao;
use lsys_web::handler::rest::app::{
    SubAppInfoParam, SubAppOAuthScopeParam, SubAppOAuthSecretParam, SubAppUserParam, subapp_info,
    subapp_oauth_scope, subapp_oauth_secret, subapp_user,
};

#[post("")]
pub(crate) async fn app(rest: RestQuery, req_dao: ReqQuery, web_dao: web::Data<WebDao>) -> ResponseJsonResult<ResponseJson> {
    Ok(match rest.rfc.method.as_deref().unwrap_or_default() {
        "sub_app_info" => {
            let param = rest.param::<SubAppInfoParam>()?;
            subapp_info(&param, &rest.get_app().await?, &req_dao, web_dao.as_ref()).await
        }
        "sub_app_user" => {
            let param = rest.param::<SubAppUserParam>()?;
            subapp_user(&param, &rest.get_app().await?, &req_dao, web_dao.as_ref()).await
        }
        "sub_app_oauth_secret" => {
            let param = rest.param::<SubAppOAuthSecretParam>()?;
            subapp_oauth_secret(&param, &rest.get_app().await?, &req_dao, web_dao.as_ref()).await
        }
        "sub_app_oauth_scope" => {
            let param = rest.param::<SubAppOAuthScopeParam>()?;
            subapp_oauth_scope(&param, &rest.get_app().await?, &req_dao, web_dao.as_ref()).await
        }
        var => handler_not_found!(var),
    }
    .map_err(|e| req_dao.fluent_error_json_response(&e))?
    .into())
}
