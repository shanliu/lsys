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
    RemoteNotify, TimeOutTask, TimeOutTaskConfig, TimeOutTaskNotify,
};

use crate::model::AppModel;

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

pub struct AppDao {
    //内部依赖
    pub app: Arc<App>,
    pub(crate) app_secret: Arc<AppSecret>,
    pub oauth_client: Arc<AppOAuthClient>,
    pub oauth_server: Arc<AppOAuthServer>,
    pub app_notify: Arc<AppNotify>,
    pub exter_login: Arc<AppExterLogin>,
    pub sub_app_timeout_task: Option<TimeOutTask<SubAppChangeNotify>>,
}

pub struct AppConfig {
    pub app_cache: LocalCacheConfig,
    pub app_secret_cache: LocalCacheConfig,
    pub sub_app_cache: LocalCacheConfig,
    pub sub_app_oauth_server_cache: LocalCacheConfig,
    pub oauth_client_code_time: u64,
    pub oauth_client_login_time: u64,
    pub sub_app_notify_config: TimeOutTaskConfig,
}

impl AppConfig {
    pub fn new(use_cache: bool, oauth_client_code_time: u64, oauth_client_login_time: u64) -> Self {
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
            sub_app_notify_config: TimeOutTaskConfig::new("sub_app_notify", 300),
            app_cache: LocalCacheConfig::new("app", if use_cache { None } else { Some(0) }, None),
            app_secret_cache: LocalCacheConfig::new(
                "app-secret",
                if use_cache { None } else { Some(0) },
                None,
            ),
            oauth_client_code_time,
            oauth_client_login_time,
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
        config: AppConfig,
        logger: Arc<ChangeLoggerDao>,
    ) -> Result<AppDao, AppCoreError> {
        let app_secret = Arc::from(AppSecret::new(
            db.clone(),
            remote_notify.clone(),
            config.app_secret_cache,
        ));
        let app_notify = Arc::new(AppNotify::new(
            redis.clone(),
            db.clone(),
            app_core.clone(),
            app_secret.clone(),
            &NotifyConfig {
                max_try: None,
                task_size: None,
                task_timeout: None,
                is_check: true,
            },
            logger.clone(),
        ));
        let sub_app_timeout_notify =
            Arc::new(TimeOutTaskNotify::new(redis, config.sub_app_notify_config));
        let sub_app_change_notify =
            SubAppChangeNotify::new(db.clone(), app_secret.clone(), app_notify.clone());
        let sub_app_timeout_task = TimeOutTask::<SubAppChangeNotify>::new(
            app_core,
            sub_app_timeout_notify.clone(),
            sub_app_change_notify.clone(),
            sub_app_change_notify.clone(),
        );
        let app = Arc::from(App::new(
            db.clone(),
            remote_notify.clone(),
            config.app_cache,
            logger.clone(),
            app_secret.clone(),
            Arc::new(sub_app_change_notify),
            sub_app_timeout_notify,
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
            },
        ));

        let exter_login = Arc::new(AppExterLogin::new(db.clone(), app.clone()));

        Ok(AppDao {
            app,
            oauth_client,
            oauth_server,
            app_secret,
            app_notify,
            exter_login,
            sub_app_timeout_task: Some(sub_app_timeout_task),
        })
    }
    pub async fn session_app(&self, session: &SessionBody) -> AccessResult<Option<AppModel>> {
        if session.session().user_app_id > 0 {
            let app = self
                .app
                .cache()
                .find_by_id(&session.session().user_app_id)
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
}
