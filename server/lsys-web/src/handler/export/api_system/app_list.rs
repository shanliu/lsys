// 系统应用列表导出（管理员视角）
//
// 包含三个导出器：
//   SystemAppListExporter — 系统所有APP列表
//     CSV 列: id, name, client_id, status, user_id, change_user_id, change_time
//
//   SystemSubAppListExporter — 系统子应用列表
//     CSV 列: id, name, client_id, status, user_id, change_user_id, change_time
//
//   SystemRequestListExporter — 系统审核请求列表
//     CSV 列: id, app_id, parent_app_id, status, request_type, request_user_id, request_time, confirm_user_id, confirm_time, confirm_note

use std::path::PathBuf;
use std::sync::Arc;

use lsys_app::dao::AppDao;
use lsys_app::dao::{AppRequestParam, SystemAppParam, SystemSubAppParam};
use lsys_app::model::{AppRequestStatus, AppRequestType, AppStatus};
use lsys_core::db::{OffsetPageParam, OffsetPageValue};

use crate::dao::access::api::system::admin::CheckAdminApp;
use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::export_task::exporter::Exporter;
use crate::dao::export_task::writer::CsvWriter;
use crate::dao::WebError;
use crate::dao::WebRbac;
use crate::dao::WebResult;
use crate::model::ExportTaskModel;

pub const EXPORT_TYPE_SYSTEM_APP_LIST: &str = "system_app_list";
pub const EXPORT_TYPE_SYSTEM_SUB_APP_LIST: &str = "system_sub_app_list";
pub const EXPORT_TYPE_SYSTEM_REQUEST_LIST: &str = "system_request_list";

/// 系统所有APP列表导出
pub struct SystemAppListExporter {
    pub app_dao: Arc<AppDao>,
    pub web_rbac: Arc<WebRbac>,
}

impl Exporter for SystemAppListExporter {
    fn check<'a>(
        &'a self,
        check_env: &'a RbacAccessCheckEnv<'_>,
        _app_id: u64,
        _app_user_id: u64,
        _user_id: u64,
        _export_type: &'a str,
        _params: &'a serde_json::Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = WebResult<()>> + Send + 'a>> {
        Box::pin(async move {
            self.web_rbac.check(check_env, &CheckAdminApp {}).await?;
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
            let user_id = params["user_id"].as_u64();
            let app_id = params["app_id"].as_u64();
            let app_name = params["app_name"].as_str();
            let status: Option<AppStatus> = params["status"]
                .as_i64()
                .and_then(|v| AppStatus::try_from(v as i8).ok());
            let client_id = params["client_id"].as_str();

            let app_param = SystemAppParam {
                user_id,
                status,
                client_id,
                app_id,
                app_name,
            };

            let mut w = CsvWriter::new(&record)
                .header((
                    "id",
                    "name",
                    "client_id",
                    "status",
                    "user_id",
                    "change_user_id",
                    "change_time",
                ))
                .await?;

            let total = self.app_dao.app.system_app_count(&app_param).await? as u64;
            if total > 0 {
                let page = OffsetPageParam::new(Some(OffsetPageValue::new(0, total)));
                let items = self
                    .app_dao
                    .app
                    .system_app_info(&app_param, None, &page)
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

/// 系统子应用列表导出
pub struct SystemSubAppListExporter {
    pub app_dao: Arc<AppDao>,
    pub web_rbac: Arc<WebRbac>,
}

impl Exporter for SystemSubAppListExporter {
    fn check<'a>(
        &'a self,
        check_env: &'a RbacAccessCheckEnv<'_>,
        _app_id: u64,
        _app_user_id: u64,
        _user_id: u64,
        _export_type: &'a str,
        _params: &'a serde_json::Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = WebResult<()>> + Send + 'a>> {
        Box::pin(async move {
            self.web_rbac.check(check_env, &CheckAdminApp {}).await?;
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
            let status: Option<AppStatus> = params["status"]
                .as_i64()
                .and_then(|v| AppStatus::try_from(v as i8).ok());

            let app_param = SystemSubAppParam {
                status,
                client_id: None,
                app_id,
            };

            let mut w = CsvWriter::new(&record)
                .header((
                    "id",
                    "name",
                    "client_id",
                    "status",
                    "user_id",
                    "change_user_id",
                    "change_time",
                ))
                .await?;

            let total = self.app_dao.app.system_sub_app_count(&app_param).await? as u64;
            if total > 0 {
                let page = OffsetPageParam::new(Some(OffsetPageValue::new(0, total)));
                let items = self
                    .app_dao
                    .app
                    .system_sub_app_info(&app_param, None, &page)
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

/// 系统审核请求列表导出
pub struct SystemRequestListExporter {
    pub app_dao: Arc<AppDao>,
    pub web_rbac: Arc<WebRbac>,
}

impl Exporter for SystemRequestListExporter {
    fn check<'a>(
        &'a self,
        check_env: &'a RbacAccessCheckEnv<'_>,
        _app_id: u64,
        _app_user_id: u64,
        _user_id: u64,
        _export_type: &'a str,
        _params: &'a serde_json::Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = WebResult<()>> + Send + 'a>> {
        Box::pin(async move {
            self.web_rbac.check(check_env, &CheckAdminApp {}).await?;
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
            let id = params["id"].as_u64();
            let app_id = params["app_id"].as_u64();
            let status: Option<AppRequestStatus> = params["status"]
                .as_i64()
                .and_then(|v| AppRequestStatus::try_from(v as i8).ok());
            let request_type: Option<AppRequestType> = params["request_type"]
                .as_i64()
                .and_then(|v| AppRequestType::try_from(v as i8).ok());

            let req_param = AppRequestParam {
                id,
                request_user_id: None,
                app_id,
                parent_app_id: Some(0),
                status,
                request_type,
            };

            let mut w = CsvWriter::new(&record)
                .header((
                    "id",
                    "app_id",
                    "parent_app_id",
                    "status",
                    "request_type",
                    "request_user_id",
                    "request_time",
                    "confirm_user_id",
                    "confirm_time",
                    "confirm_note",
                ))
                .await?;

            let total = self.app_dao.app.app_request_count(&req_param).await? as u64;
            if total > 0 {
                let page = OffsetPageParam::new(Some(OffsetPageValue::new(0, total)));
                let items = self.app_dao.app.app_request_info(&req_param, &page).await?;
                let rows: Vec<_> = items
                    .iter()
                    .map(|(req, _app, _data)| {
                        (
                            req.id,
                            req.app_id,
                            req.parent_app_id,
                            req.status,
                            req.request_type,
                            req.request_user_id,
                            req.request_time,
                            req.confirm_user_id,
                            req.confirm_time,
                            req.confirm_note.clone(),
                        )
                    })
                    .collect();
                w.write_batch(rows).await?;
            }

            w.finish().await
        })
    }
}
