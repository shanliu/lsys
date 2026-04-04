// 系统操作变更日志导出（管理员视角）
//
// CSV 列: id, log_type, log_data, message, source_id, add_user_id, add_user_ip, request_id, request_user_agent, device_id, add_time

use std::path::PathBuf;
use std::sync::Arc;

use lsys_core::db::{CursorConfig, CursorLimit, CursorPageDir, CursorPageParam, CursorPageSort};
use lsys_logger::dao::ChangeLoggerDao;

use crate::dao::access::api::system::admin::CheckAdminChangeLogsView;
use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::export_task::exporter::Exporter;
use crate::dao::export_task::writer::CsvWriter;
use crate::dao::WebError;
use crate::dao::WebRbac;
use crate::dao::WebResult;
use crate::model::ExportTaskModel;

pub const EXPORT_TYPE_SYSTEM_CHANGE_LOG: &str = "system_change_log";

pub struct SystemChangeLogExporter {
    pub change_logger_dao: Arc<ChangeLoggerDao>,
    pub web_rbac: Arc<WebRbac>,
}

impl Exporter for SystemChangeLogExporter {
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
                .check(check_env, &CheckAdminChangeLogsView {})
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
            let log_type = params["log_type"].as_str();
            let add_user_id = params["add_user_id"].as_u64();

            let mut w = CsvWriter::new(&record)
                .header((
                    "id",
                    "log_type",
                    "log_data",
                    "message",
                    "source_id",
                    "add_user_id",
                    "add_user_ip",
                    "request_id",
                    "request_user_agent",
                    "device_id",
                    "add_time",
                ))
                .await?;

            let mut cursor: Option<u64> = None;
            loop {
                let page_param = CursorPageParam::new(
                    CursorPageDir::Next,
                    CursorConfig::primary(CursorPageSort::Desc),
                    cursor,
                    CursorLimit::Limit {
                        limit: 200,
                        more: true,
                    },
                );

                let (items, page_data) = self
                    .change_logger_dao
                    .list_data(log_type, add_user_id, &page_param)
                    .await
                    .map_err(|e| {
                        WebError::Message(lsys_core::fluent_message!(
                            "change-log-error",
                            format!("{:?}", e)
                        ))
                    })?;

                if items.is_empty() {
                    break;
                }

                let rows: Vec<_> = items
                    .iter()
                    .map(|item| {
                        (
                            item.id,
                            item.log_type.clone(),
                            item.log_data.clone(),
                            item.message.clone(),
                            item.source_id,
                            item.add_user_id,
                            item.add_user_ip.clone(),
                            item.request_id.clone(),
                            item.request_user_agent.clone(),
                            item.device_id.clone(),
                            item.add_time,
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
