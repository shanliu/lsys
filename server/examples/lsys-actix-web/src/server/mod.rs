mod result;
mod rustls;
use actix_web::dev::Server;
use actix_web::web::{Data, JsonConfig};
use actix_web::{error, http, middleware as middlewares, HttpResponse, HttpServer};

use actix_web::App;
use futures_util::TryFutureExt;
use jsonwebtoken::{DecodingKey, Validation};
use lsys_web::common::FluentFormat;
use lsys_web::dao::WebDao;
use lsys_web::lsys_core::{AppCore, AppCoreError};
use std::sync::Arc;
use tracing::debug;

use crate::common::handler::{JwtQueryConfig, RestQueryConfig};
use crate::common::middleware::{AllowOrigin, RedirectSsl, RequestID};
use crate::handler::render_500;
use crate::handler::router;
use result::AppError;
use rustls::load_rustls_config;

pub async fn create_server(app_dir: &str) -> Result<Server, AppError> {
    let app_core = AppCore::new(app_dir, "config", None, None).await?;
    app_core.init().await?;
    let app_core = Arc::new(app_core);
    let app_dao = Data::new(WebDao::new(app_core.clone()).await?);
    let bind_addr = app_dao.bind_addr();
    let bind_ssl_data = app_dao.bind_ssl_data();
    let app_jwt_key = app_dao
        .app_core
        .config
        .find(None)
        .get_string("app_jwt_key")
        .map_err(|err| AppCoreError::Config(lsys_web::lsys_core::ConfigError::Config(err)))?;
    let app_json_limit = app_dao
        .app_core
        .config
        .find(None)
        .get_int("app_json_limit")
        .unwrap_or(4096);

    let origin_list = match app_dao
        .app_core
        .config
        .find(None)
        .get_string("api-allow-origin")
    {
        Ok(v) => v.split(",").map(|o| o.trim().to_string()).collect(),
        Err(err) => {
            debug!("not set api-allow-origin: {}", err);
            vec![]
        }
    };

    let is_use_ssl = bind_ssl_data.is_some();
    let mut server = HttpServer::new(move || {
        let jwt_config = JwtQueryConfig::new(
            DecodingKey::from_secret(app_jwt_key.as_bytes()),
            Validation::default(),
        );
        let json_config = JsonConfig::default()
            .limit(app_json_limit as usize)
            .error_handler(|err, _req| {
                error::InternalError::from_response(err, HttpResponse::Conflict().finish()).into()
            });
        let rest_config =
            RestQueryConfig::default().app_key_fn(Box::new(move |app_key, app_data| {
                let apps = app_data.web_app.app_dao.app.clone();
                Box::pin(async move {
                    apps.cache()
                        .find_app_secret_by_client_id(&app_key)
                        .map_err(|e| e.fluent_format(&app_data.fluent.locale(None)))
                        .await
                        .map(|e| {
                            e.into_iter()
                                .map(|e| e.secret_data)
                                .collect::<Vec<String>>()
                        })
                })
            }));
        //  use tokio::time::{sleep, Duration};
        // use actix_web::middleware::Next;
        // use actix_web::{
        //     body::MessageBody,
        //     dev::{ServiceRequest, ServiceResponse},
        //     http::header::Accept,
        //     web::{Header, Query},
        //      Error,
        // };
        // use std::collections::HashMap;
        // async fn my_extracting_mw(
        //     _: Header<Accept>,
        //     _: Query<HashMap<String, String>>,
        //     req: ServiceRequest,
        //     next: Next<impl MessageBody>,
        // ) -> Result<ServiceResponse<impl MessageBody>, Error> {
        //     sleep(Duration::from_secs(1)).await;
        //     // 继续处理请求
        //     next.call(req).await
        //     // post-processing
        // }
        let app = App::new()
            .wrap(RedirectSsl::new(is_use_ssl))
            .wrap(middlewares::Logger::default())
            .wrap(middlewares::Compress::default())
            .wrap(
                middlewares::ErrorHandlers::new()
                    .handler(http::StatusCode::INTERNAL_SERVER_ERROR, render_500),
            )
            .wrap(middlewares::DefaultHeaders::new().add(("X-Server-Name", "lsys")))
            //.wrap(middlewares::from_fn(my_extracting_mw))
            .wrap(RequestID::new(None))
            .wrap(AllowOrigin(origin_list.clone()))
            .app_data(app_dao.clone())
            .app_data(json_config)
            .app_data(jwt_config)
            .app_data(rest_config);
        router(app, &app_dao)
    });
    server = server.bind(bind_addr).map_err(AppCoreError::Io)?;
    if let Some((ssl_addr, cert_file, key_file)) = bind_ssl_data {
        let ssl_data = load_rustls_config(app_dir, &cert_file, &key_file)?;
        server = server
            .bind_rustls_0_23(ssl_addr, ssl_data)
            .map_err(AppCoreError::Io)?;
    }
    let s = server.run();
    Ok(s)
}
