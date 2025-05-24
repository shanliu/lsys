//页面及其他
mod index;
pub mod system;
use actix_service::ServiceFactory;
use actix_web::web::to;
use actix_web::{dev::ServiceRequest, web, App, Error};
use lsys_web::dao::WebDao;
use lsys_web::lsys_core::IntoFluentMessage;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, info, warn};
fn router_ui<T>(app: App<T>, app_dao: &Arc<WebDao>) -> App<T>
where
    T: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>,
{
    let ui_path = app_dao
        .app_core
        .config
        .find(None)
        .get_string("ui_path")
        .unwrap_or_else(|_| "./ui".to_string());
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

fn router_page<T>(app: App<T>, app_dao: &Arc<WebDao>) -> App<T>
where
    T: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>,
{
    let static_serve_from = match app_dao
        .app_core
        .config_path(app_dao.app_core.config.find(None), "static_file_dir")
    {
        Ok(t) => t,
        Err(err) => {
            debug!(
                "static file dir wrong:{}",
                err.to_fluent_message().default_format()
            );
            PathBuf::from("./static")
        }
    };
    debug!("static dir is:{:?}", static_serve_from);
    app.service(actix_files::Files::new("/static", static_serve_from).show_files_listing())
        .service(index::dome)
}

pub(crate) fn router<T>(app: App<T>, app_dao: &Arc<WebDao>) -> App<T>
where
    T: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>,
{
    let app = router_page(app, app_dao);
    let app = router_ui(app, app_dao);
    app.default_service(to(system::render_404))
}
