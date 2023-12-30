use crate::common::handler::{ResponseJson, ResponseJsonResult, RestQuery};
use actix_web::{post, web::Data};
use lsys_web::{
    dao::WebDao,
    handler::app::{app_rbac_check, app_rbac_menu_check, CheckParam, MenuParam},
};

#[post("access")]
pub(crate) async fn access(
    mut rest: RestQuery,

    app_dao: Data<WebDao>,
) -> ResponseJsonResult<ResponseJson> {
    Ok(match rest.rfc.method.as_deref() {
        Some("check") => {
            let param = rest.param::<CheckParam>()?;
            app_rbac_check(
                &app_dao,
                &rest.rfc.to_app_model(&app_dao.app.app_dao.app).await?,
                param,
            )
            .await
        }
        // Some("access") => {
        //     let param = rest.param::<AccessParam>()?;
        //     subapp_rbac_access_check(
        //         &app_dao,
        //         &rest.rfc.to_app_model(&app_dao.app.app_dao.app).await?,
        //         param,
        //     )
        //     .await
        // }
        Some("menu") => {
            let param = rest.param::<MenuParam>()?;
            app_rbac_menu_check(
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
