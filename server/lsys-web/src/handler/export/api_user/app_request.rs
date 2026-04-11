// 应用请求列表导出
//
// 包含两个导出器：
//   AppRequestListExporter — 我的应用发出的请求列表 (request_list)
//     CSV 列: id, app_id, parent_app_id, parent_app_name, parent_app_client_id, request_type, status, request_user_id, request_time, confirm_user_id, confirm_time, confirm_note
//
//   AppSubRequestListExporter — 别人请求我的应用开通的列表 (sub_request_list)
//     CSV 列: id, app_id, app_name, app_client_id, request_type, status, request_user_id, request_time, confirm_user_id, confirm_time, confirm_note

use std::path::PathBuf;
use std::sync::Arc;

use lsys_app::dao::{AppDao, AppRequestParam};
use lsys_app::model::{AppRequestStatus, AppRequestType};
use lsys_core::db::{OffsetPageParam, OffsetPageValue};

use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::user::CheckUserAppView;
use crate::dao::export_task::exporter::Exporter;
use crate::dao::export_task::writer::CsvWriter;
use crate::dao::{ExportTaskModel, WebExporter, WebResult};

pub const EXPORT_TYPE_USER_APP_REQUEST: &str = "user_app_request";
pub const EXPORT_TYPE_USER_SUB_REQUEST: &str = "user_sub_request";

/// 我的应用发出的请求列表导出（request_list）
/// 视角：我是子应用，向父应用发请求，关注父应用信息
pub struct AppRequestListExporter {
    pub app_dao: Arc<AppDao>,
    pub web_rbac: Arc<crate::dao::WebRbac>,
}

#[async_trait::async_trait]
impl WebExporter for AppRequestListExporter {
    async fn check(
        &self,
        check_env: &RbacAccessCheckEnv<'_>,
        param: &crate::dao::ExportCheckParam<'_>,
    ) -> WebResult<()> {
        self.web_rbac
            .check(
                check_env,
                &CheckUserAppView {
                    res_user_id: param.user_id,
                },
            )
            .await?;
        Ok(())
    }
}

impl Exporter<crate::dao::WebError> for AppRequestListExporter {
    fn export<'a>(
        &'a self,
        record: ExportTaskModel,
        params: serde_json::Value,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<PathBuf, crate::dao::WebError>> + Send + 'a>,
    > {
        Box::pin(async move {
            let id = params["id"].as_u64();
            let status: Option<AppRequestStatus> =
                serde_json::from_value(params["status"].clone()).ok();
            let request_type: Option<AppRequestType> =
                serde_json::from_value(params["request_type"].clone()).ok();

            // request_list: task app_id 是子应用，parent_app_id 从 DB 获取
            let app = self.app_dao.app.find_by_id(record.app_id).await?;
            let req_param = AppRequestParam {
                id,
                request_user_id: Some(record.user_id),
                app_id: Some(record.app_id),
                parent_app_id: Some(app.parent_app_id),
                status,
                request_type,
            };

            let mut w = CsvWriter::new(&record)
                .header((
                    "id",
                    "app_id",
                    "parent_app_id",
                    "parent_app_name",
                    "parent_app_client_id",
                    "request_type",
                    "status",
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
                    .into_iter()
                    .map(|(req, info, _data)| {
                        (
                            req.id,
                            req.app_id,
                            info.parent_app_id,
                            info.parent_app_name,
                            info.parent_app_client_id,
                            req.request_type,
                            req.status,
                            req.request_user_id,
                            req.request_time,
                            req.confirm_user_id,
                            req.confirm_time,
                            req.confirm_note,
                        )
                    })
                    .collect();
                w.write_batch(rows).await?;
            }

            w.finish().await.map_err(Into::into)
        })
    }
}

/// 别人请求我的应用开通的列表导出（sub_request_list）
/// 视角：我是父应用，子应用向我发请求，关注子应用信息
pub struct AppSubRequestListExporter {
    pub app_dao: Arc<AppDao>,
    pub web_rbac: Arc<crate::dao::WebRbac>,
}

#[async_trait::async_trait]
impl WebExporter for AppSubRequestListExporter {
    async fn check(
        &self,
        check_env: &RbacAccessCheckEnv<'_>,
        param: &crate::dao::ExportCheckParam<'_>,
    ) -> WebResult<()> {
        self.web_rbac
            .check(
                check_env,
                &CheckUserAppView {
                    res_user_id: param.user_id,
                },
            )
            .await?;
        Ok(())
    }
}

impl Exporter<crate::dao::WebError> for AppSubRequestListExporter {
    fn export<'a>(
        &'a self,
        record: ExportTaskModel,
        params: serde_json::Value,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<PathBuf, crate::dao::WebError>> + Send + 'a>,
    > {
        Box::pin(async move {
            let id = params["id"].as_u64();
            let status: Option<AppRequestStatus> =
                serde_json::from_value(params["status"].clone()).ok();
            let request_type: Option<AppRequestType> =
                serde_json::from_value(params["request_type"].clone()).ok();

            // sub_request_list: task app_id 是父应用，sub_app_id 是可选的子应用过滤
            let req_param = AppRequestParam {
                id,
                request_user_id: None,
                app_id: params["sub_app_id"].as_u64(),
                parent_app_id: Some(record.app_id),
                status,
                request_type,
            };

            let mut w = CsvWriter::new(&record)
                .header((
                    "id",
                    "app_id",
                    "app_name",
                    "app_client_id",
                    "request_type",
                    "status",
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
                    .into_iter()
                    .map(|(req, info, _data)| {
                        (
                            req.id,
                            req.app_id,
                            info.name,
                            info.client_id,
                            req.request_type,
                            req.status,
                            req.request_user_id,
                            req.request_time,
                            req.confirm_user_id,
                            req.confirm_time,
                            req.confirm_note,
                        )
                    })
                    .collect();
                w.write_batch(rows).await?;
            }

            w.finish().await.map_err(Into::into)
        })
    }
}
