use crate::common::handler::{ResponseJson, ResponseJsonResult, RestQuery};
use actix_web::post;
use actix_web::web::Data;
use lsys_web::{
    dao::WebDao,
    handler::subapp::{sms_cancel, sms_send, SmsCancelParam, SmsSendParam},
};

#[post("sms")]
pub(crate) async fn sms(
    rest: RestQuery,
    app_dao: Data<WebDao>,
) -> ResponseJsonResult<ResponseJson> {
    Ok(match rest.rfc.method.as_deref() {
        Some("send") => {
            let param = rest.param::<SmsSendParam>()?;
            sms_send(
                &app_dao,
                &rest.rfc.to_app_model(&app_dao.app.app_dao.app).await?,
                param,
            )
            .await
        }
        Some("cancel") => {
            let param = rest.param::<SmsCancelParam>()?;
            sms_cancel(
                &app_dao,
                &rest.rfc.to_app_model(&app_dao.app.app_dao.app).await?,
                param,
            )
            .await
        }
        var => handler_not_found!(var.unwrap_or_default()),
    }?
    .into())
}
