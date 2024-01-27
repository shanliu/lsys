// use config::Config;
use deadpool_redis::{Config as RedisConfig, CreatePoolError, Runtime};
use dotenv::dotenv;

use log::LevelFilter;
use redis::RedisError;
use sqlx::pool::PoolOptions;
use sqlx::{ConnectOptions, Connection, Database, Pool};
use std::env::{self, VarError};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use std::path::{Path, PathBuf};
use tera::Tera;

use sqlx_model::TableName;

use crate::{Config, ConfigError, FluentBundleError, RemoteNotifyError};

#[derive(Debug)]
pub enum AppCoreError {
    Sqlx(sqlx::Error),
    Env(VarError),
    Tera(tera::Error),
    Io(std::io::Error),
    System(String),
    Log(String),
    Redis(RedisError),
    RedisPool(CreatePoolError),
    Dotenv(dotenv::Error),
    AppDir(String),
    Config(ConfigError),
    Fluent(FluentBundleError),
    RemoteNotify(RemoteNotifyError),
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
impl From<CreatePoolError> for AppCoreError {
    fn from(err: CreatePoolError) -> Self {
        AppCoreError::RedisPool(err)
    }
}
impl From<RemoteNotifyError> for AppCoreError {
    fn from(err: RemoteNotifyError) -> Self {
        AppCoreError::RemoteNotify(err)
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
// impl From<core::convert::Infallible> for AppCoreError {
//     fn from(err: core::convert::Infallible) -> Self {
//         AppCoreError::AppDir(err.to_string())
//     }
// }
impl From<ConfigError> for AppCoreError {
    fn from(err: ConfigError) -> Self {
        AppCoreError::Config(err)
    }
}
impl From<config::ConfigError> for AppCoreError {
    fn from(err: config::ConfigError) -> Self {
        AppCoreError::Config(ConfigError::Config(err))
    }
}
impl From<FluentBundleError> for AppCoreError {
    fn from(err: FluentBundleError) -> Self {
        AppCoreError::Fluent(err)
    }
}

pub struct AppCore {
    pub app_path: PathBuf,
    pub config: Config,
}

pub type AppCoreResult = Result<AppCore, AppCoreError>;

impl AppCore {
    pub async fn init(
        app_dir: &str,
        config_dir: &str,
        config_files: Option<&[&str]>,
    ) -> AppCoreResult {
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
            config: Config::new(config_path, "app", config_files).await?,
        })
    }
    pub fn app_path<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        if !path.as_ref().is_absolute() {
            return self.app_path.join(path);
        }
        path.as_ref().to_path_buf()
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
            .find(None)
            .get_string("log_level")
            .unwrap_or_else(|_| String::from("info"));
        let log_max_level = tracing::Level::from_str(
            &self
                .config
                .find(None)
                .get_string("log_max_level")
                .unwrap_or_default(),
        )
        .unwrap_or(tracing::Level::TRACE);

        let sub = tracing_subscriber::fmt()
        // .compact() //是否隐藏参数
        // .pretty()
        ;

        let dir = self
            .config
            .find(None)
            .get_string("log_dir")
            .unwrap_or_else(|_| String::from("./"));
        let name = self
            .config
            .find(None)
            .get_string("log_name")
            .unwrap_or_else(|_| String::from("std::out"));
        if !name.is_empty() {
            match name.as_str() {
                "std::out" => {
                    sub.with_writer(std::io::stdout)
                        .with_max_level(log_max_level)
                        .with_env_filter(log_level) //格式 模块:最大等级 mod:level
                        .try_init()
                        .ok()
                }
                "std::err" => {
                    sub.with_writer(std::io::stderr)
                        .with_max_level(log_max_level)
                        .with_env_filter(log_level) //格式 模块:最大等级 mod:level
                        .try_init()
                        .ok()
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
                        .ok()
                }
            };
            // write //只能有一个writer
            //     .map_err(|e| AppCoreError::Log(e.to_string()));
        }

