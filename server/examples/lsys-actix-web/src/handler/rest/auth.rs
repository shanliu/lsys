use crate::common::handler::{ResponseJson, ResponseJsonResult, RestQuery, ReqQuery};
use actix_web::{post, web};
use lsys_web::dao::WebDao;
use lsys_web::handler::rest::auth::{
    DoLoginParam, DoLogoutParam, LoginInfoParam, MfaEnableParam, MfaIsEnabledParam, MfaVerifyParam,
    do_login, do_logout, login_info, mfa_enable, mfa_is_enabled, mfa_verify,
};

#[post("")]
pub(crate) async fn auth(rest: RestQuery, req_dao: ReqQuery, web_dao: web::Data<WebDao>) -> ResponseJsonResult<ResponseJson> {
    Ok(match rest.rfc.method.as_deref().unwrap_or_default() {
        "do_login" => {
            let param = rest.param::<DoLoginParam>()?;
            do_login(&param, &rest.get_app().await?, &req_dao, web_dao.as_ref()).await
        }
        "do_logout" => {
            let param = rest.param::<DoLogoutParam>()?;
            do_logout(&param, &rest.get_app().await?, web_dao.as_ref()).await
        }
        "login_info" => {
            let param = rest.param::<LoginInfoParam>()?;
            login_info(&param, &rest.get_app().await?, &req_dao, web_dao.as_ref()).await
        }
        "mfa_is_enabled" => {
            let param = rest.param::<MfaIsEnabledParam>()?;
            mfa_is_enabled(&param, &rest.get_app().await?, &req_dao, web_dao.as_ref()).await
        }
        "mfa_enable" => {
            let param = rest.param::<MfaEnableParam>()?;
            mfa_enable(&param, &rest.get_app().await?, &req_dao, web_dao.as_ref()).await
        }
        "mfa_verify" => {
            let param = rest.param::<MfaVerifyParam>()?;
            mfa_verify(&param, &rest.get_app().await?, &req_dao, web_dao.as_ref()).await
        }
        var => handler_not_found!(var),
    }
    .map_err(|e| req_dao.fluent_error_json_response(&e))?
    .into())
}
