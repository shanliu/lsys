use actix_web::web::{Data, JsonConfig};
use actix_web::{error, http, middleware as middlewares, App, HttpResponse, HttpServer};
use common::handler::{JwtQueryConfig, RestQueryConfig};

use handler::router_main;
use jsonwebtoken::{DecodingKey, Validation};
use lsys_core::AppCore;
use lsys_web::dao::WebDao;

use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use rustls::server::ServerConfig;
use rustls::{Certificate, PrivateKey};
use rustls_pemfile::{certs, read_one, Item};
use std::{fs::File, io::BufReader};

mod common;
mod handler;

fn load_rustls_config(config_dir: &str) -> ServerConfig {
    // init server config builder with safe defaults
    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth();

    // load TLS key/cert files

    let cert_file =
        &mut BufReader::new(File::open(config_dir.to_owned() + "data/cert.pem").unwrap());
    let key_file = &mut BufReader::new(File::open(config_dir.to_owned() + "data/key.pem").unwrap());

    // convert files to key/cert objects
    let cert_chain = certs(cert_file)
        .unwrap()
        .into_iter()
        .map(Certificate)
        .collect();
    let key = match read_one(key_file).unwrap() {
        Some(Item::PKCS8Key(key)) => key,
        _ => {
            panic!("private not support");
        }
    };
    config
        .with_single_cert(cert_chain, PrivateKey(key))
        .unwrap()
}

#[actix_web::main]
async fn main() {
    let app_dir = if PathBuf::from_str("./").unwrap().join("./config").exists() {
        "./"
    } else {
        //on dev
        env!("CARGO_MANIFEST_DIR")
    };
    let app_core = Arc::new(AppCore::init(app_dir, &["config/app.toml"]).await.unwrap());
    let app_dao = Data::new(WebDao::new(app_core.clone()).await.unwrap());
    let bind_addr = app_dao.bind_addr();
    let bind_ssl_addr = app_dao.bind_ssl_addr();
    let mut server = HttpServer::new(move || {
        let app_jwt_key = app_dao.app_core.config.get_string("app_jwt_key").unwrap();
        let jwt_config = JwtQueryConfig::new(
            DecodingKey::from_secret(app_jwt_key.as_bytes()),
            Validation::default(),
        );
        let app_json_limit = app_dao
            .app_core
            .config
            .get_int("app_json_limit")
            .unwrap_or(4096);
        let json_config = JsonConfig::default()
            .limit(app_json_limit as usize)
            .error_handler(|err, _req| {
                error::InternalError::from_response(err, HttpResponse::Conflict().finish()).into()
            });
        // let app_rest_config = app_dao.app_core.config.get_table("app_rest").unwrap();

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
                let apps = app_data.app.app_dao.app.clone();
                Box::pin(async move { apps.innernal_client_id_get(&app_key).await })
            }));

        let app = App::new()
            .wrap(middlewares::Logger::default())
            .wrap(
                middlewares::ErrorHandlers::new()
                    .handler(http::StatusCode::INTERNAL_SERVER_ERROR, handler::render_500),
            )
            .wrap(common::middleware::LangSet::new(
                app_dao.clone().into_inner(),
            ))
            .wrap(middlewares::DefaultHeaders::new().add(("Access-Control-Allow-Origin", "*")))
            .wrap(common::middleware::RequestID::new(None))
            .app_data(app_dao.clone())
            .app_data(json_config)
            .app_data(jwt_config)
            .app_data(rest_config);
        router_main(app, &app_dao)
    });
    server = server.bind(bind_addr).unwrap();
    if let Some(ssl_addr) = bind_ssl_addr {
        server = server
            .bind_rustls(ssl_addr, load_rustls_config(app_dir))
            .unwrap();
    }
    server.run().await.unwrap();
}
