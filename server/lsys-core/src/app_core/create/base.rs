// use config::Config;
use deadpool_redis::{Config as RedisConfig, Runtime};

use log::LevelFilter;

use sqlx::pool::PoolOptions;
use sqlx::{ConnectOptions, Pool};
use tokio::sync::Mutex;

use std::str::FromStr;

use crate::app_core::result::AppCoreError;
use async_trait::async_trait;
use tera::Tera;

use crate::{AppCore, ConfigError, FluentMgr};

use super::AppCoreCreate;

use tracing_appender::non_blocking::WorkerGuard;

#[derive(Default)]
pub struct BaseAppCoreCreate {
    log_guard: Mutex<Option<WorkerGuard>>,
}

#[async_trait]
impl AppCoreCreate for BaseAppCoreCreate {
    async fn init_tracing(&self, app_core: &AppCore) -> Result<(), AppCoreError> {
        let log_level = app_core
            .config
            .find(None)
            .get_string("log_level")
            .unwrap_or_else(|_| String::from("info"));
        let log_max_level = tracing::Level::from_str(
            &app_core
                .config
                .find(None)
                .get_string("log_max_level")
                .unwrap_or_default(),
        )
        .unwrap_or(tracing::Level::TRACE);

        let sub = tracing_subscriber::fmt();

        let name = app_core
            .config
            .find(None)
            .get_string("log_name")
            .unwrap_or_default();
        if !name.is_empty() {
            match name.as_str() {
                "std::out" | "std::err" => {
                    let sub = sub
                        .compact() //是否隐藏参数
                        .with_ansi(true)
                        .pretty();
                    if name.as_str() == "std::out" {
                        sub.with_writer(std::io::stdout)
                            .with_max_level(log_max_level)
                            .with_env_filter(log_level)
                            .try_init()
                    } else {
                        sub.with_writer(std::io::stderr)
                            .with_max_level(log_max_level)
                            .with_env_filter(log_level)
                            .try_init()
                    }
                    .map_err(|e| AppCoreError::System(e.to_string()))?;
                }
                _ => {
                    let dir = app_core
                        .config
                        .find(None)
                        .get_string("log_dir")
                        .unwrap_or_else(|_| String::from("./"));

                    let file_appender = tracing_appender::rolling::daily(dir, name);
                    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
                    sub.with_ansi(false)
                        .with_writer(non_blocking)
                        .with_max_level(log_max_level)
                        .with_env_filter(log_level) //格式 模块:最大等级 mod:level
                        .try_init()
                        .map_err(|e| AppCoreError::System(e.to_string()))?;
                    *self.log_guard.lock().await = Some(guard);
                }
            };
        }
        Ok(())
    }
    async fn create_db(&self, app_core: &AppCore) -> Result<Pool<sqlx::MySql>, AppCoreError> {
        let database_url = app_core
            .config
            .find(None)
            .get_string("database_url")
            .unwrap_or_default();
        let database_level = app_core
            .config
            .find(None)
            .get_string("database_log_level")
            .unwrap_or_default();
        let database_max = app_core
            .config
            .find(None)
            .get_string("database_connect_max")
            .unwrap_or_default()
            .parse::<u32>()
            .unwrap_or(5);
        let mut option = sqlx::mysql::MySqlConnectOptions::from_str(&database_url)
            .map_err(|e| AppCoreError::System(e.to_string()))?;
        let level = LevelFilter::from_str(&database_level).unwrap_or(LevelFilter::Trace);
        option = option.log_statements(level);
        let poll = PoolOptions::<sqlx::MySql>::new()
            .max_connections(database_max)
            .connect_with(option)
            .await?;
        Ok(poll)
    }
    async fn create_redis_client(&self, app_core: &AppCore) -> Result<redis::Client, AppCoreError> {
        let redis_url = app_core
            .config
            .find(None)
            .get_string("redis_url")
            .unwrap_or_default();
        let b = redis::Client::open(redis_url)?;
        Ok(b)
    }
    async fn create_redis_pool(
        &self,
        app_core: &AppCore,
    ) -> Result<deadpool_redis::Pool, AppCoreError> {
        let redis_url = app_core
            .config
            .find(None)
            .get_string("redis_url")
            .unwrap_or_default();
        let cfg = RedisConfig::from_url(redis_url);
        let pool = cfg.create_pool(Some(Runtime::Tokio1))?;
        Ok(pool)
    }
    async fn create_tera(&self, app_core: &AppCore) -> Result<Tera, AppCoreError> {
        let mut tpl_dir = app_core
            .config
            .find(None)
            .get_string("tpl_dir")
            .map_err(|e| AppCoreError::Config(ConfigError::Config(e)))?;
        if !tpl_dir.ends_with('/') {
            tpl_dir += "/";
        }
        let tpl_exts = app_core
            .config
            .find(None)
            .get_string("tpl_exts")
            .map(|e| e.split(",").map(|e| e.to_owned()).collect::<Vec<String>>());

        if !tpl_dir.ends_with('/') {
            tpl_dir += "/";
        }
        let tpl_pat = &format!(
            "{}**/*.{{{}}}",
            tpl_dir,
            tpl_exts
                .unwrap_or(vec![
                    "txt".to_string(),
                    "html".to_string(),
                    "htm".to_string(),
                    "xml".to_string()
                ])
                .join(",")
        );
        let tera = Tera::new(tpl_pat)?;
        Ok(tera)
    }
    async fn create_fluent(&self, app_core: &AppCore) -> Result<FluentMgr, AppCoreError> {
        let path = { app_core.config_path(app_core.config.find(None), "fluent_dir")? };
        let fluent = FluentMgr::new(path, "app", None).await?;
        Ok(fluent)
    }
}
