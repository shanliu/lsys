// 系统管理员用户变更日志导出
//
//   UserChangeLogExporter — 用户变更日志列表
//     CSV 列: id, log_type, add_user_id, add_user_name, change_data, add_time

use std::path::PathBuf;
use std::sync::Arc;

use lsys_core::db::{CursorConfig, CursorLimit, CursorPageDir, CursorPageParam, CursorPageSort};
use lsys_logger::dao::ChangeLoggerDao;

use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::admin::CheckAdminChangeLogsView;
use crate::dao::export_task::exporter::Exporter;
use crate::dao::export_task::writer::CsvWriter;
use crate::dao::{ExportTaskModel, WebExporterCheck, WebExportCheckParam, WebResult};

pub const EXPORT_TYPE_SYSTEM_USER_CHANGE_LOG: &str = "system_user_change_log";

/// 系统管理员用户变更日志权限检查器
pub struct UserChangeLogExportCheck {
    pub web_rbac: Arc<crate::dao::WebRbac>,
}

#[async_trait::async_trait]
impl WebExporterCheck for UserChangeLogExportCheck {
    async fn check(
        &self,
        check_env: &RbacAccessCheckEnv<'_>,
        _param: &WebExportCheckParam<'_>,
    ) -> WebResult<()> {
        self.web_rbac
            .check(check_env, &CheckAdminChangeLogsView {})
            .await?;
        Ok(())
    }
}

/// 系统管理员用户变更日志导出器
pub struct UserChangeLogExporter {
    pub change_logger_dao: Arc<ChangeLoggerDao>,
}

impl Exporter<crate::dao::WebError> for UserChangeLogExporter {
    fn export<'a>(
        &'a self,
        record: ExportTaskModel,
        params: serde_json::Value,
        lang: Option<String>,
        fluent_mgr: Arc<lsys_core::fluents::FluentMgr>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<PathBuf, crate::dao::WebError>> + Send + 'a>,
    > {
        Box::pin(async move {
            let log_type = params["log_type"].as_str();
            let add_user_id = params["add_user_id"].as_u64();

            let fluent = fluent_mgr.locale(lang.as_deref());
            let mut w = CsvWriter::new(&record)
                .header(export_header!(
                    fluent,
                    EXPORT_TYPE_SYSTEM_USER_CHANGE_LOG,
                    "id",
                    "log_type",
                    "add_user_id",
                    "log_data",
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
                    .await?;

                if items.is_empty() {
                    break;
                }

                let rows: Vec<_> = items
                    .iter()
                    .map(|item| {
                        (
                            item.id,
                            item.log_type.clone(),
                            item.add_user_id,
                            item.log_data.clone(),
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

            w.finish().await.map_err(Into::into)
        })
    }
}
