use config::Config;
use dotenv::dotenv;
use fluent::{bundle::FluentBundle, FluentResource};
use intl_memoizer::concurrent::IntlLangMemoizer;
use log::LevelFilter;
use redis::aio::ConnectionManager;
use redis::RedisError;
use sqlx::pool::PoolOptions;
use sqlx::{ConnectOptions, Connection, Database, Pool};
use std::env::{self, VarError};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use unic_langid::LanguageIdentifier;

use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tera::Tera;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

use sqlx_model::TableName;

use crate::fluent_message::FluentMessage;

#[derive(Debug)]
pub enum AppCoreError {
    Sqlx(sqlx::Error),
    Env(VarError),
    Tera(tera::Error),
    Io(std::io::Error),
    System(String),
    Log(String),
    Redis(RedisError),
    Dotenv(dotenv::Error),
    AppDir(String),
    Config(config::ConfigError),
}

impl Display for AppCoreError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl Error for AppCoreError {}
impl From<sqlx::Error> for AppCoreError {
    fn from(err: sqlx::Error) -> Self {
        AppCoreError::Sqlx(err)
    }
}
impl From<VarError> for AppCoreError {
    fn from(err: VarError) -> Self {
        AppCoreError::Env(err)
    }
}
impl From<tera::Error> for AppCoreError {
    fn from(err: tera::Error) -> Self {
        AppCoreError::Tera(err)
    }
}
impl From<std::io::Error> for AppCoreError {
    fn from(err: std::io::Error) -> Self {
        AppCoreError::Io(err)
    }
}
impl From<RedisError> for AppCoreError {
    fn from(err: RedisError) -> Self {
        AppCoreError::Redis(err)
    }
}
impl From<dotenv::Error> for AppCoreError {
    fn from(err: dotenv::Error) -> Self {
        AppCoreError::Dotenv(err)
    }
}
impl From<core::convert::Infallible> for AppCoreError {
    fn from(err: core::convert::Infallible) -> Self {
        AppCoreError::AppDir(err.to_string())
    }
}
impl From<config::ConfigError> for AppCoreError {
    fn from(err: config::ConfigError) -> Self {
        AppCoreError::Config(err)
    }
}

pub struct AppCore {
    pub app_dir: PathBuf,
    pub config: Config,
}

pub type AppCoreResult = Result<AppCore, AppCoreError>;

impl AppCore {
    pub async fn init(dir: &str, config_files: &[&str]) -> AppCoreResult {
        let mut app_dir = PathBuf::from_str(dir)?;
        if !app_dir.is_absolute() {
            app_dir = env::current_dir()?.join(dir.trim_start_matches("./"));
        }
        if !app_dir.join("config").exists() {
            return Err(AppCoreError::AppDir(format!(
                "not find config dir in : {}",
                dir,
            )));
        }
        if app_dir.join(".env").exists() {
            dotenv::from_path(app_dir.join(".env"))?;
        } else {
            dotenv().ok();
        }
        let mut app_config = Config::builder();
        for file in config_files {
            app_config = app_config.add_source(config::File::from(app_dir.join(file)))
        }
        let config = app_config
            .add_source(config::Environment::default())
            .build()?;
        Ok(AppCore { app_dir, config })
    }
    // pub fn init_gelf() {
    //     //输出格式 span{args=3}:span{args=3}: mod::mod: message
    //     let address: Result<std::net::SocketAddr, _> = app_config
    //         .get_string("log_gelf")
    //         .unwrap_or_default()
    //         .parse();
    //     if address.is_ok() {
    //         tracing_log::LogTracer::builder()
    //             .with_max_level(log::LevelFilter::Debug)
    //             .init()
    //             .unwrap();
    //         let bg_task = tracing_gelf::Logger::builder()
    //             .init_tcp(address.unwrap_or_default())
    //             .unwrap();
    //         //actix_web::rt::spawn(bg_task);
    //     }
    // }
    pub fn init_tracing(&self) -> Result<(), AppCoreError> {
        let log_level = self
            .config
            .get_string("log_level")
            .unwrap_or_else(|_| String::from("info"));
        let log_max_level =
            tracing::Level::from_str(&self.config.get_string("log_max_level").unwrap_or_default())
                .unwrap_or(tracing::Level::TRACE);

        let sub = tracing_subscriber::fmt()
        // .compact() //是否隐藏参数
        // .pretty()
        ;

        let dir = self
            .config
            .get_string("log_dir")
            .unwrap_or_else(|_| String::from("./"));
        let name = self
            .config
            .get_string("log_name")
            .unwrap_or_else(|_| String::from("std::out"));
        if !name.is_empty() {
            let write = match name.as_str() {
                "std::out" => {
                    sub.with_writer(std::io::stdout)
                        .with_max_level(log_max_level)
                        .with_env_filter(log_level) //格式 模块:最大等级 mod:level
                        .try_init()
                }
                "std::err" => {
                    sub.with_writer(std::io::stderr)
                        .with_max_level(log_max_level)
                        .with_env_filter(log_level) //格式 模块:最大等级 mod:level
                        .try_init()
                }
                _ => {
                    let file_appender = tracing_appender::rolling::hourly(dir, name);
                    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
                    sub.with_writer(non_blocking)
                        .with_max_level(log_max_level)
                        //.with_env_filter(EnvFilter::from_default_env().add_directive("echo=trace".parse()?))//手动分开配置方式
                        // 基于span过滤 target[span{field=value}]=level
                        .with_env_filter(log_level) //格式 模块:最大等级 mod:level
                        .try_init()
                }
            };
            write //只能有一个writer
                .map_err(|e| AppCoreError::Log(e.to_string()))?;
        }

        Ok(())
    }
    pub async fn create_db<DB: Database>(&self) -> Result<Pool<DB>, AppCoreError> {
        let table_prefix = self
            .config
            .get_string("database_table_prefix")
            .unwrap_or_default();
        let database_url = self.config.get_string("database_url").unwrap_or_default();
        let database_level = self
            .config
            .get_string("database_log_level")
            .unwrap_or_default();
        let database_max = self
            .config
            .get_string("database_connect_max")
            .unwrap_or_default()
            .parse::<u32>()
            .unwrap_or(5);
        TableName::set_prefix(table_prefix);
        let mut option = <DB::Connection as Connection>::Options::from_str(&database_url)
            .map_err(|e| AppCoreError::System(e.to_string()))?;
        let level = LevelFilter::from_str(&database_level).unwrap_or(LevelFilter::Trace);
        option.log_statements(level);
        let poll = PoolOptions::<DB>::new()
            .max_connections(database_max)
            .connect_with(option)
            .await?;
        Ok(poll)
    }
    pub fn create_redis_client(&self) -> Result<redis::Client, AppCoreError> {
        let redis_url = self.config.get_string("redis_url").unwrap_or_default();
        let b = redis::Client::open(redis_url)?;
        Ok(b)
    }
    pub async fn create_redis(&self) -> Result<ConnectionManager, AppCoreError> {
        let client = self.create_redis_client()?;
        let con = client.get_tokio_connection_manager().await?;
        Ok(con)
    }
    pub fn create_tera(&self, tpl_dir: &str) -> Result<Tera, AppCoreError> {
        let mut tpl_exts = self
            .config
            .get_array("tpl_ext")
            .unwrap_or_default()
            .into_iter()
            .map(|e| e.into_string().unwrap_or_default())
            .filter(|e| e.is_empty())
            .collect::<Vec<String>>();
        if tpl_exts.is_empty() {
            tpl_exts = vec!["txt", "html", "htm", "xml"]
                .into_iter()
                .map(|e| e.to_owned())
                .collect();
        }
        let tpl_pat = &format!("{}/**/*.{{{}}}", tpl_dir, tpl_exts.join(","));
        let tera = Tera::new(tpl_pat)?;
        Ok(tera)
    }

