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

mod common;
mod handler;

#[actix_web::main]
async fn main() {
    let app_dir = if PathBuf::from_str("./").unwrap().join("./config").exists() {
        "./"
    } else {
        //on dev
        env!("CARGO_MANIFEST_DIR")
    };
    let app_core = Arc::new(
        AppCore::init(
            app_dir,
            &[
                "config/app.toml",
                "config/mail.toml",
                "config/sms.toml",
                "config/oauth.toml",
            ],
        )
        .await
        .unwrap(),
    );
    let app_dao = Data::new(WebDao::new(app_core.clone()).await.unwrap());
    let bind_addr = app_dao.bind_addr();
    let server = HttpServer::new(move || {
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
            .wrap(crate::common::middleware::LangSet::new(
                app_dao.clone().into_inner(),
            ))
            .wrap(middlewares::DefaultHeaders::new().add(("Access-Control-Allow-Origin", "*")))
            .wrap(crate::common::middleware::RequestID::new(None))
            .app_data(app_dao.clone())
            .app_data(json_config)
            .app_data(jwt_config)
            .app_data(rest_config);
        router_main(app, &app_dao)
    });
    server.bind(bind_addr).unwrap().run().await.unwrap();
}
