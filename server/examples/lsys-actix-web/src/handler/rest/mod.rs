//rest 接口
mod app;
mod auth;
mod collector;
mod mail;
mod oauth;
mod rbac;
mod sms;

use actix_service::ServiceFactory;
use actix_web::{App, Error, dev::ServiceRequest, web::scope};

pub(crate) fn router<T>(app: App<T>) -> App<T>
where
    T: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>,
{
    let rest_scope = scope("/rest")
        .service(
            scope("/rbac")
                .service(rbac::role)
                .service(rbac::res)
                .service(rbac::base)
                .service(rbac::op),
        )
        .service(scope("/app").service(app::app))
        .service(scope("/auth").service(auth::auth))
        .service(scope("/mail").service(mail::mail))
        .service(scope("/sms").service(sms::sms))
        .service(scope("/collector").service(collector::collector));

    app.service(
        scope("/oauth")
            .service(oauth::token)
            .service(oauth::refresh)
            .service(oauth::user_data),
    )
    .service(rest_scope)
}
