use std::{net::SocketAddr, sync::Arc, time::Duration};

use axum::{
    routing::{get, post},
    Router,
};
use lsys_core::{db::TableName, AppCore, FluentMgr, RemoteNotify};
use lsys_logger::dao::ChangeLoggerDao;
use sqlx::MySql;
use tower_http::{cors::{Any, CorsLayer}, trace::TraceLayer};

use crate::auth::BarcodeClient;
use crate::dao::BarCodeDao;
use crate::handler;

pub struct AppState {
    pub app_core: Arc<AppCore>,
    #[allow(dead_code)]
    pub db: sqlx::Pool<MySql>,
    pub remote_notify: Arc<RemoteNotify>,
    #[allow(dead_code)]
    pub logger: Arc<ChangeLoggerDao>,
    pub barcode: Arc<BarCodeDao>,
    pub upstream: BarcodeClient,
    pub fluent: FluentMgr,
}

pub async fn run() -> Result<(), String> {
    // Reuse the same config layout as other examples.
    let app_dir = std::env::var("APP_DIR").unwrap_or_else(|_| "./".to_string());

    let app_core = AppCore::new(&app_dir, "config", None, None)
        .await
        .map_err(|e| format!("appcore init error: {e:?}"))?;
    let app_core = Arc::new(app_core);

    let db = app_core
        .create_db()
        .await
        .map_err(|e| format!("create db error: {e:?}"))?;

    let redis = app_core
        .create_redis()
        .await
        .map_err(|e| format!("create redis error: {e:?}"))?;

    let remote_notify = Arc::new(
        RemoteNotify::new("lsys-remote-notify-barcode", app_core.clone(), redis)
            .map_err(|e| format!("remote notify error: {e:?}"))?,
    );

    let logger = Arc::new(ChangeLoggerDao::new(db.clone()));

    let upstream_url = app_core
        .config
        .find(None)
        .get_string("service_url")
        .unwrap_or_else(|_| "https://lsys.cc/".to_string());

    let service_api_key = app_core
        .config
        .find(None)
        .get_string("service_api_key")
        .unwrap_or_else(|_| "".to_string());

    let upstream = crate::auth::BarcodeClient::from_base_url(&upstream_url, &service_api_key)
        .map_err(|e| format!("upstream client error: {e}"))?;

    let create_max = app_core
        .config
        .find(None)
        .get_int("barcode_create_max")
        .map(|e| if e > 0 { e as u64 } else { 0 })
        .unwrap_or(0);
    let use_cache = app_core
        .config
        .find(None)
        .get_bool("use_cache")
        .unwrap_or(false);

    let barcode = Arc::new(BarCodeDao::new(
        db.clone(),
        remote_notify.clone(),
        crate::dao::BarCodeConfig::new(create_max, use_cache),
        logger.clone(),
    ));

    // Initialize fluent for i18n
    let fluent_path = app_core
        .config_path(app_core.config.find(None), "fluent_dir")
        .map_err(|e| format!("fluent dir config error: {e:?}"))?;
    let fluent = FluentMgr::new(fluent_path, "app", None)
        .await
        .map_err(|e| format!("fluent init error: {e:?}"))?;

    let table_prefix = app_core
        .config
        .find(None)
        .get_string("database_table_prefix")
        .unwrap_or_default();
    TableName::set_prefix(table_prefix);

    // Build CORS layer from config
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

    let state = Arc::new(AppState {
        app_core,
        db,
        remote_notify,
        logger,
        barcode,
        upstream,
        fluent,
    });

    // routes
    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/rest/barcode", post(handler::rest_barcode))
        .route("/api/user/app_barcode/:op", post(handler::user_app_barcode))
        .route(
            "/barcode/:content_type/:code_id/:content_data",
            get(handler::public_show),
        )
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
        .unwrap_or(80) as u16;

    let addr: SocketAddr = format!("{host}:{port}")
        .parse()
        .map_err(|e| format!("bind addr error: {e}"))?;

    tokio::spawn({
        let remote_notify = state.remote_notify.clone();
        async move {
            remote_notify.listen().await;
        }
    });

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| format!("bind error: {e}"))?;

    tracing::info!("barcode http listening on {}", addr);
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
