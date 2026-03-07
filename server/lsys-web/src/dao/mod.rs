mod app_area;
mod app_captcha;
mod app_sender;
pub mod result;
mod web_access;
mod web_account;
mod web_app;
mod web_files;
mod web_mfa;
mod web_rbac;
mod web_setting;

pub use app_area::*;
pub use app_captcha::*;
pub use app_sender::*;
use lsys_core::db::utils::init_string_field_cache;
pub use result::{WebError, WebResult};

use ip2location::LocationDB;
use lsys_user::dao::login::{
    EmailCodeLoginReload, EmailLoginReload, ExternalLoginReload, MobileCodeLoginReload,
    MobileLoginReload, NameLoginReload,
};
use tokio::sync::Mutex;
use tracing::{info, warn};
pub use web_access::*;
pub use web_account::*;
pub use web_app::*;

pub use web_files::*;
pub use web_mfa::*;
pub use web_rbac::*;
pub use web_setting::*;

use lsys_access::dao::{AccessConfig, AccessDao, AccessLocalCacheClear};
use lsys_app::dao::{AppConfig, AppLocalCacheClear};
// use lsys_app_notify::dao::{NotifyConfig, NotifyDao};
use lsys_core::app_core::AppCore;
use lsys_core::cache::{LocalCacheClear, LocalCacheClearItem};
use lsys_core::fluents::{FluentMgr, IntoFluentMessage};
use lsys_core::remote_notify::RemoteNotify;

use lsys_logger::dao::ChangeLoggerDao;
use lsys_rbac::dao::RbacLocalCacheClear;
use lsys_rbac::dao::{RbacConfig, RbacDao};
use lsys_setting::dao::{SettingConfig, SettingDao, SettingLocalCacheClear};
use lsys_user::dao::{
    AccountConfig, AccountDao, AccountLocalCacheClear, AuthAccount, AuthAccountConfig, AuthCode,
    MfaLoginDao, UserAuthDao, UserDao,
};

use sqlx::{MySql, Pool};
use std::sync::Arc;
use std::vec;
use tera::Tera;

pub struct WebDao {
    pub app_core: Arc<AppCore>,
    pub db: Pool<MySql>,
    pub redis: deadpool_redis::Pool,
    pub tera: Arc<Tera>,
    pub fluent: FluentMgr,
    pub web_access: Arc<WebAccess>,
    pub web_user: Arc<WebUser>,
    pub web_rbac: Arc<WebRbac>,
    pub web_setting: Arc<WebSetting>,
    pub web_app: Arc<WebApp>,
    pub web_files: Arc<WebFiles>,
    pub app_captcha: Arc<AppCaptcha>,
    pub app_sender: Arc<AppSender>,
    pub app_area: Arc<AppArea>,
    pub web_mfa: Arc<WebMfa>,
}

