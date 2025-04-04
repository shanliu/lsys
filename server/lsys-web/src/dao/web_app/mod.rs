mod oauth_server;
use lsys_app::dao::AppDao;
use std::sync::Arc;

pub struct WebApp {
    pub app_dao: Arc<AppDao>,
}

impl WebApp {
    pub async fn new(app_dao: Arc<AppDao>) -> Self {
        Self { app_dao }
    }
}
