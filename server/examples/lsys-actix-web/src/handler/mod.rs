#[macro_use]
mod macros;
mod api;
mod rest;
mod web;
use std::sync::Arc;

use actix_service::ServiceFactory;
use actix_web::{dev::ServiceRequest, web::to, App, Error};
use lsys_web::dao::WebDao;
use web::index;
//pub(crate) use web::index::render_404;
pub(crate) use web::index::render_500;

pub(crate) fn router_main<T>(app: App<T>, app_dao: &Arc<WebDao>) -> App<T>
where
    T: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>,
{
    let app = api::router(app);
    let app = rest::router(app);
    let app = web::router_ui(app, app_dao);
    let app = web::router(app, app_dao);
    app.default_service(to(index::render_404))
}
