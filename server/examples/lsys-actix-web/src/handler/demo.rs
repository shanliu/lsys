//后台页面接口(jwt 接口)
use crate::common::handler::RestQuery;
use crate::common::handler::{ResponseJson, ResponseJsonResult};
use actix_service::ServiceFactory;
use actix_web::post;
use actix_web::{dev::ServiceRequest, web::scope, App, Error};
pub(crate) fn router<T>(app: App<T>) -> App<T>
where
    T: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>,
{
    //子应用示例接入 /dome_api/test/?method=api1
    app.service(scope("/dome_api").service(demo_app))
}

#[post("test")]
pub(crate) async fn demo_app(rest: RestQuery) -> ResponseJsonResult<ResponseJson> {
    Ok(match rest.rfc.method.as_deref().unwrap_or_default() {
        "api1" => {
            lsys_web_subapp_demo::handler::demo_api1(
                &rest.param::<lsys_web_subapp_demo::handler::DemoParam>()?,
                &rest.get_app().await?,
                &rest,
            )
            .await
        }
        var => handler_not_found!(var),
    }
    .map_err(|e| rest.fluent_error_json_response(&e))?
    .into())
}
