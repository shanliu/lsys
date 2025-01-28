mod app;
mod logger;
mod oauth_client;
mod oauth_server;
mod result;
mod session;

use lsys_access::dao::{AccessDao, AccessError, AccessResult, AccessSession, SessionBody};
use lsys_core::{
    cache::LocalCacheConfig, fluent_message, AppCoreError, IntoFluentMessage, RemoteNotify,
};

use crate::model::AppModel;

use lsys_logger::dao::ChangeLoggerDao;
use sqlx::{MySql, Pool};
use std::sync::Arc;

pub use app::*;
pub use oauth_client::*;
pub use oauth_server::*;
pub use result::*;
pub use session::*;

pub struct AppDao {
    //内部依赖
    pub app: Arc<App>,
    pub oauth_client: Arc<AppOAuthClient>,
    pub oauth_server: Arc<AppOAuthServer>,
}

pub struct AppConfig {
    pub app_cache: LocalCacheConfig,
    pub sub_app_cache: LocalCacheConfig,
}

impl AppConfig {
    pub fn new(use_cache: bool) -> Self {
        Self {
            sub_app_cache: LocalCacheConfig::new(
                "sub-app",
                if use_cache { None } else { Some(0) },
                None,
            ),
            app_cache: LocalCacheConfig::new("app", if use_cache { None } else { Some(0) }, None),
        }
    }
}

impl AppDao {
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        access: Arc<AccessDao>,
        db: Pool<MySql>,
        remote_notify: Arc<RemoteNotify>,
        config: AppConfig,
        logger: Arc<ChangeLoggerDao>,
        code_time: u64,
        login_time: u64,
    ) -> Result<AppDao, AppCoreError> {
        let app = Arc::from(App::new(
            db.clone(),
            remote_notify.clone(),
            config.app_cache,
            logger.clone(),
        ));
        let oauth_server = Arc::from(AppOAuthServer::new(db.clone(), app.clone(), logger.clone()));
        let oauth_client = Arc::from(AppOAuthClient::new(
            db.clone(),
            app.clone(),
            oauth_server.clone(),
            access.clone(),
            logger.clone(),
            remote_notify.clone(),
            AppOAuthClientConfig {
                cache_config: config.app_cache,
                code_time,
                login_time,
            },
        ));
        Ok(AppDao {
            app,
            oauth_client,
            oauth_server,
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
