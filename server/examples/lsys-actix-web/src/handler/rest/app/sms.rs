use crate::common::handler::{ResponseJson, ResponseJsonResult, RestQuery};
use actix_web::post;
use lsys_web::handler::app::{sms_cancel, sms_send, SmsCancelParam, SmsSendParam};

#[post("sms")]
pub(crate) async fn sms(mut rest: RestQuery) -> ResponseJsonResult<ResponseJson> {
    Ok(match rest.rfc.method.as_deref() {
        Some("send") => {
            let param = rest.param::<SmsSendParam>()?;
            sms_send(&rest, &rest.to_app_model().await?, param).await
        }
        Some("cancel") => {
            let param = rest.param::<SmsCancelParam>()?;
            sms_cancel(&rest, &rest.to_app_model().await?, param).await
        }
        var => handler_not_found!(var.unwrap_or_default()),
    }?
    .into())
}
