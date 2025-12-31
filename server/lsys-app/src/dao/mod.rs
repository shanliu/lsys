mod app;
mod app_notify;
mod app_secret;
mod cache;
mod exter_login;
mod logger;
mod oauth_client;
mod oauth_server;
mod result;
mod session;

use exter_login::AppExterLogin;
use lsys_access::dao::{AccessDao, AccessError, AccessResult, AccessSession, SessionBody};
use lsys_core::{
    cache::LocalCacheConfig, fluent_message, AppCore, AppCoreError, IntoFluentMessage,
    RemoteNotify, TimeOutTaskConfig, TimeOutTaskNotify,
};
use tracing::error;

use crate::model::{AppModel, AppNotifyTryTimeMode, AppNotifyType};

pub use app::*;
pub use app_notify::*;
pub use app_secret::*;
pub use cache::AppLocalCacheClear;
use lsys_logger::dao::ChangeLoggerDao;
pub use oauth_client::*;
pub use oauth_server::*;
pub use result::*;
pub use session::*;
use sqlx::{MySql, Pool};
use std::sync::Arc;
pub const SUB_APP_SECRET_NOTIFY_METHOD: &str = "sub_app_notify";
pub struct AppDao {
    //内部依赖
    app_core: Arc<AppCore>,
    pub app: Arc<App>,
    pub(crate) app_secret: Arc<AppSecret>,
    pub oauth_client: Arc<AppOAuthClient>,
    pub oauth_server: Arc<AppOAuthServer>,
    pub app_notify: Arc<AppNotify>,
    pub exter_login: Arc<AppExterLogin>,
}

pub struct SubAppNotifyConfig {
    pub timeout_task_config: TimeOutTaskConfig,
    pub notify_type: AppNotifyType,
    pub notify_try_max: u8,
    pub notify_try_mode: AppNotifyTryTimeMode,
    pub notify_try_delay: u16,
}

impl Default for SubAppNotifyConfig {
    fn default() -> Self {
        Self {
            timeout_task_config: TimeOutTaskConfig::new("sub_app_notify", 300),
            notify_type: AppNotifyType::Http, //HTTP 模式进行回调
            notify_try_max: 2,
            notify_try_mode: AppNotifyTryTimeMode::Fixed,
            notify_try_delay: 60,
        }
    }
}

pub struct AppConfig {
    pub app_cache: LocalCacheConfig,
    pub app_secret_cache: LocalCacheConfig,
    pub sub_app_cache: LocalCacheConfig,
    pub sub_app_oauth_server_cache: LocalCacheConfig,
    pub oauth_client_code_time: u64,
    pub oauth_client_login_time: u64,
    pub oauth_client_refresh_time: u64,
    pub sub_app_notify_config: SubAppNotifyConfig,
}

impl AppConfig {
    pub fn new(
        use_cache: bool,
        oauth_client_code_time: u64,
        oauth_client_login_time: u64,
        oauth_client_refresh_time: u64,
    ) -> Self {
        Self {
            sub_app_cache: LocalCacheConfig::new(
                "sub-app",
                if use_cache { None } else { Some(0) },
                None,
            ),
            sub_app_oauth_server_cache: LocalCacheConfig::new(
                "app-server-scope",
                if use_cache { None } else { Some(0) },
                None,
            ),
            sub_app_notify_config: SubAppNotifyConfig::default(),
            app_cache: LocalCacheConfig::new("app", if use_cache { None } else { Some(0) }, None),
            app_secret_cache: LocalCacheConfig::new(
                "app-secret",
                if use_cache { None } else { Some(0) },
                None,
            ),
            oauth_client_code_time,
            oauth_client_login_time,
            oauth_client_refresh_time,
        }
    }
}

