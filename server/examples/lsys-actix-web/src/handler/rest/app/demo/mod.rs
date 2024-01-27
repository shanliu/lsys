use crate::common::handler::{OauthAuthQuery, ResponseJson, ResponseJsonResult, RestQuery};

use actix_web::post;
use lsys_web_subapp_demo::handler::{demo_handler, DemoParam};

#[post("/subapp/demo")] //接口路径
pub(crate) async fn demo_app(
    mut rest: RestQuery,
    oauth_param: OauthAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    oauth_param.set_request_token(&rest).await; //接口需要验证加这个
    Ok(match rest.rfc.method.as_deref() {
        Some("method_dome") => {
            //接口名
            let param = rest.param::<DemoParam>()?;
            let app = &rest.to_app_model().await?;
            demo_handler(&oauth_param, app, param).await
        }
        var => handler_not_found!(var.unwrap_or_default()),
    }?
    .into())
}
