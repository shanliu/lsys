//! Service API handlers for internal service-to-service communication
//!
//! These endpoints are authenticated via X-Timestamp + X-Signature headers
//! and are intended for use by internal services
//!
//! Available endpoints:
//! - POST /service/auth/verify - Verify auth token and get user info (requires forward headers)
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
        // 令牌上传端点 /service/file/upload_by_token（multipart + 仅令牌鉴权）
        .service(scope("/file").service(super::file_token_upload::service_upload_by_token))
        .service(file::file);

    app.service(service_scope)
}
