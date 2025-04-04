mod app_area;
mod app_captcha;
mod app_notify;
mod app_sender;
mod web_access;
mod web_account;
mod web_app;
mod web_rbac;
mod web_setting;

#[cfg(feature = "barcode")]
mod app_barcode;
#[cfg(feature = "docs")]
mod web_doc;

pub use app_area::*;
#[cfg(feature = "barcode")]
pub use app_barcode::*;
pub use app_captcha::*;
pub use app_notify::*;
pub use app_sender::*;

use lsys_user::dao::login::{
    EmailCodeLoginReload, EmailLoginReload, ExternalLoginReload, MobileCodeLoginReload,
    MobileLoginReload, NameLoginReload,
};
pub use web_access::*;
pub use web_account::*;
pub use web_app::*;
#[cfg(feature = "docs")]
pub use web_doc::*;
pub use web_rbac::*;
pub use web_setting::*;

use lsys_access::dao::{AccessConfig, AccessDao, AccessLocalCacheClear};
use lsys_app::dao::{AppConfig, AppDao};
use lsys_app_notify::dao::{NotifyConfig, NotifyDao};
use lsys_core::cache::{LocalCacheClear, LocalCacheClearItem};
use lsys_core::{AppCore, AppCoreError, FluentMgr, RemoteNotify};

use lsys_logger::dao::ChangeLoggerDao;
use lsys_rbac::dao::RbacLocalCacheClear;
use lsys_rbac::dao::{RbacConfig, RbacDao};
use lsys_setting::dao::{SettingConfig, SettingDao};
use lsys_user::dao::{
    AccountConfig, AccountDao, AccountLocalCacheClear, AuthAccount, AuthAccountConfig, AuthCode,
    UserAuthDao, UserDao,
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
    pub app_captcha: Arc<AppCaptcha>,
    pub app_notify: Arc<AppNotify>,
    pub app_sender: Arc<AppSender>,
    pub app_area: Arc<AppArea>,
    #[cfg(feature = "docs")]
    pub web_doc: Arc<WebDoc>,
    #[cfg(feature = "barcode")]
    pub app_barcode: Arc<AppBarCode>,
}

impl WebDao {
    pub async fn new(app_core: Arc<AppCore>) -> Result<WebDao, AppCoreError> {
        let path = app_core.config_path(app_core.config.find(None), "fluent_dir")?;
        let fluent = FluentMgr::new(path, "app", None).await?;

        let db = app_core.create_db().await?;
        let tera = Arc::new(app_core.create_tera().await?);
        let redis = app_core.create_redis().await?;
        let remote_notify = Arc::new(RemoteNotify::new(
            "lsys-remote-notify",
            app_core.clone(),
            redis.clone(),
        )?);

        let use_cache = app_core
            .config
            .find(None)
            .get_bool("use_cache")
            .unwrap_or(false);
        let change_logger = Arc::new(ChangeLoggerDao::new(db.clone()));

        let setting = Arc::new(
            SettingDao::new(
                //app_core.clone(),
                db.clone(),
                remote_notify.clone(),
                SettingConfig::new(use_cache),
                change_logger.clone(),
            )
            .await?,
        );

        let access_dao = Arc::new(AccessDao::new(
            db.clone(),
            redis.clone(),
            remote_notify.clone(),
            AccessConfig::new(use_cache),
        ));

        let app_dao = Arc::new(
            AppDao::new(
                access_dao.clone(),
                db.clone(),
                remote_notify.clone(),
                AppConfig::new(
                    use_cache,
                    120,           //oauth Code有效期120秒
                    7 * 24 * 3600, //TOKEN有效期7天
                ),
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
        let web_rbac = Arc::new(WebRbac::new(Arc::new(
            RbacDao::new(
                db.clone(),
                remote_notify.clone(),
                RbacConfig::new(root_user_id, use_cache),
                change_logger.clone(),
            )
            .await?,
        )));

        let app_area = Arc::new(AppArea::new(app_core.clone())?);

        let app_captcha = Arc::new(AppCaptcha::new(redis.clone()));

        let notify = Arc::new(NotifyDao::new(
            redis.clone(),
            db.clone(),
            app_core.clone(),
            app_dao.app.clone(),
            &NotifyConfig {
                max_try: None,
                task_size: None,
                task_timeout: None,
                is_check: true,
            },
            change_logger.clone(),
        ));

        let app_sender = Arc::new(AppSender::new(
            app_core.clone(),
            redis.clone(),
            db.clone(),
            notify.clone(),
            setting.clone(),
            change_logger.clone(),
        )?);

        let account_dao = Arc::new(AccountDao::new(
            db.clone(),
            redis.clone(),
            setting.single.clone(),
            access_dao.clone(),
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
        let auth_account_dao = Arc::new(AuthAccount::new(
            account_dao.account_login_hostory.clone(),
            access_dao.clone(),
            AuthAccountConfig::new(None),
        ));
        let auth_code_dao = Arc::new(AuthCode::new(access_dao.clone(), app_core.clone()));

        let user_dao = Arc::new(UserDao::new(
            account_dao.clone(),
            auth_dao,
            auth_account_dao,
            auth_code_dao,
        ));

        let web_user = Arc::new(WebUser::new(
            db.clone(),
            user_dao,
            app_sender.clone(),
            app_captcha.clone(),
            app_area.clone(),
            change_logger.clone(),
        ));

        let web_app = Arc::new(WebApp::new(app_dao.clone()).await);

        let web_access = Arc::new(WebAccess::new(access_dao.clone()));

        let web_setting = Arc::new(WebSetting::new(setting.clone(), db.clone()));

        //启动回调任务
        let app_notify = Arc::new(AppNotify::new(notify.clone()));

        // 本地lua缓存清理 local cache
        let mut cache_item: Vec<Box<dyn LocalCacheClearItem + Sync + Send + 'static>> = vec![];

        for item in RbacLocalCacheClear::new_clears(&web_rbac.rbac_dao) {
            cache_item.push(Box::new(item))
        }
        for item in AccountLocalCacheClear::new_clears(&account_dao) {
            cache_item.push(Box::new(item))
        }
        for item in AccessLocalCacheClear::new_clears(&access_dao) {
            cache_item.push(Box::new(item))
        }

        #[cfg(feature = "docs")]
        let web_doc = Arc::new(
            WebDoc::new(
                app_core.clone(),
                db.clone(),
                remote_notify.clone(),
                change_logger.clone(),
            )
            .await,
        );

        #[cfg(feature = "barcode")]
        let app_barcode = {
            let barcode = Arc::new(AppBarCode::new(
                app_core.clone(),
                db.clone(),
                remote_notify.clone(),
                change_logger.clone(),
            ));
            for item in
                lsys_app_barcode::dao::BarCodeLocalCacheClear::new_clears(&barcode.barcode_dao)
            {
                cache_item.push(Box::new(item))
            }
            barcode
        };

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
            app_captcha,
            app_notify,
            app_sender,
            app_area,
            #[cfg(feature = "docs")]
            web_doc,
            #[cfg(feature = "barcode")]
            app_barcode,
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
