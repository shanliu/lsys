use actix_web::dev::Server;
use actix_web::web::{Data, JsonConfig};
use actix_web::{error, http, middleware as middlewares, App, HttpResponse, HttpServer};
use common::handler::{JwtQueryConfig, RestQueryConfig};

use futures_util::TryFutureExt;
use handler::router_main;
use jsonwebtoken::{DecodingKey, Validation};
use lsys_core::{AppCore, AppCoreError};
use lsys_web::common::FluentFormat;
use lsys_web::dao::WebDao;
use rustls::server::ServerConfig;
use rustls::{Certificate, PrivateKey};
use rustls_pemfile::{certs, read_one, Item};

use std::fmt::{Display, Formatter};
use std::sync::Arc;
use std::{fs::File, io::BufReader};

mod common;
mod handler;

#[derive(Debug)]
pub enum AppError {
    AppCore(AppCoreError),
    Rustls(rustls::Error),
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for AppError {}
impl From<AppCoreError> for AppError {
    fn from(err: AppCoreError) -> Self {
        AppError::AppCore(err)
    }
}
impl From<rustls::Error> for AppError {
    fn from(err: rustls::Error) -> Self {
        AppError::Rustls(err)
    }
}

fn load_rustls_config(
    app_dir: &str,
    cert_path: &str,
    key_path: &str,
) -> Result<ServerConfig, AppError> {
    // init server config builder with safe defaults
    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth();

    // load TLS key/cert files
    //config_dir.to_owned() + "data/cert.pem"
    let cert_path = if cert_path.strip_suffix('/').is_some() {
        cert_path.to_string()
    } else {
        app_dir.to_string() + cert_path
    };
    let key_path = if key_path.strip_suffix('/').is_some() {
        key_path.to_string()
    } else {
        app_dir.to_string() + key_path
    };
    let cert_file = &mut BufReader::new(File::open(cert_path).map_err(AppCoreError::Io)?);
    //config_dir.to_owned() + "data/key.pem"
    let key_file = &mut BufReader::new(File::open(key_path).map_err(AppCoreError::Io)?);

    // convert files to key/cert objects
    let mut cert_chain = vec![];
    for tmp in certs(cert_file) {
        match tmp {
            Ok(t) => {
                cert_chain.push(Certificate(t.as_ref().to_vec()));
            }
            Err(err) => return Err(AppError::AppCore(AppCoreError::Io(err))),
        }
    }
    let key = match read_one(key_file).map_err(AppCoreError::Io)? {
        Some(Item::Pkcs8Key(key)) => key.secret_pkcs8_der().to_vec(),
        _ => {
            return Err(AppError::AppCore(AppCoreError::System(
                "only support pcks key".to_owned(),
            )))
        }
    };
    Ok(config.with_single_cert(cert_chain, PrivateKey(key))?)
}

pub async fn create_server(app_dir: &str) -> Result<Server, AppError> {
    let mut app_core = AppCore::init(app_dir, "config", None).await?;
    app_core.init_tracing()?;
    let app_core = Arc::new(app_core);
    //console_subscriber::init();
    let app_dao = Data::new(WebDao::new(app_core.clone()).await?);
    let bind_addr = app_dao.bind_addr();
    let bind_ssl_data = app_dao.bind_ssl_data();
    let is_redirect_http = bind_ssl_data.is_some();
    let app_jwt_key = app_dao
        .app_core
        .config
        .find(None)
        .get_string("app_jwt_key")
        .map_err(|err| AppCoreError::Config(lsys_core::ConfigError::Config(err)))?;
    // let app_rest_config = app_dao.app_core.config.get_table("app_rest").unwrap();
    let mut server = HttpServer::new(move || {
        let jwt_config = JwtQueryConfig::new(
            DecodingKey::from_secret(app_jwt_key.as_bytes()),
            Validation::default(),
        );
        let app_json_limit = app_dao
            .app_core
            .config
            .find(None)
            .get_int("app_json_limit")
            .unwrap_or(4096);
        let json_config = JsonConfig::default()
            .limit(app_json_limit as usize)
            .error_handler(|err, _req| {
                error::InternalError::from_response(err, HttpResponse::Conflict().finish()).into()
            });

        let rest_config =
            RestQueryConfig::default().app_key_fn(Box::new(move |app_key, app_data| {
                // Box::pin(async move {
                //     sleep(Duration::from_secs(1)).await;
                //     if app_key == "app01" {
                //         Ok("f4dea3417a2f52ae29a635be00537395".to_owned())
                //     } else {
                //         Err("key not find".to_owned())
                //     }
                // })
                let apps = app_data.web_app.app_dao.app.clone();
                Box::pin(async move {
                    apps.cache()
                        .find_secret_by_client_id(&app_key)
                        .map_err(|e| e.fluent_format(&app_data.fluent.locale(None)))
                        .await
                })
            }));

        let app = App::new()
            .wrap(common::middleware::RedirectSsl::new(is_redirect_http))
            .wrap(middlewares::Logger::default())
            .wrap(middlewares::Compress::default())
            .wrap(
                middlewares::ErrorHandlers::new()
                    .handler(http::StatusCode::INTERNAL_SERVER_ERROR, handler::render_500),
            )
            .wrap(middlewares::DefaultHeaders::new().add(("Access-Control-Allow-Origin", "*")))
            .wrap(common::middleware::RequestID::new(None))
            .app_data(app_dao.clone())
            .app_data(json_config)
            .app_data(jwt_config)
            .app_data(rest_config);
        router_main(app, &app_dao)
    });
    server = server.bind(bind_addr).map_err(AppCoreError::Io)?;
    if let Some((ssl_addr, cert_file, key_file)) = bind_ssl_data {
        server = server
            .bind_rustls(
                ssl_addr,
                load_rustls_config(app_dir, &cert_file, &key_file)?,
            )
            .map_err(AppCoreError::Io)?;
    }
    let s = server.run();
    Ok(s)
}
