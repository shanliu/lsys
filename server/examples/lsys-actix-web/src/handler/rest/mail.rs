use crate::common::handler::{ResponseJson, ResponseJsonResult, RestQuery, ReqQuery};
use actix_web::{post, web};
use lsys_web::dao::WebDao;
use lsys_web::handler::rest::mailer::{CancelParam, SendParam, cancel, send};

#[post("")]
pub(crate) async fn mail(rest: RestQuery, req_dao: ReqQuery, web_dao: web::Data<WebDao>) -> ResponseJsonResult<ResponseJson> {
    Ok(match rest.rfc.method.as_deref().unwrap_or_default() {
        "send" => {
            let param = rest.param::<SendParam>()?;
            send(&param, &rest.get_app().await?, &req_dao, web_dao.as_ref()).await
        }
        "cancel" => {
            let param = rest.param::<CancelParam>()?;
            cancel(&param, &rest.get_app().await?, &req_dao, web_dao.as_ref()).await
        }
        var => handler_not_found!(var),
    }
    .map_err(|e| req_dao.fluent_error_json_response(&e))?
    .into())
}
