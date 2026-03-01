use dotenv::dotenv;

use std::env;
use std::path::PathBuf;
use std::str::FromStr;

use super::result::AppCoreError;
use crate::config::Config;

pub struct AppCore {
    pub app_path: PathBuf,
    pub config: Config,
}

impl AppCore {
    pub async fn new(
        app_dir: &str,
        config_dir: &str,
        app_file: &str,
        crate_files: Option<&[&str]>,
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
        dbg!(&app_path);
        Ok(AppCore {
            app_path,
            config: Config::new(config_path, app_file, crate_files).await?,
        })
    }
    //根据配置获取文件路径
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
}
