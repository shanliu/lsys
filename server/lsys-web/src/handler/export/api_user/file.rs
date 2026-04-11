// 文件相关列表导出（仅用户视角）
//
//   FileListExporter — 用户文件列表
//     CSV 列: id, file_name, file_md5, file_size, storage_type, status, content_type, add_time
//
//   FileLogExporter — 文件操作日志
//     CSV 列: id, file_id, file_chunk_id, message, user_id, add_time
//
//   FileChunkExporter — 文件分片列表
//     CSV 列: id, file_id, chunk_index, start_offset, chunk_md5, file_size, complete_size, status, add_time

use std::path::PathBuf;
use std::sync::Arc;

use lsys_core::db::{
    CursorConfig, CursorLimit, CursorPageDir, CursorPageParam, CursorPageSort, OffsetPageParam,
    OffsetPageValue,
};
use lsys_file::dao::{FileDao, FileDataListParam, FileListAttrParam};

use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::user::CheckUserFileView;
use crate::dao::export_task::exporter::Exporter;
use crate::dao::export_task::writer::CsvWriter;
use crate::dao::{ExportTaskModel, WebExporter, WebResult};

pub const EXPORT_TYPE_USER_FILE_LIST: &str = "user_file_list";
pub const EXPORT_TYPE_USER_FILE_LOG: &str = "user_file_log";
pub const EXPORT_TYPE_USER_FILE_CHUNK: &str = "user_file_chunk";

/// 用户文件列表导出
pub struct FileListExporter {
    pub file_dao: Arc<FileDao>,
    pub web_rbac: Arc<crate::dao::WebRbac>,
}

#[async_trait::async_trait]
impl WebExporter for FileListExporter {
    async fn check(
        &self,
        check_env: &RbacAccessCheckEnv<'_>,
        param: &crate::dao::ExportCheckParam<'_>,
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

impl Exporter<crate::dao::WebError> for FileListExporter {
    fn export<'a>(
        &'a self,
        record: ExportTaskModel,
        params: serde_json::Value,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<PathBuf, crate::dao::WebError>> + Send + 'a>,
    > {
        Box::pin(async move {
            let user_id = params["user_id"].as_u64();
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
                attr_tag: None,
            };

            let mut w = CsvWriter::new(&record)
                .header((
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

/// 文件操作日志导出
pub struct FileLogExporter {
    pub file_dao: Arc<FileDao>,
    pub web_rbac: Arc<crate::dao::WebRbac>,
}

#[async_trait::async_trait]
impl WebExporter for FileLogExporter {
    async fn check(
        &self,
        check_env: &RbacAccessCheckEnv<'_>,
        param: &crate::dao::ExportCheckParam<'_>,
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

impl Exporter<crate::dao::WebError> for FileLogExporter {
    fn export<'a>(
        &'a self,
        record: ExportTaskModel,
        params: serde_json::Value,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<PathBuf, crate::dao::WebError>> + Send + 'a>,
    > {
        Box::pin(async move {
            let file_id = params["file_id"].as_u64().unwrap_or(0);

            let mut w = CsvWriter::new(&record)
                .header((
                    "id",
                    "file_id",
                    "file_chunk_id",
                    "message",
                    "user_id",
                    "add_time",
                ))
                .await?;

            let total = self
                .file_dao
                .data_dao()
                .count_logs_by_file_id(file_id)
                .await? as u64;
            if total > 0 {
                let page = OffsetPageParam::new(Some(OffsetPageValue::new(0, total)));
                let items = self
                    .file_dao
                    .data_dao()
                    .list_logs_by_file_id(file_id, &page)
                    .await?;
                let rows: Vec<_> = items
                    .iter()
                    .map(|item| {
                        (
                            item.id,
                            item.file_id,
                            item.file_chunk_id,
                            item.message.clone(),
                            item.user_id,
                            item.add_time,
                        )
                    })
                    .collect();
                w.write_batch(rows).await?;
            }

            w.finish().await.map_err(Into::into)
        })
    }
}

/// 文件分片列表导出
pub struct FileChunkExporter {
    pub file_dao: Arc<FileDao>,
    pub web_rbac: Arc<crate::dao::WebRbac>,
}

#[async_trait::async_trait]
impl WebExporter for FileChunkExporter {
    async fn check(
        &self,
        check_env: &RbacAccessCheckEnv<'_>,
        param: &crate::dao::ExportCheckParam<'_>,
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

impl Exporter<crate::dao::WebError> for FileChunkExporter {
    fn export<'a>(
        &'a self,
        record: ExportTaskModel,
        params: serde_json::Value,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<PathBuf, crate::dao::WebError>> + Send + 'a>,
    > {
        Box::pin(async move {
            let file_id = params["file_id"].as_u64().unwrap_or(0);

            let mut w = CsvWriter::new(&record)
                .header((
                    "id",
                    "file_id",
                    "chunk_index",
                    "start_offset",
                    "chunk_md5",
                    "file_size",
                    "complete_size",
                    "status",
                    "add_time",
                ))
                .await?;

            let total = self
                .file_dao
                .data_dao()
                .count_chunks_by_file_id(file_id)
                .await? as u64;
            if total > 0 {
                let page = OffsetPageParam::new(Some(OffsetPageValue::new(0, total)));
                let items = self
                    .file_dao
                    .data_dao()
                    .list_chunks_by_file_id(file_id, &page)
                    .await?;
                let rows: Vec<_> = items
                    .iter()
                    .map(|item| {
                        (
                            item.id,
                            item.file_id,
                            item.chunk_index,
                            item.start_offset,
                            item.chunk_md5.clone(),
                            item.file_size,
                            item.complete_size,
                            item.status,
                            item.add_time,
                        )
                    })
                    .collect();
                w.write_batch(rows).await?;
            }

            w.finish().await.map_err(Into::into)
        })
    }
}
