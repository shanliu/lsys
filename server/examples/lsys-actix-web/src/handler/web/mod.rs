mod captcha;
pub(crate) mod index;
use std::{path::PathBuf, str::FromStr, sync::Arc};

use actix_service::ServiceFactory;
use actix_web::{dev::ServiceRequest, web, App, Error};
use lsys_web::dao::WebDao;
use tracing::{info, warn};

pub(crate) fn router_ui<T>(app: App<T>, app_dao: &Arc<WebDao>) -> App<T>
where
    T: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>,
{
    if let Ok(ui_dir) = app_dao.app_core.config.get_string("ui_dir") {
        let mut ui_path = PathBuf::from_str(&ui_dir);
        if let Ok(ref ui_buf) = ui_path {
            if !ui_buf.exists() && !ui_buf.is_absolute() {
                let cargo_dir = env!("CARGO_MANIFEST_DIR");
                ui_path = PathBuf::from_str(&format!("{}/{}", cargo_dir, ui_dir))
            }
        }
        if let Ok(ui_buf) = ui_path {
            if ui_buf.exists() {
                return app.service(web::redirect("/ui", "/ui/")).service(
                    actix_files::Files::new("/ui/", ui_buf)
                        .index_file("index.html")
                        .show_files_listing(),
                );
            }
        }
        warn!("not find ui dir : {}", ui_dir);
    } else {
        info!("not set ui dir");
    }
    app
}

pub(crate) fn router<T>(app: App<T>, _app_dao: &Arc<WebDao>) -> App<T>
where
    T: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>,
{
    app.service(captcha::captcha)
        .service(index::index)
        .service(actix_files::Files::new("/static", "./static").show_files_listing())
}
