use crate::common::handler::{ResponseJson, ResponseJsonResult, RestQuery};
use actix_web::post;
use lsys_web::handler::rest::mailer::{cancel, send, CancelParam, SendParam};

#[post("mail")]
pub(crate) async fn mail(mut rest: RestQuery) -> ResponseJsonResult<ResponseJson> {
    Ok(match rest.rfc.method.as_deref() {
        Some("send") => {
            let param = rest.param::<SendParam>()?;
            send(&param, &rest.get_app().await?, &rest).await
        }
        Some("cancel") => {
            let param = rest.param::<CancelParam>()?;
            cancel(&param, &rest.get_app().await?, &rest).await
        }
        var => handler_not_found!(var.unwrap_or_default()),
    }
    .map_err(|e| rest.fluent_error_json_data(&e))?
    .into())
}
