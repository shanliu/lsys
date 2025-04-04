//回调页面接口
mod sms;
use actix_service::ServiceFactory;
use actix_web::{dev::ServiceRequest, web::scope, App, Error};
pub(crate) fn router<T>(app: App<T>) -> App<T>
where
    T: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>,
{
    app.service(scope("/notify/sms").service(sms::notify))
}
