use crate::common::handler::{ResponseJson, ResponseJsonResult, RestQuery};
use actix_web::post;
use lsys_web::handler::rest::auth::{
    do_login, do_logout, login_info, DoLoginParam, DoLogoutParam, LoginInfoParam,
};

#[post("/auth")]
pub(crate) async fn auth(rest: RestQuery) -> ResponseJsonResult<ResponseJson> {
    Ok(match rest.rfc.method.as_deref().unwrap_or_default() {
        "do_login" => {
            let param = rest.param::<DoLoginParam>()?;
            do_login(&param, &rest.get_app().await?, &rest).await
        }
        "do_logout" => {
            let param = rest.param::<DoLogoutParam>()?;
            do_logout(&param, &rest.get_app().await?, &rest).await
        }
        "login_info" => {
            let param = rest.param::<LoginInfoParam>()?;
            login_info(&param, &rest.get_app().await?, &rest).await
        }
        var => handler_not_found!(var),
    }
    .map_err(|e| rest.fluent_error_json_response(&e))?
    .into())
}
