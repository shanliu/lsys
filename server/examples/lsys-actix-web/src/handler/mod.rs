#[macro_use]
mod macros;
mod api;
mod demo;
mod rest;
mod web;
use actix_service::ServiceFactory;
use actix_web::{dev::ServiceRequest, App, Error};
use lsys_web::dao::WebDao;
use std::sync::Arc;
pub(crate) use web::system::render_500;

pub(crate) fn router<T>(app: App<T>, app_dao: &Arc<WebDao>) -> App<T>
where
    T: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>,
{
    let app = demo::router(app);
    let app = api::router(app);
    let app = rest::router(app);
    web::router(app, app_dao)
}
