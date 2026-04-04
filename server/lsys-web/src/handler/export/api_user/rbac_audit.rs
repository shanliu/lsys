// RBAC 审计日志导出
//

//
// CSV 列: id, user_id, check_result, user_ip, user_app_id, device_id, request_id, add_time

use std::path::PathBuf;
use std::sync::Arc;

use lsys_core::db::{CursorConfig, CursorLimit, CursorPageDir, CursorPageParam, CursorPageSort};
use lsys_rbac::dao::{AuditDataParam, RbacDao};

use crate::dao::WebError;
use crate::dao::WebResult;
use crate::dao::WebRbac;
use crate::dao::WebApp;
use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::user::CheckUserAppView;
use crate::dao::export_task::writer::CsvWriter;
use crate::model::ExportTaskModel;
use crate::dao::export_task::exporter::Exporter;

pub const EXPORT_TYPE_USER_RBAC_SYSTEM_AUDIT: &str = "user_rbac_system_audit";
pub const EXPORT_TYPE_USER_RBAC_APP_AUDIT: &str = "user_rbac_app_audit";

pub struct RbacAuditExporter {
    pub rbac_dao: Arc<RbacDao>,
    pub web_rbac: Arc<WebRbac>,
    pub web_app: Arc<WebApp>,
}

impl Exporter for RbacAuditExporter {
    fn check<'a>(
        &'a self,
        check_env: &'a RbacAccessCheckEnv<'_>,
        app_id: u64,
        _app_user_id: u64,
        _user_id: u64,
        export_type: &'a str,
        _params: &'a serde_json::Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = WebResult<()>> + Send + 'a>>
    {
        Box::pin(async move {
            if export_type == EXPORT_TYPE_USER_RBAC_APP_AUDIT {
                if app_id == 0 {
                    return Err(WebError::Message(lsys_core::fluent_message!("export-miss-app-id")));
                }
                let app = self.web_app.app_dao.app.find_by_id(app_id).await?;
                self.web_rbac
                    .check(check_env, &CheckUserAppView { res_user_id: app.user_id })
                    .await?;
                app.app_status_check()?;
                self.web_app.app_dao.app.inner_feature_sub_app_check(&app).await?;
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

            let user_id = params["user_id"].as_u64();
            let user_app_id = params["user_app_id"].as_u64();
            let user_ip = params["user_ip"].as_str();
            let device_id = params["device_id"].as_str();
            let request_id = params["request_id"].as_str();
            let res_data = params["res_id"]
                .as_u64()
                .map(|res_id| (res_id, params["op_id"].as_u64()));

            let audit_param = AuditDataParam {
                user_id,
                user_app_id,
                user_ip,
                device_id,
                request_id,
                res_data,
            };

            let mut w = CsvWriter::new(&record)
                .header((
                    "id", "user_id", "check_result", "user_ip",
                    "user_app_id", "device_id", "request_id", "add_time",
                ))
                .await?;

            let mut cursor: Option<u64> = None;
            loop {
                let page_param = CursorPageParam::new(
                    CursorPageDir::Next,
                    CursorConfig::primary(CursorPageSort::Asc),
                    cursor,
                    CursorLimit::Limit { limit: 200, more: true },
                );

                let (items, page_data) = self
                    .rbac_dao
                    .access
                    .audit_data(&audit_param, &page_param)
                    .await?;

                if items.is_empty() {
                    break;
                }

                let rows: Vec<_> = items
                    .iter()
                    .map(|(audit, _details)| (
                        audit.id,
                        audit.user_id,
                        audit.check_result,
                        audit.user_ip.clone(),
                        audit.user_app_id,
                        audit.device_id.clone(),
                        audit.request_id.clone(),
                        audit.add_time,
                    ))
                    .collect();

                w.write_batch(rows).await?;

                cursor = page_data.next_cursor;
                if cursor.is_none() {
                    break;
                }
            }

            w.finish().await
        })
    }
}
