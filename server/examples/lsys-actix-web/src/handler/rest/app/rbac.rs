use crate::common::handler::{ResponseJson, ResponseJsonResult, RestQuery};
use actix_web::post;
use lsys_web::handler::rest::{app_rbac_check, CheckParam};

#[post("access")]
pub(crate) async fn access(mut rest: RestQuery) -> ResponseJsonResult<ResponseJson> {
    Ok(match rest.rfc.method.as_deref() {
        Some("check") => {
            let param = rest.param::<CheckParam>()?;
            app_rbac_check(&param, &rest.get_app().await?, &rest).await
        }
        // Some("menu") => {
        //     let param = rest.param::<MenuParam>()?;
        //     app_rbac_menu_check(&param, &rest.get_app().await?, rest).await
        // }
        var => handler_not_found!(var.unwrap_or_default()),
    }
    .map_err(|e| rest.fluent_error_json_data(&e))?
    .into())
}
