// 用户APP文件列表导出
//
//   AppFileListExporter — 指定 APP 的文件列表
//     CSV 列: id, file_name, file_md5, file_size, storage_type, status, content_type, add_time
//   AppFileListExportCheck — 权限检查器

use std::path::PathBuf;
use std::sync::Arc;

use lsys_core::db::{CursorConfig, CursorLimit, CursorPageDir, CursorPageParam, CursorPageSort};
use lsys_file::dao::{FileDao, FileDataListParam, FileListAttrParam};

use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::user::CheckUserFileView;
use crate::dao::export_task::exporter::Exporter;
use crate::dao::export_task::writer::CsvWriter;
use crate::dao::{ExportTaskModel, WebExporterCheck, WebExportCheckParam, WebResult};

pub const EXPORT_TYPE_APP_FILE_LIST: &str = "app_file_list";

/// APP 文件列表权限检查器
pub struct AppFileListExportCheck {
    pub web_rbac: Arc<crate::dao::WebRbac>,
}

#[async_trait::async_trait]
impl WebExporterCheck for AppFileListExportCheck {
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

/// APP 文件列表导出器
pub struct AppFileListExporter {
    pub file_dao: Arc<FileDao>,
}

impl Exporter<crate::dao::WebError> for AppFileListExporter {
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
            let user_id = Some(record.user_id);
            let app_id = params["app_id"].as_u64();
            let status = params["status"].as_i64().map(|v| v as i8);
            let storage_type = params["storage_type"].as_str();
            let file_md5 = params["file_md5"].as_str();

            let filter = FileDataListParam {
                user_id,
                app_id,
                local_url: None,
                source_url: None,
                add_time_start: params["add_time_start"].as_u64(),
                add_time_end: params["add_time_end"].as_u64(),
                status,
                storage_type,
                file_md5,
                tag_names: None,
            };

            let attr = FileListAttrParam {
                attr_local: None,
                attr_oss: None,
                attr_tag_list: None,
                ..Default::default()
            };

            let fluent = fluent_mgr.locale(lang.as_deref());
            let mut w = CsvWriter::new(&record)
                .header(export_header!(
                    fluent,
                    EXPORT_TYPE_APP_FILE_LIST,
                    "id",
                    "file_name",
                    "file_md5",
                    "file_size",
                    "storage_type",
                    "status",
                    "content_type",
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
                    .file_dao
                    .data_dao()
                    .list_files(&filter, &page_param, &attr)
                    .await?;

                if items.is_empty() {
                    break;
                }

                let rows: Vec<_> = items
                    .iter()
                    .map(|item| {
                        (
                            item.item.id,
                            item.item.file_name.clone(),
                            item.item.file_md5.clone(),
                            item.item.file_size,
                            item.item.storage_type.clone(),
                            item.item.status,
                            item.item.content_type.clone(),
                            item.item.add_time,
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
