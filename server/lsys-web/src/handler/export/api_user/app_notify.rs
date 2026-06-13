// 应用通知列表导出
//
// CSV 列:
//   id, app_id, notify_type, notify_method, notify_key, status, try_num, try_max, publish_time, next_time
//
// AppNotifyListExporter — 应用通知列表导出器
// AppNotifyListExportCheck — 权限检查器

use std::path::PathBuf;
use std::sync::Arc;

use lsys_app::dao::AppDao;
use lsys_app::model::AppNotifyDataStatus;
use lsys_core::db::{CursorConfig, CursorLimit, CursorPageDir, CursorPageParam, CursorPageSort};

use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::user::CheckUserNotifyView;
use crate::dao::export_task::exporter::Exporter;
use crate::dao::export_task::writer::CsvWriter;
use crate::dao::{ExportTaskModel, WebExporterCheck, WebExportCheckParam, WebResult};

pub const EXPORT_TYPE_APP_NOTIFY_LIST: &str = "app_notify_list";

/// 应用通知列表权限检查器
pub struct AppNotifyListExportCheck {
    pub web_rbac: Arc<crate::dao::WebRbac>,
}

#[async_trait::async_trait]
impl WebExporterCheck for AppNotifyListExportCheck {
    async fn check(
        &self,
        check_env: &RbacAccessCheckEnv<'_>,
        param: &WebExportCheckParam<'_>,
    ) -> WebResult<()> {
        self.web_rbac
            .check(
                check_env,
                &CheckUserNotifyView {
                    res_user_id: param.user_id,
                },
            )
            .await?;
        Ok(())
    }
}

/// 应用通知列表导出器
pub struct AppNotifyListExporter {
    pub app_dao: Arc<AppDao>,
}

impl Exporter<crate::dao::WebError> for AppNotifyListExporter {
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
            let app_id = params["app_id"].as_u64();
            let app_user_id = Some(record.user_id);
            let notify_method = params["notify_method"].as_str();
            let notify_key = params["notify_key"].as_str();
            let status: Option<Vec<AppNotifyDataStatus>> = params["status"].as_array().map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_i64())
                    .filter_map(|v| AppNotifyDataStatus::try_from(v as i8).ok())
                    .collect()
            });

            let fluent = fluent_mgr.locale(lang.as_deref());
            let mut w = CsvWriter::new(&record)
                .header(export_header!(
                    fluent,
                    EXPORT_TYPE_APP_NOTIFY_LIST,
                    "id",
                    "app_id",
                    "notify_type",
                    "notify_method",
                    "notify_key",
                    "status",
                    "try_num",
                    "try_max",
                    "publish_time",
                    "next_time",
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
                    .app_dao
                    .app_notify
                    .record
                    .data_list(
                        app_id,
                        app_user_id,
                        notify_method,
                        notify_key,
                        status.as_deref(),
                        false,
                        &page_param,
                    )
                    .await?;

                if items.is_empty() {
                    break;
                }

                let rows: Vec<_> = items
                    .iter()
                    .map(|(item, _callback)| {
                        (
                            item.id,
                            item.app_id,
                            item.notify_type,
                            item.notify_method.clone(),
                            item.notify_key.clone(),
                            item.status,
                            item.try_num,
                            item.try_max,
                            item.publish_time,
                            item.next_time,
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
