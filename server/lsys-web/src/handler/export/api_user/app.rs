// 应用列表导出（仅用户视角）
//
// 包含三个导出器：
//   UserAppListExporter — 用户应用列表
//   UserParentAppListExporter — 用户父应用列表
//   UserSubAppListExporter — 用户子应用列表
//
// CSV 列: id, name, client_id, status, user_id, parent_app_id, change_user_id, change_time

use std::path::PathBuf;
use std::sync::Arc;

use lsys_app::dao::{AppDao, UserAppDataParam, UserParentAppDataParam, UserSubAppParam};
use lsys_app::model::AppStatus;
use lsys_core::db::{OffsetPageParam, OffsetPageValue};

use crate::dao::access::api::system::user::CheckUserAppView;
use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::export_task::exporter::Exporter;
use crate::dao::export_task::writer::CsvWriter;
use crate::dao::WebError;
use crate::dao::WebRbac;
use crate::dao::WebResult;
use crate::model::ExportTaskModel;

pub const EXPORT_TYPE_USER_APP_LIST: &str = "user_app_list";
pub const EXPORT_TYPE_USER_PARENT_APP_LIST: &str = "user_parent_app_list";
pub const EXPORT_TYPE_USER_SUB_APP_LIST: &str = "user_sub_app_list";

/// 用户应用列表导出
pub struct UserAppListExporter {
    pub app_dao: Arc<AppDao>,
    pub web_rbac: Arc<WebRbac>,
}

impl Exporter for UserAppListExporter {
    fn check<'a>(
        &'a self,
        check_env: &'a RbacAccessCheckEnv<'_>,
        _app_id: u64,
        _app_user_id: u64,
        user_id: u64,
        _export_type: &'a str,
        params: &'a serde_json::Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = WebResult<()>> + Send + 'a>> {
        Box::pin(async move {
            self.web_rbac
                .check(
                    check_env,
                    &CheckUserAppView {
                        res_user_id: user_id,
                    },
                )
                .await?;
            let parent_app_id = params["parent_app_id"].as_u64().unwrap_or(0);
            if parent_app_id > 0 {
                let papp = self.app_dao.app.find_by_id(parent_app_id).await?;
                self.app_dao.app.inner_feature_sub_app_check(&papp).await?;
            }
            Ok(())
        })
    }

    fn export<'a>(
        &'a self,
        record: ExportTaskModel,
        params: serde_json::Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<PathBuf, WebError>> + Send + 'a>>
    {
        Box::pin(async move {
            let user_id = params["user_id"].as_u64().unwrap_or(0);
            let app_id = params["app_id"].as_u64();
            let parent_app_id = params["parent_app_id"].as_u64();
            let status: Option<AppStatus> = serde_json::from_value(params["status"].clone()).ok();
            let client_id = params["client_id"].as_str();
            let like_client_id = params["like_client_id"].as_str();

            let app_where = UserAppDataParam {
                app_id,
                parent_app_id,
                status,
                client_id,
                like_client_id,
            };

            let mut w = CsvWriter::new(&record)
                .header((
                    "id",
                    "name",
                    "client_id",
                    "status",
                    "user_id",
                    "parent_app_id",
                    "change_user_id",
                    "change_time",
                ))
                .await?;

            let total = self.app_dao.app.user_app_count(user_id, &app_where).await? as u64;
            if total > 0 {
                let page = OffsetPageParam::new(Some(OffsetPageValue::new(0, total)));
                let items = self
                    .app_dao
                    .app
                    .user_app_info(user_id, &app_where, None, &page)
                    .await?;
                let rows: Vec<_> = items
                    .iter()
                    .map(|(app, _attr)| {
                        (
                            app.id,
                            app.name.clone(),
                            app.client_id.clone(),
                            app.status,
                            app.user_id,
                            app.parent_app_id,
                            app.change_user_id,
                            app.change_time,
                        )
                    })
                    .collect();
                w.write_batch(rows).await?;
            }

            w.finish().await
        })
    }
}

/// 用户父应用列表导出
pub struct UserParentAppListExporter {
    pub app_dao: Arc<AppDao>,
    pub web_rbac: Arc<WebRbac>,
}