    async fn init_fluent<P: AsRef<Path>>(
        path: P,
    ) -> Result<HashMap<String, FluentBundle<FluentResource, IntlLangMemoizer>>, AppCoreError> {
        let mut fluents = HashMap::new();
        let path = path.as_ref();
        match tokio::fs::read_dir(path).await {
            Ok(mut dir) => {
                while let Some(entry) = dir.next_entry().await? {
                    let ftype = entry.file_type().await;
                    if ftype.is_err() || !ftype?.is_dir() {
                        continue;
                    }
                    let lang = entry.file_name();
                    #[allow(unused_assignments)]
                    let mut lang_den = LanguageIdentifier::default();
                    let lang_str = lang.clone().into_string().unwrap_or_default();
                    match LanguageIdentifier::from_str(lang_str.as_str()) {
                        Err(_) => continue,
                        Ok(_lang_den) => {
                            lang_den = _lang_den;
                        }
                    }
                    let _path = path.to_path_buf().join(lang);
                    let mut sdir = tokio::fs::read_dir(_path.as_path()).await?;
                    while let Some(fileentry) = sdir.next_entry().await? {
                        if !fileentry.file_type().await?.is_file() {
                            continue;
                        }
                        let file_path = fileentry.path();
                        let file_path = file_path.as_path();
                        if file_path.extension().unwrap_or_default() != "ftl" {
                            continue;
                        }
                        let mut f = File::open(file_path).await?;
                        let mut buffer = Vec::new();
                        // read the whole file
                        f.read_to_end(&mut buffer).await?;
                        let ftl_string = String::from_utf8(buffer).unwrap_or_default();
                        let res = FluentResource::try_new(ftl_string).map_err(|_| {
                            AppCoreError::System("parse ftl data error".to_string())
                        })?;
                        let mut bundle = FluentBundle::new_concurrent(vec![lang_den.clone()]);
                        if let Err(err) = bundle.add_resource(res) {
                            tracing::error!("fluent add res:{:?}", err);
                        }
                        fluents.insert(lang_str.clone(), bundle);
                    }
                }
            }
            Err(err) => {
                tracing::error!("fluent dir:{:?} on {:?}", err, path);
            }
        }
        Ok(fluents)
    }
    pub async fn create_fluent<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<FluentMessage, AppCoreError> {
        let fluents = Self::init_fluent(path).await?;
        let lang = self
            .config
            .get_string("app_lang")
            .unwrap_or_else(|_| String::from("en-US"));
        let langid_en = LanguageIdentifier::from_str(&lang).unwrap_or_default();
        let bundle = FluentBundle::new_concurrent(vec![langid_en]);
        Ok(FluentMessage {
            fluent_key: RwLock::new(lang),
            fluents: RwLock::new(fluents),
            fluent_def: RwLock::new(bundle),
        })
    }
}
