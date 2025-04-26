use crate::{
    dao::AppResult,
    model::{AppModel, AppRequestModel, AppRequestStatus, AppRequestType},
};
use lsys_core::db::ModelTableName;
use lsys_core::db::SqlQuote;
use lsys_core::sql_format;
use lsys_core::RequestEnv;

use super::AppExterLogin;

impl AppExterLogin {
    //外部登录权限申请
    pub async fn inner_feature_exter_login_request(
        &self,
        app: &AppModel,
        req_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> AppResult<()> {
        app.app_status_check()?;
        app.is_system_app_check()?; //必须是系统APP
        self.app
            .inner_feature_request(app, AppRequestType::ExterLogin, req_user_id, env_data)
            .await
    }
    //外部登录权限确认
    pub async fn inner_feature_exter_login_confirm(
        &self,
        app: &AppModel,
        req_status: AppRequestStatus,
        confirm_note: &str,
        confirm_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> AppResult<()> {
        app.app_status_check()?;
        let req = sqlx::query_as::<_, AppRequestModel>(&sql_format!(
            "select id,status from {} where app_id={} and feature_key = {}",
            AppRequestModel::table_name(),
            app.id,
            AppRequestType::ExterLogin.feature_key()
        ))
        .fetch_one(&self.db)
        .await?;
        self.app
            .inner_feature_confirm(
                app,
                &req,
                req_status,
                confirm_note,
                confirm_user_id,
                env_data,
            )
            .await
    }
    //外部登录权限检测
    pub async fn inner_feature_exter_login_check(&self, app: &AppModel) -> AppResult<()> {
        app.app_status_check()?;
        app.is_system_app_check()?; //必须是系统APP
        self.app
            .cache()
            .feature_check(app, &[AppRequestType::ExterLogin.feature_key()])
            .await
    }
}
