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
use lsys_core::db::utils::{FetchField, fetch_field_init};
use lsys_file::dao::FileDao;
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
use lsys_access::dao::{AccessConfig, AccessDao, AccessLocalCacheClear, SessionLimitConfig};
use lsys_app::dao::{AppConfig, AppLocalCacheClear, FILE_UPLOAD_NOTIFY_METHOD};
use lsys_app::model::{AppNotifyTryTimeMode, AppNotifyType};
// use lsys_app_notify::dao::{NotifyConfig, NotifyDao};
use lsys_core::app_core::{AppCore, AppEnv};
use lsys_core::cache::{LocalCacheClear, LocalCacheClearItem};
use lsys_core::fluents::{FluentMgr, IntoFluentMessage};
use lsys_core::remote_notify::RemoteNotify;
use lsys_core::secret::{FieldEncryptor, SecretManager};
use lsys_file::dao::FileLocalCacheClear;
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

use crate::handler::export::register_exporters;

// 从 lsys-file-manager 导入公共类型
pub use lsys_file_manager::{
    COLLECTOR_LOG_LEVEL_DEBUG, COLLECTOR_LOG_LEVEL_ERROR, COLLECTOR_LOG_LEVEL_INFO,
    COLLECTOR_LOG_LEVEL_SYSTEM, COLLECTOR_LOG_LEVEL_TRACE, COLLECTOR_LOG_LEVEL_WARN,
    CollectorLogModel, CollectorRecordModel, CollectorRecordStatus, CollectorScriptModel,
    CollectorScriptStatus, ExportTaskModel, ExportTaskStatus, FileManagerError,
    FileManagerResult, FileCollector,
};

// 重新导出 export_task 子模块，保持向后兼容
pub mod export_task {
    pub use lsys_file_manager::dao::export_task::exporter;
    pub use lsys_file_manager::dao::export_task::writer;
    pub use lsys_file_manager::dao::export_task::{ExportTaskFileItem, ExportTaskItem};
}

pub struct WebDao {
    pub app_core: Arc<AppCore>,
    pub db: Pool<MySql>,
    pub redis: deadpool_redis::Pool,
    pub tera: Arc<Tera>,
    pub fluent: Arc<FluentMgr>,
    pub secret_manager: Arc<SecretManager>,
    pub field_encryptor: Arc<FieldEncryptor>,  // 统一的字段加密器
    pub web_access: Arc<WebAccess>,
    pub web_user: Arc<WebUser>,
    pub web_rbac: Arc<WebRbac>,
    pub web_setting: Arc<WebSetting>,
    pub web_app: Arc<WebApp>,
    pub web_file: Arc<WebFile>,
    pub web_collector: Arc<WebCollector>,
    pub web_export: Arc<WebExport>,
    pub app_captcha: Arc<AppCaptcha>,
    pub app_sender: Arc<AppSender>,
    pub app_area: Arc<AppArea>,
    pub web_mfa: Arc<WebMfa>,
    
    /// 任务树根节点
    pub task_root: Arc<lsys_core::task_lifecycle::TaskNode>,
}

