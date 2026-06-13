use std::{net::SocketAddr, sync::Arc, time::Duration};

use axum::{
    Router,
    routing::{get, post},
};
use lsys_core::app_core::AppCore;
use lsys_sdk::ServiceClient;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use crate::handler;

/// 应用共享状态
pub struct AppState {
    pub app_core: Arc<AppCore>,
    /// SDK 客户端（服务间 HTTP 通信）
    pub upstream: ServiceClient,
}

pub async fn run() -> Result<(), String> {
    let app_dir = std::env::var("APP_DIR").unwrap_or_else(|_| "./".to_string());

    let app_core = AppCore::new(&app_dir, "config", "app-file-demo", None, None)
        .await
        .map_err(|e| format!("appcore init error: {e:?}"))?;
    let app_core = Arc::new(app_core);

    // 读取上游服务配置
    let upstream_url = app_core
        .config
        .find(None)
        .get_string("service_url")
        .unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());

    let service_api_key = app_core
        .config
        .find(None)
        .get_string("service_api_key")
        .unwrap_or_else(|_| "".to_string());

    // 创建 SDK 客户端
    let upstream = ServiceClient::new(&upstream_url, &service_api_key)
        .map_err(|e| format!("upstream client error: {e}"))?;

    // CORS 配置
    let cors_layer = {
        let allow_origin = app_core
            .config
            .find(None)
            .get_string("api_allow_origin")
            .unwrap_or_else(|_| "*".to_string());

        if allow_origin == "*" {
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
        } else {
            let origins: Vec<_> = allow_origin
                .split(',')
                .filter_map(|s| s.trim().parse().ok())
                .collect();
            CorsLayer::new()
                .allow_origin(origins)
                .allow_methods(Any)
                .allow_headers(Any)
        }
    };

    let state = Arc::new(AppState { app_core, upstream });

    // 路由
    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        // 示例接口：演示所有文件 SDK 操作
        .route("/demo/upload_create", post(handler::demo_upload_create))
        .route("/demo/upload_retoken", post(handler::demo_upload_retoken))
        .route("/demo/upload_by_md5", post(handler::demo_upload_by_md5))
        .route("/demo/from_url", post(handler::demo_from_url))
        .route("/demo/from_local", post(handler::demo_from_local))
        .route("/demo/file_list", post(handler::demo_file_list))
        .route("/demo/file_delete", post(handler::demo_file_delete))
        .route("/demo/file_urls", post(handler::demo_file_urls))
        .route("/demo/file_info", post(handler::demo_file_info))
        .route("/demo/mapping", get(handler::demo_mapping))
        .layer(TraceLayer::new_for_http())
        .layer(cors_layer)
        .with_state(state.clone());

    let host = state
        .app_core
        .config
        .find(None)
        .get_string("app_host")
        .unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = state
        .app_core
        .config
        .find(None)
        .get_int("app_port")
        .unwrap_or(8090) as u16;

    let addr: SocketAddr = format!("{host}:{port}")
        .parse()
        .map_err(|e| format!("bind addr error: {e}"))?;

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| format!("bind error: {e}"))?;

    tracing::info!("file-demo http listening on {}", addr);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| format!("serve error: {e}"))?;

    Ok(())
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
    tokio::time::sleep(Duration::from_millis(100)).await;
}
