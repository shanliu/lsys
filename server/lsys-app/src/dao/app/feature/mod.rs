mod exter;
mod inner;
use super::{App, AppError};
use crate::{
    dao::AppResult,
    model::{AppFeatureModel, AppFeatureStatus, AppModel},
};
use lsys_core::db::{QueryBuilderExt, TableMeta};
use lsys_core::valid_key;
use lsys_core::valid_param::{ValidParam, ValidParamCheck, ValidPattern, ValidStrlen};
use sqlx::{MySql, QueryBuilder};

impl App {
    async fn exter_feature_param_valid(&self, featuer_data: &[&str]) -> AppResult<()> {
        let mut valid_param = ValidParam::default();
        for tmp in featuer_data {
            valid_param.add(
                valid_key!("featuer_data"),
                tmp,
                &ValidParamCheck::default()
                    .add_rule(ValidStrlen::range(2, 32))
                    .add_rule(ValidPattern::Ident),
            );
        }
        valid_param.check()?;
        Ok(())
    }
    //仅用在后台,不带缓存
    pub async fn feature_check(&self, app: &AppModel, featuer_data: &[&str]) -> AppResult<()> {
        if featuer_data.is_empty() {
            return Ok(());
        }
        self.exter_feature_param_valid(featuer_data).await?;
        let mut qb = QueryBuilder::<MySql>::new(format!(
            "select feature_key from {}",
            AppFeatureModel::table_name()
        ));
        qb.push_where().field_eq("app_id", app.id);
        qb.push_and()
            .field_eq("status", AppFeatureStatus::Enable as i8);
        qb.push_and().field_in_string("feature_key", featuer_data);
        let oa_res = qb
            .build_query_scalar::<String>()
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
