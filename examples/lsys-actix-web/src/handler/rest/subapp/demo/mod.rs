use crate::common::handler::{OauthAuthQuery, ResponseJson, ResponseJsonResult, RestQuery};

use actix_web::post;
use lsys_web_subapp_demo::handler::{demo_handler, DemoParam};

#[post("/subapp/demo")] //接口路径
pub(crate) async fn demo_app(
    rest: RestQuery,
    auth_dao: OauthAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&rest).await; //接口需要验证加这个
    Ok(match rest.rfc.method.as_deref() {
        Some("method_dome") => {
            //接口名
            let param = rest.param::<DemoParam>()?;
            let app = &rest
                .rfc
                .to_app_model(&auth_dao.web_dao.app.app_dao.app)
                .await?;
            demo_handler(&auth_dao.web_dao, app, param).await
        }
        var => handler_not_found!(var.unwrap_or_default()),
    }?
    .into())
}
