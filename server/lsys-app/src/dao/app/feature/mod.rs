mod exter;
mod inner;
use crate::{
    dao::AppResult,
    model::{AppFeatureModel, AppFeatureStatus, AppModel},
};
use lsys_core::db::ModelTableName;
use lsys_core::db::SqlQuote;
use lsys_core::sql_format;

use super::{App, AppError};

impl App {
    //仅用在后台,不带缓存
    pub async fn feature_check(&self, app: &AppModel, featuer_data: &[&str]) -> AppResult<()> {
        if featuer_data.is_empty() {
            return Ok(());
        }
        let oa_res = sqlx::query_scalar::<_, String>(&sql_format!(
            "select feature_key from {} where app_id={} and status={} and feature_key in ({})",
            AppFeatureModel::table_name(),
            app.id,
            AppFeatureStatus::Enable as i8,
            featuer_data,
        ))
        .fetch_all(&self.db)
        .await?;
        let mut bad = vec![];
        for tmp in featuer_data {
            let ot = tmp.to_string();
            if !oa_res.contains(&ot) && !tmp.is_empty() {
                bad.push(ot);
            }
        }
        if !bad.is_empty() {
            return Err(AppError::AppBadFeature(app.name.clone(), bad));
        }
        Ok(())
    }
}
