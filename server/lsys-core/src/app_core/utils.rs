#[cfg(feature = "redis")]
use deadpool_redis::{Config as RedisConfig, Runtime};

#[cfg(feature = "db")]
use sqlx::pool::PoolOptions;
#[cfg(feature = "db")]
use sqlx::{ConnectOptions, Pool};

use std::str::FromStr;

use crate::app_core::result::AppCoreError;
use crate::app_core::AppCore;
use crate::fluents::FluentMgr;
#[cfg(feature = "tera")]
use crate::utils::{tera_second_format, tera_time_format};
#[cfg(feature = "tera")]
use tera::Tera;

/// 初始化追踪系统
pub async fn init_tracing(app_core: &AppCore) -> Result<(), AppCoreError> {
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
                .map_err(|e| AppCoreError::System(e.to_string()))?
            }
            _ => {
                let dir = app_core
                    .config
                    .find(None)
                    .get_string("log_dir")
                    .unwrap_or_else(|_| String::from("./"));

                let file_appender = tracing_appender::rolling::daily(dir, name);
                let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
                sub.with_ansi(false)
                    .with_writer(non_blocking)
                    .with_max_level(log_max_level)
                    .with_env_filter(log_level) //格式 模块:最大等级 mod:level
                    .try_init()
                    .map_err(|e| AppCoreError::System(e.to_string()))?;
                std::mem::forget(_guard);
            }
        }
    }
    Ok(())
}

/// 创建 Redis 客户端
#[cfg(feature = "redis")]
pub async fn create_redis_client(app_core: &AppCore) -> Result<redis::Client, AppCoreError> {
    let redis_url = app_core
        .config
        .find(None)
        .get_string("redis_url")
        .unwrap_or_default();
    let b = redis::Client::open(redis_url)?;
    Ok(b)
}

/// 创建 Redis 连接池
#[cfg(feature = "redis")]
pub async fn create_redis_pool(app_core: &AppCore) -> Result<deadpool_redis::Pool, AppCoreError> {
    let redis_url = app_core
        .config
        .find(None)
        .get_string("redis_url")
        .unwrap_or_default();
    let cfg = RedisConfig::from_url(redis_url);
    let pool = cfg.create_pool(Some(Runtime::Tokio1))?;
    Ok(pool)
}

/// 创建 Tera 模板引擎
#[cfg(feature = "tera")]
pub async fn create_tera(app_core: &AppCore) -> Result<Tera, AppCoreError> {
    let tpl_dir = app_core.config_path(app_core.config.find(None), "tpl_dir")?;
    let tpl_exts = app_core
        .config
        .find(None)
        .get_string("tpl_exts")
        .map(|e| e.split(",").map(|e| e.to_owned()).collect::<Vec<String>>());
    let tpl_pat = &format!(
        "{}**/*.{{{}}}",
        tpl_dir.to_string_lossy(),
        tpl_exts
            .unwrap_or(vec![
                "txt".to_string(),
                "html".to_string(),
                "htm".to_string(),
                "xml".to_string()
            ])
            .join(",")
    );
    tracing::debug!("tpl dir is:{}", tpl_pat);
    let mut tera = Tera::new(tpl_pat)?;
    tera.register_filter("second_format", tera_second_format);
    tera.register_filter("time_format", tera_time_format);
    Ok(tera)
}

/// 创建 Fluent 国际化管理器
pub async fn create_fluent(app_core: &AppCore) -> Result<FluentMgr, AppCoreError> {
    let path = { app_core.config_path(app_core.config.find(None), "fluent_dir")? };
    #[cfg(not(feature = "tokio"))]
    let fluent = FluentMgr::new(path, "lsys", None)?;
    #[cfg(feature = "tokio")]
    let fluent = FluentMgr::new(path, "lsys", None).await?;
    Ok(fluent)
}

/// 创建数据库连接池（支持 MySQL）
#[cfg(feature = "db-mysql")]
pub async fn create_mysql_pool(app_core: &AppCore) -> Result<Pool<sqlx::MySql>, AppCoreError> {
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
        .get_int("database_connect_max")
        .unwrap_or(5) as u32;
    let level = log::LevelFilter::from_str(&database_level).unwrap_or(log::LevelFilter::Trace);

    let mut option = sqlx::mysql::MySqlConnectOptions::from_str(&database_url)
        .map_err(|e| AppCoreError::System(e.to_string()))?;
    option = option.log_statements(level);
    let pool = PoolOptions::<sqlx::MySql>::new()
        .max_connections(database_max)
        .connect_with(option)
        .await?;
    Ok(pool)
}

/// 创建 Snowflake ID 生成器
pub fn create_snowflake_id_generator(app_core: &AppCore) -> snowflake::SnowflakeIdGenerator {
    let machine_id = app_core
        .config
        .find(None)
        .get_int("snowflake_machine_id")
        .unwrap_or(1);
    let machine_id = (machine_id.abs() % 31) as i32;
    let node_id = app_core
        .config
        .find(None)
        .get_int("snowflake_node_id")
        .unwrap_or_else(|_| {
            crc32fast::hash(
                hostname::get()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .as_bytes(),
            )
            .into()
        });
    let node_id = (node_id.abs() % 31) as i32;
    snowflake::SnowflakeIdGenerator::new(machine_id, node_id)
}
