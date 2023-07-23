mod captcha;
pub(crate) mod index;
use std::{path::PathBuf, str::FromStr, sync::Arc};

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
        .get_string("ui_path")
        .unwrap_or_else(|_| "/ui".to_string());
    if let Ok(ui_config) = app_dao.app_core.config.get_string("ui_dir") {
        let mut ui_dir = PathBuf::from_str(&ui_config);
        if let Ok(ref ui_buf) = ui_dir {
            if !ui_buf.exists() && !ui_buf.is_absolute() {
                let cargo_dir = env!("CARGO_MANIFEST_DIR");
                ui_dir = PathBuf::from_str(&format!("{}/{}", cargo_dir, ui_config))
            }
        }
        if let Ok(ui_buf) = ui_dir {
            if ui_buf.exists() {
                let ui_path_full = ui_path.trim_matches('/').to_string() + "/";
                return app
                    .service(web::redirect(
                        ui_path.trim_matches('/').to_string(),
                        ui_path_full.to_owned(),
                    ))
                    .service(
                        actix_files::Files::new(&ui_path_full, ui_buf)
                            .index_file("index.html")
                            .show_files_listing(),
                    );
            }
        }
        warn!("not find ui dir : {}", ui_config);
    } else {
        info!("not set ui dir");
    }
    app
}

pub(crate) fn router<T>(app: App<T>, _app_dao: &Arc<WebDao>) -> App<T>
where
    T: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>,
{
    app.service(scope("/captcha").service(captcha::captcha))
        .service(index::index)
        .service(actix_files::Files::new("/static", "./static").show_files_listing())
}
