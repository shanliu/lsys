use crate::dao::{AppError, AppResult};
use crate::model::{AppModel, AppOAuthServerScopeModel, AppRequestType};

use super::AppOAuthServer;
use crate::model::AppOAuthServerScopeStatus;
use lsys_core::db::ModelTableName;
use lsys_core::db::SqlQuote;
use lsys_core::sql_format;

impl AppOAuthServer {
    //检测指定SCOPE在OAUTH服务中是否存在
    pub async fn check_scope(&self, app: &AppModel, scope_data: &[&str]) -> AppResult<()> {
        app.app_status_check()?;
        let oa_res = sqlx::query_scalar::<_, String>(&sql_format!(
            "select scope_key from {} where app_id={} and status={} and scope_key in ({})",
            AppOAuthServerScopeModel::table_name(),
            app.id,
            AppOAuthServerScopeStatus::Enable as i8,
            scope_data,
        ))
        .fetch_all(&self.db)
        .await?;
        let mut out = vec![];
        for tmp in scope_data {
            let stmp = tmp.to_string();
            if !oa_res.contains(&stmp) {
                out.push(stmp);
            }
        }
        if !out.is_empty() {
            return Err(AppError::ScopeBad(out));
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct AppOAuthServerScopeData {
    pub scope_key: String,
    pub scope_name: String,
    pub scope_desc: String,
}

impl AppOAuthServer {
    //获取所有的OAUTH服务中的SCOPE
    pub async fn get_scope(&self, app: &AppModel) -> AppResult<Vec<AppOAuthServerScopeData>> {
        app.app_status_check()?;
        let oa_res = sqlx::query_as::<_, (String, String, String)>(&sql_format!(
            "select scope_key,scope_name,scope_desc from {} where app_id={} and status={}",
            AppOAuthServerScopeModel::table_name(),
            app.id,
            AppOAuthServerScopeStatus::Enable as i8,
        ))
        .fetch_all(&self.db)
        .await?
        .into_iter()
        .map(|e| AppOAuthServerScopeData {
            scope_key: e.0,
            scope_name: e.1,
            scope_desc: e.2,
        })
        .collect::<Vec<AppOAuthServerScopeData>>();
        Ok(oa_res)
    }
    pub async fn oauth_check(&self, app: &AppModel) -> AppResult<()> {
        app.app_status_check()?;
        self.app
            .cache()
            .feature_check(app, &[AppRequestType::OAuthServer.feature_key()])
            .await
    }
}
