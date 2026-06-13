// 采集器脚本相关导出
//
//   ScriptRecordsExporter — 脚本执行记录列表
//     CSV 列: id, request_id, script_id, add_user_id, app_id, status, elapsed_ms, error_message, add_time, start_time, finish_time
//   ScriptRecordsExportCheck — 脚本执行记录权限检查器

use std::path::PathBuf;
use std::sync::Arc;

use lsys_core::db::{CursorConfig, CursorLimit, CursorPageDir, CursorPageParam, CursorPageSort};
use lsys_file_manager::dao::collector::CollectorRecordListAttr;
use lsys_file_manager::FileCollector;

use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::user::CheckUserFileView;
use crate::dao::export_task::exporter::Exporter;
use crate::dao::export_task::writer::CsvWriter;
use crate::dao::{ExportTaskModel, FileManagerError, WebExporterCheck, WebExportCheckParam, WebResult};

pub const EXPORT_TYPE_APP_SCRIPT_RECORDS: &str = "app_script_records";

/// 脚本执行记录权限检查器
pub struct ScriptRecordsExportCheck {
    pub web_rbac: Arc<crate::dao::WebRbac>,
}

#[async_trait::async_trait]
impl WebExporterCheck for ScriptRecordsExportCheck {
    async fn check(
        &self,
        check_env: &RbacAccessCheckEnv<'_>,
        param: &WebExportCheckParam<'_>,
    ) -> WebResult<()> {
        self.web_rbac
            .check(
                check_env,
                &CheckUserFileView {
                    res_user_id: param.user_id,
                },
            )
            .await?;
        Ok(())
    }
}

/// 脚本执行记录导出器
pub struct ScriptRecordsExporter {
    pub collector: Arc<FileCollector>,
}

impl Exporter<crate::dao::WebError> for ScriptRecordsExporter {
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
            let script_id = params["script_id"].as_u64().unwrap_or(0);
            let status = params["status"].as_i64().map(|v| v as i8);

            let mut scripts = Vec::new();
            if script_id > 0 {
                let script = self
                    .collector
                    .find_script_by_id(script_id)
                    .await?
                    .ok_or_else(|| {
                        FileManagerError::Message(lsys_core::fluent_message!(
                            "collector-script-not-found"
                        ))
                    })?;
                scripts.push(script);
            } else {
                let mut page = 0;
                loop {
                    let page_param = lsys_core::db::OffsetPageParam::new(Some(
                        lsys_core::db::OffsetPageValue::new(page * 200, 200)
                    ));
                    let mut items = self
                        .collector
                        .list_scripts(record.app_id, None, &page_param)
                        .await?;
                    if items.is_empty() {
                        break;
                    }
                    let count = items.len();
                    scripts.append(&mut items);
                    if count < 200 {
                        break;
                    }
                    page += 1;
                }
            }

            let attr = CollectorRecordListAttr::default();

            let fluent = fluent_mgr.locale(lang.as_deref());
            let mut w = CsvWriter::new(&record)
                .header(export_header!(
                    fluent,
                    EXPORT_TYPE_APP_SCRIPT_RECORDS,
                    "id",
                    "request_id",
                    "script_id",
                    "add_user_id",
                    "app_id",
                    "status",
                    "elapsed_ms",
                    "error_message",
                    "add_time",
                    "start_time",
                    "finish_time",
                ))
                .await?;

            for script in scripts {
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
                        .collector
                        .list_records(&script, None, status, &page_param, &attr)
                        .await?;

                    if items.is_empty() {
                        break;
                    }

                    let rows: Vec<_> = items
                        .iter()
                        .map(|item| {
                            (
                                item.record.id,
                                item.record.request_id.clone(),
                                item.record.script_id,
                                item.record.add_user_id,
                                item.record.app_id,
                                item.record.status,
                                item.record.elapsed_ms,
                                item.record.error_message.clone(),
                                item.record.add_time,
                                item.record.start_time,
                                item.record.finish_time,
                            )
                        })
                        .collect();

                    w.write_batch(rows).await?;

                    cursor = page_data.next_cursor;
                    if cursor.is_none() {
                        break;
                    }
                }
            }

            w.finish().await.map_err(Into::into)
        })
    }
}
