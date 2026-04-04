// 应用通知列表导出
//

//
// CSV 列:
//   id, app_id, notify_type, notify_method, notify_key, status, try_num, try_max, publish_time, next_time

use std::path::PathBuf;
use std::sync::Arc;

use lsys_app::dao::AppDao;
use lsys_app::model::AppNotifyDataStatus;
use lsys_core::db::{CursorConfig, CursorLimit, CursorPageDir, CursorPageParam, CursorPageSort};

use crate::dao::access::api::system::user::CheckUserNotifyView;
use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::export_task::exporter::Exporter;
use crate::dao::export_task::writer::CsvWriter;
use crate::dao::WebError;
use crate::dao::WebResult;
use crate::dao::WebRbac;
use crate::model::ExportTaskModel;

pub const EXPORT_TYPE_APP_NOTIFY_LIST: &str = "app_notify_list";

/// 应用通知列表导出
pub struct AppNotifyListExporter {
    pub app_dao: Arc<AppDao>,
    pub web_rbac: Arc<WebRbac>,
}

impl Exporter for AppNotifyListExporter {
    fn check<'a>(
        &'a self,
        check_env: &'a RbacAccessCheckEnv<'_>,
        _app_id: u64,
        _app_user_id: u64,
        user_id: u64,
        _export_type: &'a str,
        _params: &'a serde_json::Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = WebResult<()>> + Send + 'a>>
    {
        Box::pin(async move {
            self.web_rbac
                .check(check_env, &CheckUserNotifyView { res_user_id: user_id })
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

            let app_id = params["app_id"].as_u64();
            let app_user_id = params["app_user_id"].as_u64();
            let notify_method = params["notify_method"].as_str();
            let notify_key = params["notify_key"].as_str();
            let status: Option<Vec<AppNotifyDataStatus>> = params["status"].as_array().map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_i64())
                    .filter_map(|v| AppNotifyDataStatus::try_from(v as i8).ok())
                    .collect()
            });

            let mut w = CsvWriter::new(&record)
                .header((
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

            w.finish().await
        })
    }
}
