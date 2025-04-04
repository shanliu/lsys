use dotenv::dotenv;

use crate::AppCoreCreate;
use sqlx::Pool;
use std::env;
use std::path::PathBuf;
use std::str::FromStr;
use tera::Tera;

use crate::db::TableName;

use super::result::AppCoreError;
use crate::BaseAppCoreCreate;
use crate::Config;

pub struct AppCore {
    pub app_path: PathBuf,
    pub config: Config,
    create: Box<dyn AppCoreCreate>,
}

impl AppCore {
    pub async fn init(
        app_dir: &str,
        config_dir: &str,
        config_files: Option<&[&str]>,
        create: Option<Box<dyn AppCoreCreate>>,
    ) -> Result<AppCore, AppCoreError> {
        let mut app_path = PathBuf::from_str(app_dir)
            .map_err(|e| AppCoreError::AppDir(format!("app dir [{}] error: {}", app_dir, e)))?;
        if !app_path.is_absolute() {
            app_path = env::current_dir()?.join(app_dir.trim_start_matches("./"));
        }
        if app_path.join(".env").exists() {
            dotenv::from_path(app_path.join(".env"))?;
        } else {
            dotenv().ok();
        }
        let mut config_path = PathBuf::from_str(config_dir)
            .map_err(|e| AppCoreError::AppDir(format!("config dir [{}] error: {}", app_dir, e)))?;
        if !config_path.is_absolute() {
            config_path = app_path.join(config_dir.trim_start_matches("./"));
        }
        if !app_path.join(config_dir).exists() {
            return Err(AppCoreError::AppDir(format!(
                "not find config dir in : {}",
                app_dir,
            )));
        }
        Ok(AppCore {
            app_path,
            create: match create {
                Some(val) => val,
                None => Box::new(BaseAppCoreCreate::default()),
            },
            config: Config::new(config_path, "app", config_files).await?,
        })
    }
    pub fn config_path(
        &self,
        config: &config::Config,
        config_key: &str,
    ) -> Result<PathBuf, AppCoreError> {
        let path = config.get_string(config_key).map(PathBuf::from)?;
        if path.is_absolute() {
            return Ok(path);
        }
        Ok(self.app_path.join(path))
    }
    pub async fn init_tracing(&self) -> Result<(), AppCoreError> {
        self.create.init_tracing(self).await
    }
    pub async fn create_db(&self) -> Result<Pool<sqlx::MySql>, AppCoreError> {
        let table_prefix = self
            .config
            .find(None)
            .get_string("database_table_prefix")
            .unwrap_or_default();
        TableName::set_prefix(table_prefix);
        let poll = self.create.create_db(self).await?;
        Ok(poll)
    }
    pub fn create_snowflake_id_generator(&self) -> snowflake::SnowflakeIdGenerator {
        let machine_id = self
            .config
            .find(None)
            .get_int("snowflake_machine_id")
            .unwrap_or(1);
        let machine_id = (machine_id.abs() % 31) as i32;
        let node_id = self
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
    pub async fn create_redis_client(&self) -> Result<redis::Client, AppCoreError> {
        self.create.create_redis_client(self).await
    }
    pub async fn create_redis(&self) -> Result<deadpool_redis::Pool, AppCoreError> {
        self.create.create_redis_pool(self).await
    }
    pub async fn create_tera(&self) -> Result<Tera, AppCoreError> {
        self.create.create_tera(self).await
    }
}