        Ok(())
    }
    pub async fn create_db<DB: Database>(&self) -> Result<Pool<DB>, AppCoreError> {
        let table_prefix = self
            .config
            .find(None)
            .get_string("database_table_prefix")
            .unwrap_or_default();
        let database_url = self
            .config
            .find(None)
            .get_string("database_url")
            .unwrap_or_default();
        let database_level = self
            .config
            .find(None)
            .get_string("database_log_level")
            .unwrap_or_default();
        let database_max = self
            .config
            .find(None)
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
    pub fn create_redis_client(&self) -> Result<redis::Client, AppCoreError> {
        let redis_url = self
            .config
            .find(None)
            .get_string("redis_url")
            .unwrap_or_default();
        let b = redis::Client::open(redis_url)?;
        Ok(b)
    }
    pub async fn create_redis(&self) -> Result<deadpool_redis::Pool, AppCoreError> {
        let redis_url = self
            .config
            .find(None)
            .get_string("redis_url")
            .unwrap_or_default();
        let cfg = RedisConfig::from_url(redis_url);
        let pool = cfg.create_pool(Some(Runtime::Tokio1))?;
        Ok(pool)
    }
    pub fn create_tera(&self, tpl_exts: Option<&[&'static str]>) -> Result<Tera, AppCoreError> {
        let mut tpl_dir = self
            .config
            .find(None)
            .get_string("tpl_dir")
            .map_err(|e| AppCoreError::Config(ConfigError::Config(e)))?;
        if !tpl_dir.ends_with('/') {
            tpl_dir += "/";
        }
        let tpl_pat = &format!(
            "{}**/*.{{{}}}",
            tpl_dir,
            tpl_exts.unwrap_or(&["txt", "html", "htm", "xml"]).join(",")
        );
        let tera = Tera::new(tpl_pat)?;
        Ok(tera)
    }
    // async fn init_fluent<P: AsRef<Path>>(
    //     path: P,
    // ) -> Result<HashMap<String, FluentBundle<FluentResource, IntlLangMemoizer>>, AppCoreError> {
    //     let mut fluents = HashMap::new();
    //     let path = path.as_ref();
    //     match tokio::fs::read_dir(path).await {
    //         Ok(mut dir) => {
    //             while let Some(entry) = dir.next_entry().await? {
    //                 let ftype = entry.file_type().await;
    //                 if ftype.is_err() || !ftype?.is_dir() {
    //                     continue;
    //                 }
    //                 let lang = entry.file_name();
    //                 #[allow(unused_assignments)]
    //                 let mut lang_den = LanguageIdentifier::default();
    //                 let lang_str = lang.clone().into_string().unwrap_or_default();
    //                 match LanguageIdentifier::from_str(lang_str.as_str()) {
    //                     Err(_) => continue,
    //                     Ok(_lang_den) => {
    //                         lang_den = _lang_den;
    //                     }
    //                 }
    //                 let _path = path.to_path_buf().join(lang);
    //                 let mut sdir = tokio::fs::read_dir(_path.as_path()).await?;
    //                 while let Some(fileentry) = sdir.next_entry().await? {
    //                     if !fileentry.file_type().await?.is_file() {
    //                         continue;
    //                     }
    //                     let file_path = fileentry.path();
    //                     let file_path = file_path.as_path();
    //                     if file_path.extension().unwrap_or_default() != "ftl" {
    //                         continue;
    //                     }
    //                     let mut f = File::open(file_path).await?;
    //                     let mut buffer = Vec::new();
    //                     // read the whole file
    //                     f.read_to_end(&mut buffer).await?;
    //                     let ftl_string = String::from_utf8(buffer).unwrap_or_default();
    //                     let res = FluentResource::try_new(ftl_string).map_err(|_| {
    //                         AppCoreError::System("parse ftl data error".to_string())
    //                     })?;
    //                     let mut bundle = FluentBundle::new_concurrent(vec![lang_den.clone()]);
    //                     if let Err(err) = bundle.add_resource(res) {
    //                         tracing::error!("fluent add res:{:?}", err);
    //                     }
    //                     fluents.insert(lang_str.clone(), bundle);
    //                 }
    //             }
    //         }
    //         Err(err) => {
    //             tracing::error!("fluent dir:{:?} on {:?}", err, path);
    //         }
    //     }
    //     Ok(fluents)
    // }
    // pub async fn create_fluent<P: AsRef<Path>>(
    //     &self,
    //     path: P,
    // ) -> Result<FluentBuild, AppCoreError> {
    //     let fluents = Self::init_fluent(path).await?;
    //     let lang = self
    //         .config
    //         .get_string("app_lang")
    //         .unwrap_or_else(|_| String::from("en-US"));
    //     let langid_en = LanguageIdentifier::from_str(&lang).unwrap_or_default();
    //     let bundle = FluentBundle::new_concurrent(vec![langid_en]);
    //     Ok(FluentBuild {
    //         fluent_key: RwLock::new(lang),
    //         fluents: RwLock::new(fluents),
    //         fluent_def: RwLock::new(bundle),
    //     })
    // }
}
