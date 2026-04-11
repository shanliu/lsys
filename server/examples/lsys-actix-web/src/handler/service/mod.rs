//! Service API handlers for internal service-to-service communication
//!
//! These endpoints are authenticated via X-Timestamp + X-Signature headers
//! and are intended for use by internal services
//!
//! Available endpoints:
//! - POST /service/auth/verify - Verify JWT and get user info (requires forward headers)
//! - POST /service/rbac/check - Check RBAC permissions for multiple items
//! - POST /service/app/{method} - App related operations (feature, secret)

mod app;
mod auth;
mod file;
mod rbac;

use actix_service::ServiceFactory;
use actix_web::{App, Error, dev::ServiceRequest, web::scope};

pub(crate) fn router<T>(app: App<T>) -> App<T>
where
    T: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>,
{
    let service_scope = scope("/service")
        .service(auth::verify)
        .service(rbac::check)
        .service(app::app)
        .service(file::file);

    app.service(service_scope)
}
