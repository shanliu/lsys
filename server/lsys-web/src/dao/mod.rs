use ip2location::LocationDB;
use lsys_app::dao::AppDao;
use lsys_core::cache::{LocalCacheClear, LocalCacheClearItem};
use lsys_core::{AppCore, AppCoreError};
use lsys_docs::dao::DocsDao;
use lsys_logger::dao::ChangeLogger;
use lsys_rbac::dao::rbac::RbacLocalCacheClear;
use lsys_rbac::dao::{RbacDao, SystemRole};
use lsys_sender::dao::MessageTpls;
use lsys_setting::dao::Setting;
use lsys_user::dao::account::cache::UserAccountLocalCacheClear;
use lsys_user::dao::auth::{UserAuthConfig, UserAuthRedisStore};
use lsys_user::dao::UserDao;

use sqlx::{MySql, Pool};
use std::time::Duration;
use std::vec;
use std::{path::PathBuf, str::FromStr, sync::Arc};
use tera::Tera;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

pub mod app;
mod captcha;
mod mailer;
mod request;
pub mod site_config;
mod smser;
pub mod user;
use self::app::WebApp;
use self::captcha::WebAppCaptcha;
use self::mailer::WebAppMailer;

pub use self::captcha::CaptchaKey;
pub use self::mailer::WebAppMailerError;
pub use self::request::*;
pub use self::site_config::*;
use self::smser::WebAppSmser;
pub use self::smser::WebAppSmserError;
use self::user::WebUser;

pub struct WebDao {
    pub user: Arc<WebUser>,
    pub docs: Arc<DocsDao>,
    pub app: Arc<WebApp>,
    pub captcha: Arc<WebAppCaptcha>,
    pub sender_mailer: Arc<WebAppMailer>,
    pub sender_smser: Arc<WebAppSmser>,
    pub sender_tpl: Arc<MessageTpls>,
    pub app_core: Arc<AppCore>,
    pub db: Pool<MySql>,
    pub redis: deadpool_redis::Pool,
    pub tera: Arc<Tera>,
    pub setting: Arc<Setting>,
    pub logger: Arc<ChangeLogger>,
}

