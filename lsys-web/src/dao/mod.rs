use lsys_app::dao::AppDao;
use lsys_core::cache::{LocalCacheClear, LocalCacheClearItem};
use lsys_core::{AppCore, AppCoreError};
use lsys_rbac::dao::rbac::RbacLocalCacheClear;
use lsys_rbac::dao::{RbacDao, SystemRole};
use lsys_user::dao::account::cache::UserAccountLocalCacheClear;
use lsys_user::dao::auth::UserAuthRedisStore;
use lsys_user::dao::UserDao;
use redis::aio::ConnectionManager;
use sqlx::{MySql, Pool};
use std::time::Duration;
use std::vec;
use std::{path::PathBuf, str::FromStr, sync::Arc};
use tera::Tera;
use tokio::sync::Mutex;

pub mod app;
mod captcha;
mod mailer;
mod request;
mod smser;
pub mod user;
#[macro_use]
pub mod access;

use self::app::WebApp;
use self::captcha::WebAppCaptcha;
use self::mailer::WebAppMailer;

pub use self::captcha::CaptchaKey;
pub use self::mailer::WebAppMailerError;
pub use self::request::*;
use self::smser::WebAppSmser;
pub use self::smser::WebAppSmserError;
use self::user::WebUser;

pub struct WebDao {
    pub user: Arc<WebUser>,
    pub app: Arc<WebApp>,
    pub captcha: Arc<WebAppCaptcha>,
    pub mailer: Arc<WebAppMailer>,
    pub smser: Arc<WebAppSmser>,
    pub app_core: Arc<AppCore>,
    pub db: Pool<MySql>,
    pub redis: Arc<Mutex<ConnectionManager>>,
    pub tera: Arc<Tera>,
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

        let redis = Arc::new(Mutex::new(app_core.create_redis().await?));
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
                Some(Box::new(SystemRole::new(true, root_user_id))),
                app_core.config.get_bool("rbac_cache").unwrap_or(false),
            )
            .await?,
        );
        let login_store = UserAuthRedisStore::new(redis.clone());
        let user_dao = Arc::new(
            UserDao::new(
                app_core.clone(),
                db.clone(),
                redis.clone(),
                login_store,
                None,
            )
            .await?,
        );
        let app_dao = Arc::new(
            AppDao::new(
                user_dao.user_account.clone(),
                app_core.clone(),
                db.clone(),
                redis.clone(),
                7 * 24 * 3600, //TOKEN有效期7天
            )
            .await?,
        );
        let apps = WebApp::new(app_dao).await;
        let mailer = Arc::new(WebAppMailer::new(
            app_core.clone(),
            tera.clone(),
            user_dao.fluent.clone(),
        ));
        let smser = Arc::new(WebAppSmser::new(
            app_core.clone(),
            tera.clone(),
            user_dao.fluent.clone(),
        ));
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

        Ok(WebDao {
            user: Arc::new(WebUser::new(
                user_dao,
                rbac_dao,
                db.clone(),
                redis.clone(),
                captcha.clone(),
                app_core.clone(),
            )),
            app: Arc::new(apps),
            captcha,
            mailer,
            smser,
            app_core,
            db,
            redis,
            tera,
        })
    }
    pub fn bind_addr(&self) -> String {
        let host = self.app_core.config.get_string("app_host").unwrap();
        let port = self.app_core.config.get_string("app_port").unwrap();
        format!("{}:{}", host, port)
    }
}
