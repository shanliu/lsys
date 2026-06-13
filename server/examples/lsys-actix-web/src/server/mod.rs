
mod result;
mod rustls;
use actix_web::dev::Server;
use actix_web::web::{Data, JsonConfig};
use actix_web::{HttpResponse, HttpServer, error, http, middleware as middlewares};

use crate::common::handler::RestQueryConfig;
use crate::common::handler::TokenSignConfig;
use crate::common::app::{build_fuse_rules, build_ip_throttle};
use crate::common::middleware::{TrafficGuard, RedirectSsl, RequestID};
use crate::handler::render_500;
use crate::handler::router;
use actix_cors::Cors;
use actix_web::App;
use futures_util::TryFutureExt;
use lsys_web::common::FluentFormat;
use lsys_web::dao::WebDao;
use lsys_web::lsys_core::app_core::utils::init_tracing;
use lsys_web::lsys_core::app_core::{AppCore, AppCoreError};
use lsys_web::lsys_core::fluents::IntoFluentMessage;
pub use result::AppError;
use rustls::load_rustls_config;
use std::sync::Arc;
use std::time::Duration;
use tracing::debug;

pub async fn create_server(app_dir: &str) -> Result<Server, AppError> {
    let app_core = AppCore::new(app_dir, "config", "app", None, None).await?;
    init_tracing(&app_core).await?;
    let app_core = Arc::new(app_core);
    let app_dao = Data::new(WebDao::new(app_core.clone()).await.map_err(|e| {
        AppError::AppCore(AppCoreError::System(e.to_fluent_message().default_format()))
    })?);
    let bind_addr = app_dao.bind_addr();
    let bind_ssl_data = app_dao.bind_ssl_data();
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
        .get_string("api_allow_origin")
    {
        Ok(v) => v.split(",").map(|o| o.trim().to_string()).collect(),
        Err(err) => {
            debug!("not set api_allow_origin: {}", err);
            vec![]
        }
    };

    // 登录 token 校验配置：启动时读取一次，注入后由各解析器从请求取出
    let token_config = TokenSignConfig::from_config(&app_dao);

    // 创建流量守护 - 标签熔断 + IP限流
    // 标签熔断：业务端通过 X-Fuse 响应头标记失败，中间件按规则匹配并独立计数
    // IP 限流：全局限流，防止单 IP 过载
    // 规则定义见 common/app/fuse_tags.rs
    let traffic_guard = TrafficGuard::builder()
        .fuse_rules(build_fuse_rules())
        .ip_throttle(build_ip_throttle())
        .build();

    // 启动后台清理任务
    let cleanup_guard = traffic_guard.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            cleanup_guard.cleanup();
        }
    });

    let is_use_ssl = bind_ssl_data.is_some();
    let mut server = HttpServer::new(move || {
        // 每个worker使用独立的熔断器实例（共享统计数据）
        let traffic_guard = traffic_guard.clone();
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
        let mut cors = Cors::default()
            .allow_any_header()
            .allowed_methods(vec!["GET", "POST", "OPTIONS"])
            .expose_headers(vec![
                "Content-Disposition",
                "Content-Type",
                "Content-Length",
            ])
            .max_age(3600);

        if origin_list.iter().any(|o| o == "*") || origin_list.is_empty() {
            cors = cors.allow_any_origin();
        } else {
            for origin in &origin_list {
                cors = cors.allowed_origin(origin);
            }
        }

        let app = App::new()
            // 流量守护中间件 - 熔断+限流，防止恶意攻击（放在最外层，最先拦截）
            .wrap(traffic_guard)
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
            .wrap(cors)
            .app_data(app_dao.clone())
            .app_data(json_config)
            .app_data(rest_config)
            .app_data(token_config.clone());
        router(app, &app_dao)
    });
    server = server.bind(bind_addr).map_err(AppCoreError::Io)?;
    if let Some((ssl_addr, cert_file, key_file)) = bind_ssl_data {
        let ssl_data = load_rustls_config(app_dir, &cert_file, &key_file)?;
        server = server
            .bind_rustls_0_23(ssl_addr, ssl_data)
            .map_err(AppCoreError::Io)?;
    }
    let s = server
        .keep_alive(Duration::from_secs(75)) // Keep-Alive 超时
        .client_request_timeout(Duration::from_secs(60)) // 客户端请求超时
        .client_disconnect_timeout(Duration::from_secs(10)) // 客户端关闭超时
        .run();
    Ok(s)
}