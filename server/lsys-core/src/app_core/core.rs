use dotenv::dotenv;

use std::env;
use std::path::PathBuf;
use std::str::FromStr;

use super::env::AppEnv;
use super::result::AppCoreError;
use crate::config::Config;

pub struct AppCore {
    pub app_path: PathBuf,
    pub config: Config,
    pub env: AppEnv,
}

impl AppCore {
    pub async fn new(
        app_dir: &str,
        config_dir: &str,
        app_file: &str,
        crate_files: Option<&[&str]>,
        env: Option<AppEnv>,
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
        
        let config = Config::new(config_path, app_file, crate_files).await?;
        
        // 环境优先级：传入参数 > 配置文件 app_env/env（自动包含环境变量 APP_ENV/ENV） > 默认值
        let env = env
            .or_else(|| {
                config
                    .find(None)
                    .get_string("app_env")
                    .or_else(|_| config.find(None).get_string("env"))
                    .ok()
                    .and_then(|s| s.parse().ok())
            })
            .unwrap_or_default();
        
        log::debug!("app_path: {:?}", &app_path);
        log::info!("Running in {} environment", env);
        Ok(AppCore {
            app_path,
            config,
            env,
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
