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
pub const APP_FEATURE_SMS: &str = "mail";
pub const APP_FEATURE_MAIL: &str = "mail";
pub const APP_FEATURE_BARCODE: &str = "barcode";
pub const APP_FEATURE_RBAC: &str = "barcode";
impl WebApp {
    pub fn exter_feature_list(&self) -> &[&str] {
        &[
            APP_FEATURE_MAIL,
            APP_FEATURE_SMS,
            APP_FEATURE_RBAC,
            APP_FEATURE_BARCODE,
        ]
    }
}
