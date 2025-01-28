use crate::model::{AppModel, AppOAuthClientModel, AppRequestType};

use lsys_core::db::ModelTableName;
use lsys_core::db::SqlQuote;
use lsys_core::sql_format;

use super::super::{AppError, AppResult};
use super::AppOAuthClient;

impl AppOAuthClient {
    pub async fn oauth_check(&self, app: &AppModel) -> AppResult<()> {
        app.app_status_check()?;
        self.app
            .cache()
            .feature_check(app, &[AppRequestType::OAuthClient.feature_key()])
            .await
    }
    /// 根据APP client_id 找到对应记录
    pub async fn find_by_app(&self, app: &AppModel) -> AppResult<AppOAuthClientModel> {
        self.oauth_check(app).await?;
        self.inner_find_by_app(app).await
    }
    //检测指定回调地址是否符合配置
    pub async fn check_callback_domain(
        &self,
        app: &AppModel,
        redirect_uri: &str,
    ) -> AppResult<bool> {
        let oauth = self.find_by_app(app).await?;
        if oauth.callback_domain.is_empty() {
            return Err(AppError::AppOAuthClientBadConfig(app.client_id.to_owned()));
        }
        Ok(
            !redirect_uri.starts_with(&("https://".to_string() + &oauth.callback_domain))
                && !redirect_uri.starts_with(&("http://".to_string() + &oauth.callback_domain)),
        )
    }
    pub(crate) async fn inner_find_by_app(&self, app: &AppModel) -> AppResult<AppOAuthClientModel> {
        sqlx::query_as::<_, AppOAuthClientModel>(&sql_format!(
            "select * from {} where app_id={}",
            AppOAuthClientModel::table_name(),
            app.id
        ))
        .fetch_one(&self.db)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::AppOAuthClientBadConfig(app.client_id.to_owned()),
            _ => AppError::Sqlx(e),
        })
    }
}