impl AppDao {
    pub async fn new(
        app_core: Arc<AppCore>,
        access: Arc<AccessDao>,
        db: Pool<MySql>,
        redis: deadpool_redis::Pool,
        remote_notify: Arc<RemoteNotify>,
        logger: Arc<ChangeLoggerDao>,
        config: AppConfig,
    ) -> Result<AppDao, AppCoreError> {
        let app_secret = Arc::from(AppSecret::new(
            db.clone(),
            remote_notify.clone(),
            config.app_secret_cache,
        ));
        let app_notify = Arc::new(AppNotify::new(
            redis.clone(),
            db.clone(),
            &NotifyConfig {
                task_size: None,
                task_timeout: None,
            },
            logger.clone(),
            app_secret.clone(),
        ));
        let sub_app_timeout_notify = Arc::new(TimeOutTaskNotify::new(
            redis.clone(),
            config.sub_app_notify_config.timeout_task_config,
        ));
        let sub_app_notify_sender = app_notify.sender_create(
            SUB_APP_SECRET_NOTIFY_METHOD,
            config.sub_app_notify_config.notify_type,
            config.sub_app_notify_config.notify_try_max,
            config.sub_app_notify_config.notify_try_mode,
            config.sub_app_notify_config.notify_try_delay,
            true,
        );
        let sub_app_change_notify = Arc::new(SubAppChangeNotify::new(
            db.clone(),
            app_secret.clone(),
            sub_app_notify_sender,
        ));

        let app = Arc::from(App::new(
            app_core.clone(),
            db.clone(),
            remote_notify.clone(),
            config.app_cache,
            logger.clone(),
            app_secret.clone(),
            sub_app_change_notify,
            sub_app_timeout_notify,
            access.clone(),
        ));
        let oauth_server = Arc::from(AppOAuthServer::new(
            db.clone(),
            app.clone(),
            logger.clone(),
            remote_notify.clone(),
            config.sub_app_oauth_server_cache,
        ));
        let oauth_client = Arc::from(AppOAuthClient::new(
            db.clone(),
            redis,
            app.clone(),
            oauth_server.clone(),
            access.clone(),
            logger.clone(),
            remote_notify.clone(),
            app_secret.clone(),
            AppOAuthClientConfig {
                cache_config: config.app_cache,
                code_time: config.oauth_client_code_time,
                login_time: config.oauth_client_login_time,
                refresh_time: config.oauth_client_refresh_time,
            },
        ));

        let exter_login = Arc::new(AppExterLogin::new(db.clone(), app.clone()));

        Ok(AppDao {
            app_core,
            app,
            oauth_client,
            oauth_server,
            app_secret,
            app_notify,
            exter_login,
        })
    }
    pub async fn listen_sub_app_change_notify(&self, channel_buffer: Option<usize>) {
        self.app.listen_sub_app_change_notify(channel_buffer).await;
    }
    pub async fn listen_task_notify(&self) {
        if let Err(err) = self
            .app_notify
            .task(self.app_core.clone(), self.app.clone())
            .await
        {
            error!("notify error:{}", err.to_fluent_message().default_format())
        }
    }

    pub async fn session_app(&self, session: &SessionBody) -> AccessResult<Option<AppModel>> {
        if session.session().user_app_id > 0 {
            let app = self
                .app
                .cache()
                .find_by_id(session.session().user_app_id)
                .await
                .map_err(|e| {
                    AccessError::System(fluent_message!("app-bad-id-error",{
                        "id":&session.session().user_app_id,
                        "msg":e.to_fluent_message(),
                    }))
                })?;
            Ok(Some(app))
        } else {
            Ok(None)
        }
    }
    pub async fn rest_session_app(&self, rest_session: &RestAuthSession) -> AccessResult<AppModel> {
        let app = self
            .app
            .cache()
            .find_by_client_id(&rest_session.get_session_token().client_id)
            .await
            .map_err(|e| {
                AccessError::System(fluent_message!("app-bad-error",{
                    "client_id":&rest_session.get_session_token().client_id,
                    "msg":e.to_fluent_message(),
                }))
            })?;
        Ok(app)
    }

    pub fn log_types() -> Vec<&'static str> {
        use lsys_logger::dao::ChangeLogData;
        vec![
            logger::AppLog::log_type(),
            logger::AppRequestLog::log_type(),
            logger::AppOAuthClientSetDomainLog::log_type(),
            logger::AppOAuthClientSecretSetLog::log_type(),
            logger::AppOAuthServerSetLog::log_type(),
            logger::AppViewSecretLog::log_type(),
            logger::AppNotifyConfigLog::log_type(),
            logger::AppNotifyDataDelLog::log_type(),
        ]
    }
}
