use crate::common::handler::{ResponseJson, ResponseJsonResult, RestQuery};
use actix_web::post;
use actix_web::web::Data;
use lsys_web::{
    dao::WebDao,
    handler::app::{mail_cancel, mail_send, MailCancelParam, MailSendParam},
};

#[post("mail")]
pub(crate) async fn mail(
    mut rest: RestQuery,
    app_dao: Data<WebDao>,
) -> ResponseJsonResult<ResponseJson> {
    Ok(match rest.rfc.method.as_deref() {
        Some("send") => {
            let param = rest.param::<MailSendParam>()?;
            mail_send(
                &app_dao,
                &rest.rfc.to_app_model(&app_dao.app.app_dao.app).await?,
                param,
                Some(&rest.req_env),
            )
            .await
        }
        Some("cancel") => {
            let param = rest.param::<MailCancelParam>()?;
            mail_cancel(
                &app_dao,
                &rest.rfc.to_app_model(&app_dao.app.app_dao.app).await?,
                param,
                Some(&rest.req_env),
            )
            .await
        }
        var => handler_not_found!(var.unwrap_or_default()),
    }?
    .into())
}