impl WebDao {
    pub async fn new(app_core: Arc<AppCore>, task_root: Arc<lsys_core::task_lifecycle::TaskNode>) -> WebResult<WebDao> {
        let path = app_core.config_path(app_core.config.find(None), "fluent_dir")?;
        let use_cache = app_core
            .config
            .find(None)
            .get_bool("use_cache")
            .unwrap_or(false);

        let fluent = Arc::new(FluentMgr::new(path, "app", None)
            .await
            .map_err(|e| WebError::AppCore(e.into()))?);

        let tera = Arc::new(lsys_core::app_core::create_tera(&app_core).await?);
        let redis = lsys_core::app_core::create_redis_pool(&app_core).await?;
        let remote_notify = Arc::new(RemoteNotify::new(
            "lsys-remote-notify",
            app_core.clone(),
            redis.clone(),
        )?);
        fetch_field_init(remote_notify.clone(), use_cache).await;
        let db = lsys_core::app_core::create_mysql_pool(&app_core).await?;
        FetchField::init_cache(&db).await;

        // 构建 SecretManager 并集成 KMS 提供商
        let mut secret_builder = SecretManager::builder(&app_core.config);

        // 检查并注册阿里云 KMS
        if let Ok(aliyun_config) = app_core.config.find(None).get_table("kms_aliyun")
            && let (Some(access_key_id), Some(access_key_secret), Some(region)) = (
                aliyun_config.get("access_key_id").and_then(|v| v.clone().into_string().ok()),
                aliyun_config.get("access_key_secret").and_then(|v| v.clone().into_string().ok()),
                aliyun_config.get("region").and_then(|v| v.clone().into_string().ok()),
            )
        {
            let aliyun_decryptor = lsys_kms::aliyun::AliyunKmsDecryptor::new(
                access_key_id,
                access_key_secret,
                region,
            );
            secret_builder = secret_builder.kms_provider("aliyun", Arc::new(aliyun_decryptor));
            info!("KMS provider 'aliyun' registered");
        }

        // 检查并注册腾讯云 KMS
        if let Ok(tencent_config) = app_core.config.find(None).get_table("kms_tencent")
            && let (Some(secret_id), Some(secret_key), Some(region)) = (
                tencent_config.get("secret_id").and_then(|v| v.clone().into_string().ok()),
                tencent_config.get("secret_key").and_then(|v| v.clone().into_string().ok()),
                tencent_config.get("region").and_then(|v| v.clone().into_string().ok()),
            )
        {
            let tencent_decryptor = lsys_kms::tencent::TencentKmsDecryptor::new(
                secret_id,
                secret_key,
                region,
            );
            secret_builder = secret_builder.kms_provider("tencent", Arc::new(tencent_decryptor));
            info!("KMS provider 'tencent' registered");
        }

        let secret_manager = Arc::new(secret_builder.build().await?);

        // 创建全局字段加密器（用于 Email/Mobile/SMTP密码 等所有敏感数据）
        let field_encryptor = Arc::new(FieldEncryptor::new(
            secret_manager.clone(), 
            "field_encrypt_key",
            matches!(app_core.env,AppEnv::Production)
        ));

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
        let login_max_per_type = app_core
            .config
            .find(None)
            .get::<std::collections::HashMap<String, u32>>("login_max_per_type")
            .unwrap_or_default();
        let access_dao = Arc::new(AccessDao::new(
            db.clone(),
            remote_notify.clone(),
            AccessConfig::new(use_cache, SessionLimitConfig::new(login_max_per_type)),
        ));

        let account_dao = Arc::new(AccountDao::new(
            db.clone(),
            redis.clone(),
            setting_dao.single.clone(),
            AccountConfig::new(use_cache),
            remote_notify.clone(),
            change_logger.clone(),
            secret_manager.clone(),  // 传递 SecretManager（用于 password_pepper）
            field_encryptor.clone(),  // 传递 FieldEncryptor（用于字段加密）
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
                    warn!(
                        "read ip city db error[{}]:{:?} [download url: https://github.com/shanliu/lsys/releases/tag/v0.0.0 IP2LOCATION-LITE-DB11.BIN.zip (unzip) ]",
                        ip_db_path.display(),
                        err
                    );
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
            account_dao.account_login_history.clone(),
            ip_db.clone(),
        ));
        let auth_account_dao = Arc::new(AuthAccount::new(
            account_dao.account_login_history.clone(),
            access_dao.clone(),
            AuthAccountConfig::new(ip_db),
            mfa_login_dao.clone(),
        ));
        let auth_code_dao = Arc::new(AuthCode::new(access_dao.clone(), secret_manager.clone()));

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
                task_root.child("web-app"),
            )
            .await?,
        );

        let app_area = Arc::new(AppArea::new(app_core.clone())?);
        let app_captcha = Arc::new(AppCaptcha::new(redis.clone()));
        let oss_registry = Arc::new(lsys_file::oss::OssProviderRegistry::with_defaults());
        let file_dao = Arc::new(
            FileDao::build(
                db.clone(),
                app_core.clone(),
                redis.clone(),
                secret_manager.clone(),
                setting_dao.clone(),
                oss_registry.clone(),
                change_logger.clone(),
                remote_notify.clone(),
            )
            .await,
        );

        let app_sender = Arc::new(
            AppSender::new(
                app_core.clone(),
                redis.clone(),
                db.clone(),
                web_app.app_dao.app_notify.clone(),
                setting_dao.clone(),
                change_logger.clone(),
                field_encryptor.clone(),  // SMTP 也使用同一个加密器
                task_root.child("app-sender"),
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

        let web_collector = Arc::new(WebCollector::new(
            db.clone(),
            file_dao.clone(),
            change_logger.clone(),
            &app_core,
            task_root.child("web-collector"),
        )?);

        // 创建 Web 导出任务管理器（内部初始化 ExportTask）
        let mut web_export_task = WebExportTask::new(
            db.clone(),
            file_dao.clone(),
            change_logger.clone(),
            &app_core,
            fluent.clone(),
        );

        // 注册所有导出器
        register_exporters(
            &mut web_export_task,
            web_rbac.clone(),
            account_dao.clone(),
            access_dao.clone(),
            web_app.clone(),
            app_sender.clone(),
            file_dao.clone(),
            web_collector.collector.clone(),
            web_rbac.rbac_dao.clone(),
            change_logger.clone(),
        )
        .await?;

        // web_export_task 在注册完导出器后通过 start_dispatch 启动后台调度循环
        web_export_task.start_dispatch(task_root.child("web-export"));
        web_export_task.start_dispatch(task_root.child("web-export"));

        // 文件上传完成回调发送器（rest 场景且应用配置了回调地址时生效）
        let file_notify_sender = Arc::new(web_app.app_dao.app_notify.sender_create(
            FILE_UPLOAD_NOTIFY_METHOD,
            AppNotifyType::Http,
            3,
            AppNotifyTryTimeMode::Exponential,
            60,
            false,
        ));

        let web_file = Arc::new(WebFile::new(
            redis.clone(),
            app_core.clone(),
            file_dao.clone(),
            file_notify_sender,
            task_root.child("web-file"),
        )?);

        let web_export = Arc::new(WebExport::new(Arc::new(web_export_task)));

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
        for item in FileLocalCacheClear::new_clears(&file_dao) {
            cache_item.push(Box::new(item))
        }

        remote_notify
            .push_run(Box::new(LocalCacheClear::new(cache_item)))
            .await;

        //远程任务后台任务
        let notify_node = task_root.child("remote-notify");
        notify_node.spawn(|token| async move {
            //listen redis notify
            remote_notify.listen(token).await;
        });

        Ok(WebDao {
            app_core,
            db,
            redis,
            tera,
            fluent,
            secret_manager,
            field_encryptor,  // 全局统一的字段加密器
            web_access,
            web_user,
            web_rbac,
            web_setting,
            web_app,
            web_file,
            web_collector,
            web_export,
            app_captcha,
            app_sender,
            app_area,
            web_mfa,
            task_root,
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
    
    /// 关闭所有后台任务
    pub async fn shutdown(&self) -> WebResult<()> {
        let report = self.task_root.shutdown().await;
        report.log_tree(0);
        let (completed, timed_out, panicked) = report.count_summary();
        if timed_out > 0 || panicked > 0 {
            warn!(
                completed,
                timed_out,
                panicked,
                "some tasks did not shut down cleanly"
            );
        }
        Ok(())
    }
}
