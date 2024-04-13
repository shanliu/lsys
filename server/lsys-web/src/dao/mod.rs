use ip2location::LocationDB;
use lsys_app::dao::{AppConfig, AppDao};
use lsys_app_barcode::dao::BarCodeDao;
use lsys_app_notify::dao::Notify;
use lsys_app_sender::dao::MessageTpls;
use lsys_core::cache::{LocalCacheClear, LocalCacheClearItem};
use lsys_core::{AppCore, AppCoreError, FluentMgr, IntoFluentMessage, RemoteNotify};
#[cfg(feature = "docs")]
use lsys_docs::dao::{DocsDao, GitRemoteTask};
#[cfg(feature = "area")]
use lsys_lib_area::AreaDao;
use lsys_logger::dao::ChangeLogger;
use lsys_rbac::dao::rbac::RbacLocalCacheClear;
use lsys_rbac::dao::{RbacConfig, RbacDao, SystemRole};
use lsys_setting::dao::{Setting, SettingConfig};
use lsys_user::dao::account::cache::UserAccountLocalCacheClear;
use lsys_user::dao::account::UserAccountConfig;
use lsys_user::dao::auth::{UserAuthConfig, UserAuthRedisStore};
use lsys_user::dao::{UserConfig, UserDao};

use sqlx::{MySql, Pool};
use std::sync::Arc;
use std::vec;
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
pub use self::request::*;
pub use self::site_config::*;
use self::smser::WebAppSmser;
use self::user::WebUser;

pub struct WebDao {
    pub user: Arc<WebUser>,
    #[cfg(feature = "docs")]
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
    #[cfg(feature = "area")]
    pub area: Option<Arc<AreaDao>>,
    pub notify: Arc<Notify>,
    pub fluent: FluentMgr,
    pub barcode: Arc<BarCodeDao>,
}

