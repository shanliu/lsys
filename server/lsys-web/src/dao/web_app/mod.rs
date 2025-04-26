mod oauth_server;
use lsys_app::dao::AppDao;

use lsys_core::IntoFluentMessage;
use std::sync::Arc;
use tracing::error;
pub struct WebApp {
    pub app_dao: Arc<AppDao>,
}

impl WebApp {
    pub async fn new(app_dao: Arc<AppDao>) -> Self {
        let notify_task = app_dao.app_notify.clone();
        tokio::spawn(async move {
            if let Err(err) = notify_task.task().await {
                error!("notify error:{}", err.to_fluent_message().default_format())
            }
        });
        Self { app_dao }
    }
}
