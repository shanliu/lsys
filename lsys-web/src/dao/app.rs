use std::sync::Arc;

use lsys_app::{dao::AppDao, model::AppsModel};

use lsys_rbac::dao::RoleRelationKey;

pub struct WebApp {
    pub app_dao: Arc<AppDao>,
}

impl WebApp {
    pub async fn new(app_dao: Arc<AppDao>) -> Self {
        Self { app_dao }
    }
    pub async fn app_relation_key(&self, app: &AppsModel) -> RoleRelationKey {
        RoleRelationKey::system(format!("app-{}", app.id))
    }
}