impl WebDao {
    pub async fn new(app_core: Arc<AppCore>) -> Result<WebDao, AppCoreError> {
        let path = app_core.config_path(app_core.config.find(None), "fluent_dir")?;
        let fluent = FluentMgr::new(path, "app", None).await?;

        let db = app_core.create_db().await?;
        let tera = Arc::new(app_core.create_tera(None)?);
        let redis = app_core.create_redis().await?;
        let remote_notify = Arc::new(RemoteNotify::new(
            "lsys-remote-notify",
            app_core.clone(),
            redis.clone(),
        )?);

        let use_cache=app_core.config.find(None).get_bool("use_cache").unwrap_or(false);

        let change_logger = Arc::new(ChangeLogger::new(db.clone()));
        let setting = Arc::new(
            Setting::new(
                //app_core.clone(),
                db.clone(),
                remote_notify.clone(),
                SettingConfig::new(use_cache),
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
        let rbac_dao = Arc::new(
            RbacDao::new(
                db.clone(),
                remote_notify.clone(),
                RbacConfig::new(use_cache),
                change_logger.clone(),
                Some(Box::new(SystemRole::new(true, root_user_id)))
            )
            .await?,
        );
        let login_store = UserAuthRedisStore::new(redis.clone());
        let mut login_config = UserAuthConfig::new(use_cache);

        match app_core.config_path(app_core.config.find(None), "ip_city_db") {
            Ok(ip_db_path) => match LocationDB::from_file(&ip_db_path) {
                Ok(city_db) => {
                    login_config.ip_db = Some(Mutex::new(ip2location::DB::LocationDb(city_db)));
                }
                Err(err) => {
                    warn!("read ip city db error[{}]:{:?} [download url: https://github.com/shanliu/lsys/releases/tag/v0.0.0 IP2LOCATION-LITE-DB11.BIN.zip (unzip) ]", ip_db_path.display(), err)
                }
            },
            Err(err) => {
                info!(
                    "ip city db not config:{}",
                    err.to_fluent_message().default_format()
                );
            }
        }
        #[cfg(feature = "docs")]
        let docs = {
            let doc_dir = app_core.config.find(None).get_string("doc_git_dir").ok();
            let docs = Arc::new(
                DocsDao::new(
                    // app_core.clone(),
                    db.clone(),
                    remote_notify.clone(),
                    change_logger.clone(),
                    None,
                    doc_dir,
                )
                .await,
            );
            // 文档后台同步任务
            let task_docs = docs.task.clone();
            tokio::spawn(async move {
                task_docs.dispatch().await;
            });
            docs
        };

        let user_dao = Arc::new(
            UserDao::new(
                db.clone(),
                redis.clone(),
                setting.single.clone(),
                change_logger.clone(),
                remote_notify.clone(),
                login_store,
                UserConfig{
                    account:UserAccountConfig::new(use_cache),
                    oauth:login_config,
                }
            )
            .await?,
        );
        let app_dao = Arc::new(
            AppDao::new(
                user_dao.user_account.clone(),
                db.clone(),
                redis.clone(),
                remote_notify.clone(),
                AppConfig::new(use_cache),
                change_logger.clone(),
                7 * 24 * 3600, //TOKEN有效期7天
            )
            .await?,
        );
        let apps = WebApp::new(app_dao).await;
        let mailer = Arc::new(WebAppMailer::new(
            app_core.clone(),
            redis.clone(),
            db.clone(),
            setting.clone(),
            change_logger.clone(),
            None,
            300, //任务最大执行时间
            true,
        ));
        // 邮件发送任务
        let mail_task = mailer.clone();
        tokio::spawn(async move {
            if let Err(err) = mail_task.task_sender().await {
                error!(
                    "mailer task error:{}",
                    err.to_fluent_message().default_format()
                )
            }
        });
        let mail_wait = mailer.clone();
        tokio::spawn(async move { mail_wait.task_wait().await });

        let notify = Arc::new(Notify::new(
            redis.clone(),
            db.clone(),
            app_core.clone(),
            apps.app_dao.app.clone(),
            change_logger.clone(),
            None,
            None,
            None,
            true,
        ));

        //启动回调任务
        let notify_task = notify.clone();
        tokio::spawn(async move {
            if let Err(err) = notify_task.task().await {
                error!(
                    "smser sender error:{}",
                    err.to_fluent_message().default_format()
                )
            }
        });

        let web_smser = Arc::new(WebAppSmser::new(
            app_core.clone(),
            redis.clone(),
            db.clone(),
            setting.clone(),
            change_logger.clone(),
            notify.clone(),
            None,
            None,
            300, //任务最大执行时间
            true,
        ));
        //启动短信发送任务
        let sms_task_sender = web_smser.clone();
        tokio::spawn(async move {
            if let Err(err) = sms_task_sender.task_sender().await {
                error!(
                    "smser sender error:{}",
                    err.to_fluent_message().default_format()
                )
            }
        });
        //启动短信状态查询任务
        let sms_task_notify = web_smser.clone();
        tokio::spawn(async move {
            if let Err(err) = sms_task_notify.task_status_query().await {
                error!(
                    "smser notify error:{}",
                    err.to_fluent_message().default_format()
                )
            }
        });

        let sms_task_wait = web_smser.clone();
        tokio::spawn(async move { sms_task_wait.task_wait().await });

        let captcha = Arc::new(WebAppCaptcha::new(redis.clone()));

        let sender_tpl = Arc::new(MessageTpls::new(db.clone(), change_logger.clone()));

        // 本地lua缓存清理 local cache
        let mut cache_item: Vec<Box<dyn LocalCacheClearItem + Sync + Send + 'static>> = vec![];
        for item in RbacLocalCacheClear::new_clears(&rbac_dao.rbac) {
            cache_item.push(Box::new(item))
        }
        for item in UserAccountLocalCacheClear::new_clears(&user_dao.user_account) {
            cache_item.push(Box::new(item))
        }
        let local_cache_clear = LocalCacheClear::new(cache_item);
        remote_notify.push_run(Box::new(local_cache_clear)).await;

        //git文档 远程同步任务
        #[cfg(feature = "docs")]
        remote_notify
            .push_run(Box::new(GitRemoteTask::new(docs.task.clone())))
            .await;

        //远程任务后台任务
        tokio::spawn(async move {
            //listen redis notify
            remote_notify.listen().await;
        });

        //行政区域地址库数据初始化
        #[cfg(feature = "area")]
        let area = match app_core.config_path(app_core.config.find(None), "area_code_db") {
            Ok(code_path) => {
                match lsys_lib_area::CsvAreaCodeData::from_inner_path(code_path.clone(), true) {
                    Ok(tmp) => {

                        let  geo_data =  match app_core.config_path(app_core.config.find(None), "area_geo_db") {
                            Ok(geo_path) => {
                                match lsys_lib_area::CsvAreaGeoData::from_inner_path(geo_path.clone(), true){
                                    Ok(geo_obj) => {
                                        Some(geo_obj)
                                    }
                                    Err(err) => {
                                        warn!("area code db load fail on {} [download url: https://github.com/shanliu/lsys/releases/tag/v0.0.0 2023-7-area-geo.csv.gz ],error detail:{}",geo_path.display(),err);
                                        None
                                    }
                                }
                            }
                            Err(err) => {
                                info!("area geo config load fail :{}", err.to_fluent_message().default_format());
                                None
                            }
                        };
                        let data = lsys_lib_area::CsvAreaData::new(tmp, geo_data);
                        let area_index_dir = app_core
                            .config_path(app_core.config.find(None), "area_index_dir")
                            .unwrap_or_else(|_| {
                                let mut index_dir = std::env::temp_dir();
                                index_dir.push("lsys_area_cache");
                                index_dir
                            });
                        let area_index_size = app_core
                            .config
                            .find(None)
                            .get_int("area_index_size")
                            .map(|e| e.abs() as usize)
                            .ok();
                        let area_store =
                            lsys_lib_area::AreaStoreDisk::new(area_index_dir, area_index_size)
                                .map_err(|e| AppCoreError::System(e.to_string()))?;
                        Some(Arc::new(
                            AreaDao::from_csv_disk(data, area_store)
                                .map_err(|e| AppCoreError::System(e.to_string()))?,
                        ))
                    }
                    Err(err) => {
                        warn!("area code db load fail on {} [download url: https://github.com/shanliu/lsys/releases/tag/v0.0.0 2023-7-area-code.csv.gz ],error detail:{}",code_path.display(),err);
                        None
                    }
                }
            }
            Err(err) => {
                error!(
                    "load area config fail:{}",
                    err.to_fluent_message().default_format()
                );
                None
            }
        };
        #[cfg(feature = "barcode")]
        let barcode=Arc::new(BarCodeDao::new(db.clone()));


        Ok(WebDao {
            #[cfg(feature = "docs")]
            docs,
            fluent,
            user: Arc::new(WebUser::new(
                user_dao,
                rbac_dao,
                db.clone(),
                redis.clone(),
                captcha.clone(),
                app_core.clone(),
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
            #[cfg(feature = "area")]
            area,
            notify,
            #[cfg(feature = "barcode")]
            barcode,
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