impl Exporter for UserParentAppListExporter {
    fn check<'a>(
        &'a self,
        check_env: &'a RbacAccessCheckEnv<'_>,
        _app_id: u64,
        _app_user_id: u64,
        user_id: u64,
        _export_type: &'a str,
        _params: &'a serde_json::Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = WebResult<()>> + Send + 'a>> {
        Box::pin(async move {
            self.web_rbac
                .check(
                    check_env,
                    &CheckUserAppView {
                        res_user_id: user_id,
                    },
                )
                .await?;
            Ok(())
        })
    }

    fn export<'a>(
        &'a self,
        record: ExportTaskModel,
        params: serde_json::Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<PathBuf, WebError>> + Send + 'a>>
    {
        Box::pin(async move {
            let key_word = params["key_word"].as_str();

            let app_where = UserParentAppDataParam { key_word };

            let mut w = CsvWriter::new(&record)
                .header(("id", "name", "client_id", "status", "user_id"))
                .await?;

            let total = self.app_dao.app.user_parent_app_count(&app_where).await? as u64;
            if total > 0 {
                let page = OffsetPageParam::new(Some(OffsetPageValue::new(0, total)));
                let items = self
                    .app_dao
                    .app
                    .user_parent_app_data(&app_where, &page)
                    .await?;
                let rows: Vec<_> = items
                    .iter()
                    .map(|app| {
                        (
                            app.id,
                            app.name.clone(),
                            app.client_id.clone(),
                            app.status,
                            app.user_id,
                        )
                    })
                    .collect();
                w.write_batch(rows).await?;
            }

            w.finish().await
        })
    }
}

/// 用户子应用列表导出
pub struct UserSubAppListExporter {
    pub app_dao: Arc<AppDao>,
    pub web_rbac: Arc<WebRbac>,
}

impl Exporter for UserSubAppListExporter {
    fn check<'a>(
        &'a self,
        check_env: &'a RbacAccessCheckEnv<'_>,
        app_id: u64,
        _app_user_id: u64,
        _user_id: u64,
        _export_type: &'a str,
        _params: &'a serde_json::Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = WebResult<()>> + Send + 'a>> {
        Box::pin(async move {
            if app_id == 0 {
                return Err(WebError::Message(lsys_core::fluent_message!(
                    "export-miss-app-id"
                )));
            }
            let app = self.app_dao.app.find_by_id(app_id).await?;
            if app.parent_app_id != 0 {
                return Err(WebError::Message(lsys_core::fluent_message!(
                    "app-only-parent-can-list-sub"
                )));
            }
            self.web_rbac
                .check(
                    check_env,
                    &CheckUserAppView {
                        res_user_id: app.user_id,
                    },
                )
                .await?;
            Ok(())
        })
    }

    fn export<'a>(
        &'a self,
        record: ExportTaskModel,
        params: serde_json::Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<PathBuf, WebError>> + Send + 'a>>
    {
        Box::pin(async move {
            let app_id = params["app_id"].as_u64().unwrap_or(0);
            let sub_app_id = params["sub_app_id"].as_u64();
            let status: Option<AppStatus> = serde_json::from_value(params["status"].clone()).ok();

            let app_where = UserSubAppParam {
                app_id,
                sub_app_id,
                status,
            };

            let mut w = CsvWriter::new(&record)
                .header((
                    "id",
                    "name",
                    "client_id",
                    "status",
                    "user_id",
                    "parent_app_id",
                    "change_user_id",
                    "change_time",
                ))
                .await?;

            let total = self.app_dao.app.user_sub_app_count(&app_where).await? as u64;
            if total > 0 {
                let page = OffsetPageParam::new(Some(OffsetPageValue::new(0, total)));
                let items = self
                    .app_dao
                    .app
                    .user_sub_app_info(&app_where, None, &page)
                    .await?;
                let rows: Vec<_> = items
                    .iter()
                    .map(|(app, _attr)| {
                        (
                            app.id,
                            app.name.clone(),
                            app.client_id.clone(),
                            app.status,
                            app.user_id,
                            app.parent_app_id,
                            app.change_user_id,
                            app.change_time,
                        )
                    })
                    .collect();
                w.write_batch(rows).await?;
            }

            w.finish().await
        })
    }
}