impl WebDao {
    pub async fn new(app_core: Arc<AppCore>) -> WebResult<WebDao> {
        let path = app_core.config_path(app_core.config.find(None), "fluent_dir")?;
        let use_cache = app_core
            .config
            .find(None)
            .get_bool("use_cache")
            .unwrap_or(false);

        let fluent = FluentMgr::new(path, "app", None)
            .await
            .map_err(|e| WebError::AppCore(e.into()))?;

        let tera = Arc::new(lsys_core::app_core::create_tera(&app_core).await?);
        let redis = lsys_core::app_core::create_redis_pool(&app_core).await?;
        let remote_notify = Arc::new(RemoteNotify::new(
            "lsys-remote-notify",
            app_core.clone(),
            redis.clone(),
        )?);
        init_string_field_cache(remote_notify.clone(), use_cache).await;
        let db = lsys_core::app_core::create_mysql_pool(&app_core).await?;

        let change_logger = Arc::new(ChangeLoggerDao::new(db.clone()));
        let setting_dao = Arc::new(
            SettingDao::new(
                //app_core.clone(),
                db.clone(),
                remote_notify.clone(),
                SettingConfig::new(use_cache),
                change_logger.clone(),
            )
            .await?,
        );

        let web_setting = Arc::new(WebSetting::new(setting_dao.clone(), db.clone()));
        let access_dao = Arc::new(AccessDao::new(
            db.clone(),
            remote_notify.clone(),
            AccessConfig::new(use_cache),
        ));

        let account_dao = Arc::new(AccountDao::new(
            db.clone(),
            redis.clone(),
            setting_dao.single.clone(),
            AccountConfig::new(use_cache),
            remote_notify.clone(),
            change_logger.clone(),
        ));
        let auth_dao = Arc::new(UserAuthDao::new(
            access_dao.clone(),
            vec![
                Box::new(NameLoginReload::new(account_dao.clone())),
                Box::new(EmailLoginReload::new(account_dao.clone())),
                Box::new(EmailCodeLoginReload::new(account_dao.clone())),
                Box::new(MobileCodeLoginReload::new(account_dao.clone())),
                Box::new(MobileLoginReload::new(account_dao.clone())),
                Box::new(ExternalLoginReload::new(account_dao.clone())),
            ],
        ));
        let ip_db = match app_core.config_path(app_core.config.find(None), "ip_city_db") {
            Ok(ip_db_path) => match LocationDB::from_file(&ip_db_path) {
                Ok(city_db) => Some(Arc::new(Mutex::new(ip2location::DB::LocationDb(city_db)))),
                Err(err) => {
                    warn!("read ip city db error[{}]:{:?} [download url: https://github.com/shanliu/lsys/releases/tag/v0.0.0 IP2LOCATION-LITE-DB11.BIN.zip (unzip) ]", ip_db_path.display(), err);
                    None
                }
            },
            Err(err) => {
                info!(
                    "ip city db not config:{}",
                    err.to_fluent_message().default_format()
                );
                None
            }
        };

        let mfa_totp_dao = Arc::new(lsys_mfa::dao::MfaTotpDao::new(db.clone(), None));
        let mfa_login_dao = Arc::new(MfaLoginDao::new(
            redis.clone(),
            access_dao.clone(),
            mfa_totp_dao.clone(),
            app_core.clone(),
            account_dao.account_login_hostory.clone(),
            ip_db.clone(),
        ));
        let auth_account_dao = Arc::new(AuthAccount::new(
            account_dao.account_login_hostory.clone(),
            access_dao.clone(),
            AuthAccountConfig::new(ip_db),
            mfa_login_dao.clone(),
        ));
        let auth_code_dao = Arc::new(AuthCode::new(access_dao.clone(), app_core.clone()));

        let user_dao = Arc::new(UserDao::new(
            account_dao.clone(),
            auth_dao,
            auth_account_dao,
            auth_code_dao,
            mfa_login_dao,
        ));

        let web_app = Arc::new(
            WebApp::new(
                db.clone(),
                redis.clone(),
                app_core.clone(),
                access_dao.clone(),
                remote_notify.clone(),
                change_logger.clone(),
                web_setting.clone(),
                AppConfig::new(
                    use_cache,
                    120,             //oauth Code有效期120秒
                    7 * 24 * 3600,   //TOKEN有效期7天
                    180 * 24 * 3600, //TOKEN有效期180天
                ),
            )
            .await?,
        );

        let app_area = Arc::new(AppArea::new(app_core.clone())?);
        let app_captcha = Arc::new(AppCaptcha::new(redis.clone()));
        let web_files = Arc::new(WebFiles::new(db.clone(), redis.clone(), &app_core, change_logger.clone())?);
        let app_sender = Arc::new(
            AppSender::new(
                app_core.clone(),
                redis.clone(),
                db.clone(),
                web_app.app_dao.app_notify.clone(),
                setting_dao.clone(),
                change_logger.clone(),
            )
            .await?,
        );
        let root_user_id = app_core
            .config
            .find(None)
            .get_array("root_user_id")
            .unwrap_or_default()
            .iter()
            .filter_map(|e| e.to_owned().into_int().map(|e| e as u64).ok())
            .collect::<Vec<u64>>();
        let web_rbac = Arc::new(WebRbac::new(
            Arc::new(
                RbacDao::new(
                    db.clone(),
                    remote_notify.clone(),
                    RbacConfig::new(use_cache),
                    change_logger.clone(),
                )
                .await?,
            ),
            root_user_id,
        ));
        let web_mfa = Arc::new(WebMfa::new(mfa_totp_dao));
        let web_user = Arc::new(WebUser::new(
            db.clone(),
            user_dao,
            web_app.app_dao.clone(),
            app_sender.clone(),
            app_captcha.clone(),
            app_area.clone(),
            change_logger.clone(),
            web_mfa.clone(),
            access_dao.clone(),
        ));

        let web_access = Arc::new(WebAccess::new(access_dao.clone()));

        // web_setting 已在 web_app 之前初始化，用于 WebApp 内部读取/写入 setting.multiple

        // 本地lua缓存清理 local cache
        let mut cache_item: Vec<Box<dyn LocalCacheClearItem>> = vec![];

        for item in AccountLocalCacheClear::new_clears(&account_dao) {
            cache_item.push(Box::new(item))
        }
        for item in AccessLocalCacheClear::new_clears(&access_dao) {
            cache_item.push(Box::new(item))
        }
        for item in AppLocalCacheClear::new_clears(&web_app.app_dao) {
            cache_item.push(Box::new(item))
        }
        for item in SettingLocalCacheClear::new_clears(&setting_dao) {
            cache_item.push(Box::new(item))
        }
        for item in RbacLocalCacheClear::new_clears(&web_rbac.rbac_dao) {
            cache_item.push(Box::new(item))
        }

        remote_notify
            .push_run(Box::new(LocalCacheClear::new(cache_item)))
            .await;

        //远程任务后台任务
        tokio::spawn(async move {
            //listen redis notify
            remote_notify.listen().await;
        });

        Ok(WebDao {
            app_core,
            db,
            redis,
            tera,
            fluent,
            web_access,
            web_user,
            web_rbac,
            web_setting,
            web_app,
            web_files,
            app_captcha,
            app_sender,
            app_area,
            web_mfa,
        })
    }
    pub fn bind_addr(&self) -> String {
        let host = self
            .app_core
            .config
            .find(None)
            .get_string("app_host")
            .unwrap_or("127.0.0.1".to_owned());
        let port = self
            .app_core
            .config
            .find(None)
            .get_string("app_port")
            .unwrap_or("80".to_owned());
        format!("{}:{}", host, port)
    }
    pub fn bind_ssl_data(&self) -> Option<(String, String, String)> {
        let host = self
            .app_core
            .config
            .find(None)
            .get_string("app_host")
            .unwrap_or("127.0.0.1".to_owned());
        let port = self
            .app_core
            .config
            .find(None)
            .get_string("app_ssl_port")
            .unwrap_or("443".to_string());
        let cert = self
            .app_core
            .config
            .find(None)
            .get_string("app_ssl_cert")
            .ok()?;
        let key = self
            .app_core
            .config
            .find(None)
            .get_string("app_ssl_key")
            .ok()?;
        Some((format!("{}:{}", host, port), cert, key))
    }
}
