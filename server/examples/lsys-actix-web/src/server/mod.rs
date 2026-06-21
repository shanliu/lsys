
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
use lsys_web::lsys_core::task_lifecycle::TaskNode;
pub use result::AppError;
use rustls::load_rustls_config;
use std::sync::Arc;
use std::time::Duration;
use tracing::debug;

/// 应用服务器：持有 actix-web Server 和任务树根节点
///
/// 通过 [`create_server`](crate::create_server) 创建，调用 `.await` 运行：
/// ```ignore
/// let app = create_server(app_dir).await?;
/// app.await?;
/// ```
pub struct AppServer {
    server: Server,
    task_root: Arc<TaskNode>,
}

impl AppServer {
    /// 运行 server 直到收到 OS 信号停止，然后 drain 所有后台任务
    ///
    /// actix-web Server 自身监听 SIGINT/SIGTERM 并 graceful shutdown，
    /// HTTP 停止后再执行 task_root.shutdown() drain 后台任务。
    pub async fn run(self) -> Result<(), AppError> {
        self.server
            .await
            .map_err(|e| AppError::AppCore(AppCoreError::Io(e)))?;
        tracing::info!("HTTP server stopped, draining background tasks...");
        let report = self.task_root.shutdown().await;
        report.log_tree(0);
        let (completed, timed_out, panicked) = report.count_summary();
        tracing::info!(
            completed,
            timed_out,
            panicked,
            "shutdown drain completed"
        );
        Ok(())
    }
}

impl std::future::IntoFuture for AppServer {
    type Output = Result<(), AppError>;
    type IntoFuture = std::pin::Pin<Box<dyn std::future::Future<Output = Self::Output> + Send>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(self.run())
    }
}

pub async fn create_server(app_dir: &str) -> Result<AppServer, AppError> {
    let app_core = AppCore::new(app_dir, "config", "app", None, None).await?;
    init_tracing(&app_core).await?;
    let app_core = Arc::new(app_core);

    // 创建任务树根节点
    // 信号监听和 shutdown 顺序由应用层控制，task_lifecycle 只负责任务树管理
    let grace_period_secs = app_core
        .config
        .find(None)
        .get_int("task_shutdown_timeout")
        .unwrap_or(30) as u64;
    let task_root = lsys_web::lsys_core::task_lifecycle::TaskNode::root(
        "lsys-app",
        std::time::Duration::from_secs(grace_period_secs),
    );

    let app_dao = Data::new(WebDao::new(app_core.clone(), task_root.clone()).await.map_err(|e| {
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

    // 启动后台清理任务（使用 task_node 管理）
    let cleanup_guard = traffic_guard.clone();
    let traffic_node = task_root.child("traffic-guard-cleanup");
    traffic_node.spawn(|token| async move {
        loop {
            tokio::select! {
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(60)) => {
                    let task_id = lsys_web::lsys_core::utils::rand_str(lsys_web::lsys_core::utils::RandType::LowerHex, 8);
                    tracing::info_span!(
                        "background_task",
                        task = "traffic-guard-cleanup",
                        task_id = task_id
                    ).in_scope(|| cleanup_guard.cleanup());
                }
                _ = token.cancelled() => {
                    break;
                }
            }
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
            .wrap(traffic_guard)
            .wrap(RedirectSsl::new(is_use_ssl))
            .wrap(middlewares::Logger::default())
            .wrap(middlewares::Compress::default())
            .wrap(
                middlewares::ErrorHandlers::new()
                    .handler(http::StatusCode::INTERNAL_SERVER_ERROR, render_500),
            )
            .wrap(middlewares::DefaultHeaders::new().add(("X-Server-Name", "lsys")))
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

    Ok(AppServer {
        server: s,
        task_root,
    })
}