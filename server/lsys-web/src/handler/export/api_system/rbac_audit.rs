// 系统 RBAC 审计日志导出（管理员视角）
//
// CSV 列: id, user_id, check_result, user_ip, user_app_id, device_id, request_id, add_time

use std::path::PathBuf;
use std::sync::Arc;

use lsys_core::db::{CursorConfig, CursorLimit, CursorPageDir, CursorPageParam, CursorPageSort};
use lsys_rbac::dao::{AuditDataParam, RbacDao};

use crate::dao::access::api::system::admin::CheckAdminRbacView;
use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::export_task::exporter::Exporter;
use crate::dao::export_task::writer::CsvWriter;
use crate::dao::WebError;
use crate::dao::WebRbac;
use crate::dao::WebResult;
use crate::model::ExportTaskModel;

pub const EXPORT_TYPE_SYSTEM_RBAC_AUDIT: &str = "system_rbac_audit";

pub struct SystemRbacAuditExporter {
    pub rbac_dao: Arc<RbacDao>,
    pub web_rbac: Arc<WebRbac>,
}

impl Exporter for SystemRbacAuditExporter {
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
            self.web_rbac
                .check(check_env, &CheckAdminRbacView {})
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
                    "id",
                    "user_id",
                    "check_result",
                    "user_ip",
                    "user_app_id",
                    "device_id",
                    "request_id",
                    "add_time",
                ))
                .await?;

            let mut cursor: Option<u64> = None;
            loop {
                let page_param = CursorPageParam::new(
                    CursorPageDir::Next,
                    CursorConfig::primary(CursorPageSort::Asc),
                    cursor,
                    CursorLimit::Limit {
                        limit: 200,
                        more: true,
                    },
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
                    .map(|(audit, _details)| {
                        (
                            audit.id,
                            audit.user_id,
                            audit.check_result,
                            audit.user_ip.clone(),
                            audit.user_app_id,
                            audit.device_id.clone(),
                            audit.request_id.clone(),
                            audit.add_time,
                        )
                    })
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
