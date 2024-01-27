use crate::common::handler::{ResponseJson, ResponseJsonResult, RestQuery};
use actix_web::post;
use lsys_web::handler::app::{mail_cancel, mail_send, MailCancelParam, MailSendParam};

#[post("mail")]
pub(crate) async fn mail(mut rest: RestQuery) -> ResponseJsonResult<ResponseJson> {
    Ok(match rest.rfc.method.as_deref() {
        Some("send") => {
            let param = rest.param::<MailSendParam>()?;
            mail_send(&rest, &rest.to_app_model().await?, param).await
        }
        Some("cancel") => {
            let param = rest.param::<MailCancelParam>()?;
            mail_cancel(&rest, &rest.to_app_model().await?, param).await
        }
        var => handler_not_found!(var.unwrap_or_default()),
    }?
    .into())
}
