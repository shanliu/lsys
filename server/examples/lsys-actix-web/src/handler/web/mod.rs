mod captcha;
pub(crate) mod index;
use std::sync::Arc;

use actix_service::ServiceFactory;
use actix_web::{
    dev::ServiceRequest,
    web::{self, scope},
    App, Error,
};
use lsys_web::dao::WebDao;
use tracing::{info, warn};

pub(crate) fn router_ui<T>(app: App<T>, app_dao: &Arc<WebDao>) -> App<T>
where
    T: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>,
{
    let ui_path = app_dao
        .app_core
        .config
        .find(None)
        .get_string("ui_path")
        .unwrap_or_else(|_| "/ui".to_string());
    if let Ok(ui_config) = app_dao
        .app_core
        .config_path(app_dao.app_core.config.find(None), "ui_dir")
    {
        if ui_config.exists() {
            let ui_path_full = ui_path.trim_matches('/').to_string() + "/";
            return app
                .service(web::redirect(
                    ui_path.trim_matches('/').to_string(),
                    ui_path_full.to_owned(),
                ))
                .service(
                    actix_files::Files::new(&ui_path_full, ui_config)
                        .index_file("index.html")
                        .show_files_listing(),
                );
        }
        warn!("not find ui dir : {}", ui_config.display());
    } else {
        info!("not set ui dir");
    }
    app
}

pub(crate) fn router<T>(app: App<T>, app_dao: &Arc<WebDao>) -> App<T>
where
    T: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>,
{
    let static_serve_from = app_dao
        .app_core
        .config
        .find(None)
        .get_string("static_serve_from")
        .unwrap_or_else(|_| String::from("./static"));
    let mut app = app
        .service(scope("/captcha").service(captcha::captcha))
        .service(actix_files::Files::new("/static", static_serve_from).show_files_listing());
    app = app.service(index::dome);
    app
}