impl WebDao {
    pub async fn new(app_core: Arc<AppCore>) -> Result<WebDao, AppCoreError> {
        app_core.init_tracing()?;
        let db = app_core.create_db().await?;

        let tera_dir = app_core.app_dir.join("src/template");
        let tera_tpl = if tera_dir.exists() {
            String::from(tera_dir.to_string_lossy())
        } else {
            let cargo_dir = env!("CARGO_MANIFEST_DIR");
            let tpl_dir = format!("{}/src/template", cargo_dir);
            if !PathBuf::from_str(&tpl_dir)?.exists() {
                return Err(AppCoreError::AppDir(format!(
                    "not find tpl dir :{}",
                    tpl_dir
                )));
            }
            tpl_dir
        };
        let tera = Arc::new(app_core.create_tera(&tera_tpl)?);

        let redis = app_core.create_redis().await?;
        let change_logger = Arc::new(ChangeLogger::new(db.clone()));
        let setting = Arc::new(
            Setting::new(
                app_core.clone(),
                db.clone(),
                redis.clone(),
                change_logger.clone(),
            )
            .await?,
        );

        let root_user_id = app_core
            .config
            .get_array("root_user_id")
            .unwrap_or_default()
            .iter()
            .filter_map(|e| e.to_owned().into_int().map(|e| e as u64).ok())
            .collect::<Vec<u64>>();
        let rbac_dao = Arc::new(
            RbacDao::new(
                app_core.clone(),
                db.clone(),
                redis.clone(),
                change_logger.clone(),
                Some(Box::new(SystemRole::new(true, root_user_id))),
                app_core.config.get_bool("rbac_cache").unwrap_or(false),
            )
            .await?,
        );
        let login_store = UserAuthRedisStore::new(redis.clone());
        let mut login_config = UserAuthConfig::default();

        let ip_db_path = app_core.config.get_string("ip_city_db").unwrap_or_default();
        if !ip_db_path.is_empty() {
            let city_data = app_core.app_dir.join(ip_db_path);
            match LocationDB::from_file(&city_data) {
                Ok(city_db) => {
                    login_config.ip_db = Some(Mutex::new(ip2location::DB::LocationDb(city_db)));
                }
                Err(err) => {
                    warn!(
                        "read ip city db error[{}]:{:?}",
                        city_data.to_string_lossy(),
                        err
                    )
                }
            }
        } else {
            info!("ip city db not config")
        }
        let docs = Arc::new(DocsDao::new(db.clone(), setting.single.clone()).await);

        let task_docs = docs.task.clone();
        tokio::spawn(async move {
            task_docs.dispatch().await;
        });

        let user_dao = Arc::new(
            UserDao::new(
                app_core.clone(),
                db.clone(),
                redis.clone(),
                setting.single.clone(),
                change_logger.clone(),
                login_store,
                Some(login_config),
            )
            .await?,
        );
        let app_dao = Arc::new(
            AppDao::new(
                user_dao.user_account.clone(),
                app_core.clone(),
                db.clone(),
                redis.clone(),
                change_logger.clone(),
                7 * 24 * 3600, //TOKEN有效期7天
            )
            .await?,
        );
        let apps = WebApp::new(app_dao).await;
        let mailer = Arc::new(WebAppMailer::new(
            app_core.clone(),
            user_dao.fluent.clone(),
            redis.clone(),
            db.clone(),
            setting.multiple.clone(),
            change_logger.clone(),
            None,
            300, //任务最大执行时间
            true,
        ));
        let task_web_mailer = mailer.clone();
        tokio::spawn(async move {
            if let Err(err) = task_web_mailer.task().await {
                error!("mailer error:{}", err.to_string())
            }
        });
        let web_smser = Arc::new(WebAppSmser::new(
            app_core.clone(),
            redis.clone(),
            db.clone(),
            user_dao.fluent.clone(),
            setting.multiple.clone(),
            change_logger.clone(),
            None,
            300, //任务最大执行时间
            true,
        ));
        let task_web_smser = web_smser.clone();
        tokio::spawn(async move {
            if let Err(err) = task_web_smser.task().await {
                error!("smser error:{}", err.to_string())
            }
        });
        let captcha = Arc::new(WebAppCaptcha::new(redis.clone()));

        let clear_rbac_dao = rbac_dao.clone();
        let clear_user_dao = user_dao.clone();
        let clear_app_core = app_core.clone();

        tokio::spawn(async move {
            let mut cache_item: Vec<Box<dyn LocalCacheClearItem + Sync + Send + 'static>> = vec![];
            for item in RbacLocalCacheClear::new_clears(&clear_rbac_dao.rbac) {
                cache_item.push(Box::new(item))
            }
            for item in UserAccountLocalCacheClear::new_clears(&clear_user_dao.user_account) {
                cache_item.push(Box::new(item))
            }
            let local_cache_clear = LocalCacheClear::new(cache_item);
            loop {
                let redis_client = clear_app_core.create_redis_client().unwrap();
                local_cache_clear.listen(redis_client).await;
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });
        let app_locale_dir = app_core.app_dir.join("locale/lsys-web");
        let fluents_message = Arc::new(if app_locale_dir.exists() {
            app_core.create_fluent(app_locale_dir).await?
        } else {
            let cargo_dir = env!("CARGO_MANIFEST_DIR");
            app_core
                .create_fluent(cargo_dir.to_owned() + "/locale")
                .await?
        });
        let sender_tpl = Arc::new(MessageTpls::new(
            db.clone(),
            fluents_message.clone(),
            change_logger.clone(),
        ));
        Ok(WebDao {
            docs,
            user: Arc::new(WebUser::new(
                user_dao,
                rbac_dao,
                db.clone(),
                redis.clone(),
                captcha.clone(),
                app_core.clone(),
                fluents_message,
                setting.clone(),
            )),
            app: Arc::new(apps),
            captcha,
            sender_mailer: mailer,
            sender_smser: web_smser,
            sender_tpl,
            app_core,
            db,
            redis,
            tera,
            setting,
            logger: change_logger,
        })
    }
    pub fn bind_addr(&self) -> String {
        let host = self.app_core.config.get_string("app_host").unwrap();
        let port = self.app_core.config.get_string("app_port").unwrap();
        format!("{}:{}", host, port)
    }
    pub fn bind_ssl_addr(&self) -> Option<String> {
        let host = self.app_core.config.get_string("app_host").unwrap();
        let port = self.app_core.config.get_string("app_ssl_port");
        port.ok()
            .map(|port| Some(format!("{}:{}", host, port)))
            .unwrap_or_default()
    }
}
