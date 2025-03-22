use super::AppBarCode;
use crate::common::JsonResult;
use lsys_app::model::AppModel;

impl AppBarCode {
    pub async fn app_feature_check_from_app_id(&self, app_id: u64) -> JsonResult<()> {
        let app = self.app_dao.app.cache().find_by_id(&app_id).await?;
        self.app_feature_check(&app).await
    }
    pub async fn app_feature_check(&self, app: &AppModel) -> JsonResult<()> {
        self.app_dao
            .app
            .cache()
            .exter_feature_check(app, &[crate::dao::APP_FEATURE_BARCODE])
            .await?;
        Ok(())
    }
}
