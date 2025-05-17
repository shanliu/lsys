use crate::dao::logger::AppViewSecretLog;
use crate::dao::AppSecretRecrod;
use crate::model::AppSecretType;
use crate::model::{AppModel, AppOAuthClientModel, AppRequestType};

use lsys_core::db::ModelTableName;
use lsys_core::db::SqlQuote;
use lsys_core::sql_format;
use lsys_core::RequestEnv;

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
            redirect_uri.starts_with(&("https://".to_string() + &oauth.callback_domain))
                || redirect_uri.starts_with(&("http://".to_string() + &oauth.callback_domain)),
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

impl AppOAuthClient {
    //添加查看secret日志
    pub async fn oauth_view_secret(
        &self,
        app: &AppModel,
        view_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> AppResult<Vec<AppSecretRecrod>> {
        let secret_data = self
            .app_secret
            .multiple_find_secret_by_app_id(app.id, AppSecretType::OAuth)
            .await?;
        self.logger
            .add(
                &AppViewSecretLog {
                    action: "secret_view",
                    app_id: app.id,
                    user_id: app.user_id,
                    app_name: &app.name,
                    secret_data: &secret_data
                        .iter()
                        .map(|e| e.secret_data.as_str())
                        .collect::<Vec<_>>()
                        .join(","),
                },
                Some(app.id),
                Some(view_user_id),
                None,
                env_data,
            )
            .await;
        Ok(secret_data)
    }
}
