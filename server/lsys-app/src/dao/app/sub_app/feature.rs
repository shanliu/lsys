use crate::{
    dao::{App, AppResult},
    model::{AppModel, AppRequestModel, AppRequestStatus, AppRequestType},
};
use lsys_core::db::ModelTableName;
use lsys_core::db::SqlQuote;
use lsys_core::sql_format;
use lsys_core::RequestEnv;

impl App {
    //子APP信息查询权限检测,非系统应用无此功能
    pub async fn inner_feature_sub_app_check(&self, app: &AppModel) -> AppResult<()> {
        app.app_status_check()?;
        app.is_system_app_check()?; //必须是系统APP
        self.cache()
            .feature_check(app, &[AppRequestType::SubApp.feature_key()])
            .await
    }
    //子APP信息查询权限申请
    pub async fn inner_feature_sub_app_request(
        &self,
        app: &AppModel,
        req_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> AppResult<()> {
        app.app_status_check()?;
        app.is_system_app_check()?; //必须是系统APP
        self.inner_feature_request(app, AppRequestType::SubApp, req_user_id, env_data)
            .await
    }
    //子APP信息查询权限确认
    pub async fn inner_feature_sub_app_confirm(
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
            AppRequestType::SubApp.feature_key()
        ))
        .fetch_one(&self.db)
        .await?;
        self.inner_feature_confirm(
            app,
            &req,
            req_status,
            confirm_note,
            confirm_user_id,
            env_data,
        )
        .await
    }
}
