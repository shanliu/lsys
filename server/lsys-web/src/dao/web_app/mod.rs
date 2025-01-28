mod oauth_server;
use lsys_app::dao::AppDao;
use std::sync::Arc;

use super::{APP_FEATURE_BARCODE, APP_FEATURE_MAIL, APP_FEATURE_RBAC, APP_FEATURE_SMS};

pub struct WebApp {
    pub app_dao: Arc<AppDao>,
}

impl WebApp {
    pub async fn new(app_dao: Arc<AppDao>) -> Self {
        Self { app_dao }
    }
    pub fn exter_feature(&self) -> &[&str] {
        &[
            APP_FEATURE_MAIL,
            APP_FEATURE_SMS,
            APP_FEATURE_RBAC,
            APP_FEATURE_BARCODE,
        ]
    }
}
