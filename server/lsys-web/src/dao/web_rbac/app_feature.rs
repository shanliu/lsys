use super::WebRbac;
use crate::common::JsonResult;
use lsys_app::model::AppModel;

impl WebRbac {
    pub async fn app_feature_check(&self, app: &AppModel) -> JsonResult<()> {
        self.app_dao.app.inner_feature_sub_app_check(app).await?;
        self.app_dao
            .app
            .cache()
            .exter_feature_check(app, &[crate::dao::APP_FEATURE_RBAC])
            .await?;
        Ok(())
    }
}
