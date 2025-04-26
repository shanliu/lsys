//rest 接口
mod app;
mod auth;
#[cfg(feature = "barcode")]
mod barcode;
mod mail;
mod oauth;
mod rbac;
mod sms;

//后台页面接口(jwt 接口)
use actix_service::ServiceFactory;
use actix_web::{dev::ServiceRequest, web::scope, App, Error};
pub(crate) fn router<T>(app: App<T>) -> App<T>
where
    T: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>,
{
    let rest_scope = scope("/rest")
        .service(
            scope("/rbac")
                .service(rbac::role)
                .service(rbac::res)
                .service(rbac::check)
                .service(rbac::op),
        )
        .service(scope("/app").service(app::app))
        .service(scope("/auth").service(auth::auth))
        .service(scope("/mail").service(mail::mail))
        .service(scope("/sms").service(sms::sms));

    #[cfg(feature = "barcode")]
    let rest_scope = rest_scope.service(scope("/barcode").service(barcode::barcode));

    app.service(
        scope("/oauth")
            .service(oauth::token)
            .service(oauth::refresh)
            .service(oauth::user_data),
    )
    .service(rest_scope)
}
