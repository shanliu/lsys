mod result;
pub use result::*;

use std::{collections::HashMap, path::Path};

use config::builder::DefaultState;

pub struct Config {
    def_config: config::Config,
    configs: HashMap<String, config::Config>,
}

impl Config {
    fn default_config<P: AsRef<Path>>(
        path: &P,
        app_config: &str,
    ) -> config::ConfigBuilder<DefaultState> {
        let mut config = config::Config::builder().add_source(config::Environment::default());
        let app_config_file = path
            .as_ref()
            .to_path_buf()
            .join(format!("./{}.toml", app_config));
        if app_config_file.is_file() {
            config = config.add_source(config::File::from(app_config_file))
        }
        config
    }
    pub async fn new<P: AsRef<Path>>(
        path: P,
        app_config: &str,
        crate_config: Option<&[&str]>,
    ) -> Result<Self, ConfigError> {
        let default_config = Self::default_config(&path, app_config).build()?;
        let mut crate_configs = HashMap::new();
        match crate_config {
            Some(config_name) => {
                for item in config_name {
                    let file_path = path.as_ref().to_path_buf().join(format!("./{}.toml", item));
                    let crate_config = Self::default_config(&path, app_config)
                        .add_source(config::File::from(file_path));
                    crate_configs.insert(item.to_string(), crate_config.build()?);
                }
            }
            None => match tokio::fs::read_dir(path.as_ref()).await {
                Ok(mut dir) => {
                    while let Some(fileentry) = dir.next_entry().await? {
                        if !fileentry.file_type().await?.is_file() {
                            continue;
                        }
                        let file_path = fileentry.path();
                        let file_path = file_path.as_path();
                        if file_path.extension().unwrap_or_default() != "toml" {
                            continue;
                        }
                        let file_name = if let Some(name) = file_path.file_stem() {
                            name.to_string_lossy().to_string()
                        } else {
                            continue;
                        };
                        if file_name.as_str() == app_config {
                            continue;
                        }
                        let crate_config = Self::default_config(&path, app_config)
                            .add_source(config::File::from(file_path));
                        crate_configs.insert(file_name, crate_config.build()?);
                    }
                }
                Err(err) => {
                    tracing::error!("fluent dir:{:?} on {:?}", err, path.as_ref());
                }
            },
        }
        Ok(Config {
            def_config: default_config,
            configs: crate_configs,
        })
    }
    pub fn find(&self, crate_name: Option<&str>) -> &config::Config {
        match crate_name {
            Some(name) => self.configs.get(name),
            None => Some(&self.def_config),
        }
        .unwrap_or(&self.def_config)
    }
}

#[macro_export]
macro_rules! config {
    ($config_mgr:expr) => {
        $config_mgr.find(Some(env!("CARGO_PKG_NAME")))
    };
}
