mod oauth_server;
mod stat;
use lsys_access::dao::AccessDao;
use lsys_app::dao::AppConfig;
use lsys_app::dao::AppDao;
use lsys_core::AppCore;
use lsys_core::AppCoreError;
use lsys_core::RemoteNotify;
use lsys_logger::dao::ChangeLoggerDao;
use sqlx::MySql;
use std::sync::Arc;

pub struct WebApp {
    pub app_dao: Arc<AppDao>,
    db: sqlx::Pool<MySql>,
}

impl WebApp {
    pub async fn new(
        db: sqlx::Pool<MySql>,
        redis: deadpool_redis::Pool,
        app_core: Arc<AppCore>,
        access_dao: Arc<AccessDao>,
        remote_notify: Arc<RemoteNotify>,
        change_logger: Arc<ChangeLoggerDao>,
        config: AppConfig,
    ) -> Result<Self, AppCoreError> {
        let app_dao = AppDao::new(
            app_core,
            access_dao,
            db.clone(),
            redis,
            remote_notify,
            change_logger,
            config,
        )
        .await?;
        let app_dao = Arc::new(app_dao);
        tokio::spawn({
            let sub_app_change_notify = app_dao.clone();
            async move {
                sub_app_change_notify
                    .listen_sub_app_change_notify(None)
                    .await
            }
        });
        tokio::spawn({
            let task_notify_app_dao = app_dao.clone();
            async move { task_notify_app_dao.listen_task_notify().await }
        });
        Ok(Self { app_dao, db })
    }
}
