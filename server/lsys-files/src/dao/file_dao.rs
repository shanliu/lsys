use std::path::PathBuf;
use std::sync::Arc;

use lsys_core::db::{
    CursorPageData, CursorPageParam, Insert, SqlQuote, SqlSuffix, TableMeta, Update,
};
use lsys_core::sql_format;
use lsys_core::{fluent_message, now_time, RequestEnv};
use lsys_logger::dao::ChangeLoggerDao;
use sqlx::{MySql, Pool};
use tokio::fs;
use tracing::warn;

use super::file_download::{DownloadTask, FileDownloadManager};
use super::file_helpers::{validate_chunks, ChunkInfo, FileHelper};
use super::file_log::FileLogDao;
use super::logger::*;
use super::*;
use crate::model::*;

/// 文件 DAO 主入口
pub struct FileDao {
    helper: Arc<FileHelper>,
    download_manager: Arc<FileDownloadManager>,
    logger: Arc<ChangeLoggerDao>,
}

/// 本地文件导入模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalFileMode {
    /// 移动源文件到存储目录（源文件将被删除）
    Move,
    /// 拷贝源文件到存储目录（保留源文件）
    Copy,
}

/// 文件列表过滤参数
#[derive(Debug, Default)]
pub struct FileListFilter {
    pub url: Option<String>,
    pub source_url: Option<String>,
    pub user_id: Option<u64>,
    pub app_id: Option<u64>,
    pub add_time_start: Option<u64>,
    pub add_time_end: Option<u64>,
    pub status: Option<i8>,
    pub storage_type: Option<String>,
    pub file_md5: Option<String>,
}

/// 文件列表返回结果 (file join file_user)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct FileListItem {
    // file fields
    pub id: u64,
    pub storage_type: String,
    pub status: i8,
    pub file_name: String,
    pub file_md5: String,
    pub file_size: u64,
    pub modify_time: u64,
    pub content_type: String,
    pub copy_file_id: u64,
    pub from_user_id: u64,
    pub add_time: u64,
    pub change_time: u64,
    // file_user fields
    pub file_user_id: u64,
    pub user_id: u64,
    pub app_id: u64,
    pub file_id: u64,
    pub file_user_status: i8,
    pub source_url: String,
    pub source_md5: String,
    pub file_user_add_time: u64,
    pub delete_time: u64,
}

/// 文件列表 attr 参数
/// 
/// 用于指定在列表查询中是否需要查询关联表的详细信息。
/// - attr_local: 为 true 时，对于 storage_type 为 "local" 的文件，查询并返回 file_local 表的关键信息
/// - attr_oss: 为 true 时，对于 storage_type 非 "local" 的文件，查询并返回 file_oss 表的关键信息
///
/// # 使用示例
/// ```ignore
/// let attr_param = FileListAttrParam {
///     attr_local: Some(true),
///     attr_oss: Some(true),
/// };
/// let (items, page_data) = file_dao.list_files(&filter, &page, &attr_param).await?;
/// // items 中的每个 FileListItemAttr 都可能包含 attr_local 或 attr_oss 信息
/// ```
#[derive(Debug, Default)]
pub struct FileListAttrParam {
    pub attr_local: Option<bool>,
    pub attr_oss: Option<bool>,
}

/// 本地文件属性（摊平后的关键数据）
/// 
/// 当 FileListAttrParam 中 attr_local 为 true 时，对于本地文件会返回此结构体中的信息，
/// 已将 file_local 表的关键字段摊平在此结构体中。
/// 
/// 注意：本结构体不进行序列化，由调用端负责摊平数据并输出到 JSON。
#[derive(Debug, Clone)]
pub struct FileLocalAttr {
    pub id: u64,
    pub source_type: i8,
    pub source_name: String,
    pub oss_file_id: u64,
    pub local_path: String,
    pub file_chunk_total: u32,
    pub file_chunk_succ: u32,
    pub file_chunk_size: u64,
    pub last_error: String,
}

/// OSS 文件属性（摊平后的关键数据）
/// 
/// 当 FileListAttrParam 中 attr_oss 为 true 时，对于 OSS 文件会返回此结构体中的信息，
/// 已将 file_oss 表的关键字段摊平在此结构体中。
/// 
/// 注意：本结构体不进行序列化，由调用端负责摊平数据并输出到 JSON。
#[derive(Debug, Clone)]
pub struct FileOssAttr {
    pub id: u64,
    pub object_key: String,
    pub local_file_id: u64,
    pub object_url: String,
    pub bucket: String,
    pub region: String,
    pub size: u64,
    pub last_error: String,
}

/// 文件列表返回结果（包含 attr 属性）
/// 
/// 本结构体由 DAO 层返回，包含基础信息 item 和可选的关联表属性 attr_local/attr_oss。
/// 不进行序列化，由调用端（web 层）负责摊平所有数据并生成 JSON 响应。
/// 
/// # 调用端摊平示例
/// ```ignore
/// let (items_with_attrs, page) = file_dao.list_files(
///     &filter, 
///     &page, 
///     &FileListAttrParam {
///         attr_local: Some(true),
///         attr_oss: Some(true),
///     }
/// ).await?;
/// 
/// let json_items: Vec<serde_json::Value> = items_with_attrs
///     .into_iter()
///     .map(|item| {
///         let mut obj = serde_json::json!({
///             "id": item.item.id,
///             "file_name": item.item.file_name,
///             "file_size": item.item.file_size,
///             // ... 摊平 FileListItem 的所有字段
///         });
///         
///         // 摊平 attr_local
///         if let Some(local) = item.attr_local {
///             obj["local_path"] = json!(local.local_path);
///             obj["file_chunk_total"] = json!(local.file_chunk_total);
///             // ... 其他字段
///         }
///         
///         // 摊平 attr_oss
///         if let Some(oss) = item.attr_oss {
///             obj["object_url"] = json!(oss.object_url);
///             obj["bucket"] = json!(oss.bucket);
///             // ... 其他字段
///         }
///         
///         obj
///     })
///     .collect();
/// ```
#[derive(Debug, Clone)]
pub struct FileListItemAttr {
    pub item: FileListItem,
    pub attr_local: Option<FileLocalAttr>,
    pub attr_oss: Option<FileOssAttr>,
}

impl FileDao {
    pub fn new(
        helper: Arc<FileHelper>,
        download_manager: Arc<FileDownloadManager>,
        logger: Arc<ChangeLoggerDao>,
    ) -> Self {
        Self {
            helper,
            download_manager,
            logger,
        }
    }

    pub fn helper(&self) -> &FileHelper {
        &self.helper
    }

    pub(crate) fn db(&self) -> &Pool<MySql> {
        self.helper.db()
    }

    pub(crate) fn log_dao(&self) -> &FileLogDao {
        self.helper.log_dao()
    }

    pub(crate) fn logger(&self) -> &ChangeLoggerDao {
        &self.logger
    }

    // ==================== 创建方法 1: OSS 远程文件 ====================
    pub async fn create_from_oss(
        &self,
        oss_result: OssResult,
        user_id: u64,
        app_id: u64,
        storage_type: &str,
        env_data: Option<&RequestEnv>,
    ) -> FileResult<FileModel> {
        use tracing::info;

        info!(
            "create_from_oss: starting, user_id={}, storage_type={}, file_md5={}",
            user_id, storage_type, &oss_result.file_md5
        );
        let now = now_time()?;

        // 辅助函数.1: 检测是否存在
        if let Some(existing) = self
            .helper
            .find_existing_file(storage_type, &oss_result.file_md5)
            .await?
        {
            info!("create_from_oss: existing file found, id={}", existing.id);
            // 检查 file_user 是否已存在
            if let Some(_fu) = self
                .helper
                .find_file_user(user_id, app_id, existing.id, FileUserStatus::Normal)
                .await?
            {
                info!("create_from_oss: user already linked to existing file");
                self.log_dao()
                    .add(
                        existing.id,
                        0,
                        user_id,
                        "create_from_oss: existing file, user already linked",
                        None,
                    )
                    .await;
                return Ok(existing);
            }

            info!("create_from_oss: creating file_user link to existing file");
            // 创建 file_user
            let mut tx = self.db().begin().await?;
            let tx_result: FileResult<()> = async {
                Insert::<FileUserModel>::new()
                    .set(FileUserModel::USER_ID, user_id)
                    .set(FileUserModel::APP_ID, app_id)
                    .set(FileUserModel::FILE_ID, existing.id)
                    .set(FileUserModel::STATUS, FileUserStatus::Normal as i8)
                    .set(
                        FileUserModel::SOURCE_URL,
                        oss_result.source_url.as_deref().unwrap_or(""),
                    )
                    .set(FileUserModel::SOURCE_MD5, "")
                    .set(FileUserModel::ADD_TIME, now)
                    .set(FileUserModel::DELETE_TIME, 0u64)
                    .execute(&mut *tx)
                    .await?;
                self.log_dao()
                    .add(
                        existing.id,
                        0,
                        user_id,
                        "create_from_oss: existing file, created file_user",
                        Some(&mut tx),
                    )
                    .await;
                Ok(())
            }
            .await;
            match tx_result {
                Ok(_) => {
                    tx.commit().await?;
                    return Ok(existing);
                }
                Err(e) => {
                    let _ = tx.rollback().await;
                    return Err(e);
                }
            }
        }

        // 不存在: 创建新记录
        let mut tx = self.db().begin().await?;

        let tx_result: FileResult<u64> = async {
            let file_res = Insert::<FileModel>::new()
                .set(FileModel::STORAGE_TYPE, storage_type)
                .set(FileModel::STATUS, FileStatus::Normal as i8)
                .set(FileModel::FILE_MD5, &oss_result.file_md5)
                .set(FileModel::FILE_SIZE, oss_result.file_size.unwrap_or(0))
                .set(
                    FileModel::FILE_NAME,
                    oss_result.file_name.as_deref().unwrap_or(""),
                )
                .set(
                    FileModel::CONTENT_TYPE,
                    oss_result.content_type.as_deref().unwrap_or(""),
                )
                .set(FileModel::MODIFY_TIME, oss_result.modify_time.unwrap_or(0))
                .set(FileModel::FROM_USER_ID, user_id)
                .set(FileModel::ADD_TIME, now)
                .set(FileModel::CHANGE_TIME, 0u64)
                .set(FileModel::COPY_FILE_ID, 0u64)
                .execute(&mut *tx)
                .await?;

            let file_id = file_res.last_insert_id();

            let object_url_md5 = FileHelper::compute_str_md5(&oss_result.object_url);

            Insert::<FileOssModel>::new()
                .set(FileOssModel::FILE_ID, file_id)
                .set(FileOssModel::OBJECT_KEY, &oss_result.object_key)
                .set(FileOssModel::BUCKET, &oss_result.bucket)
                .set(FileOssModel::OBJECT_URL, &oss_result.object_url)
                .set(FileOssModel::OBJECT_URL_MD5, &object_url_md5)
                .set(
                    FileOssModel::REGION,
                    oss_result.region.as_deref().unwrap_or(""),
                )
                .set(FileOssModel::SIZE, oss_result.file_size.unwrap_or(0))
                .set(
                    FileOssModel::LOCAL_FILE_ID,
                    oss_result.local_file_id.unwrap_or(0),
                )
                .set(FileOssModel::LAST_ERROR, "")
                .execute(&mut *tx)
                .await?;

            Insert::<FileUserModel>::new()
                .set(FileUserModel::USER_ID, user_id)
                .set(FileUserModel::APP_ID, app_id)
                .set(FileUserModel::FILE_ID, file_id)
                .set(FileUserModel::STATUS, FileUserStatus::Normal as i8)
                .set(
                    FileUserModel::SOURCE_URL,
                    oss_result.source_url.as_deref().unwrap_or(""),
                )
                .set(FileUserModel::SOURCE_MD5, "")
                .set(FileUserModel::ADD_TIME, now)
                .set(FileUserModel::DELETE_TIME, 0u64)
                .execute(&mut *tx)
                .await?;

            self.log_dao()
                .add(
                    file_id,
                    0,
                    user_id,
                    "create_from_oss: new file created",
                    Some(&mut tx),
                )
                .await;

            Ok(file_id)
        }
        .await;

        let file_id = match tx_result {
            Ok(id) => {
                tx.commit().await?;
                id
            }
            Err(e) => {
                let _ = tx.rollback().await;
                return Err(e);
            }
        };

        let file = self
            .helper
            .find_file_by_id(file_id)
            .await?
            .ok_or_else(|| FileError::System(fluent_message!("file-create-error")))?;

        self.logger
            .add(
                &LogFileCreate {
                    action: "create_from_oss",
                    storage_type,
                    user_id,
                    file_id: file.id,
                    file_md5: &file.file_md5,
                },
                Some(file.id),
                Some(user_id),
                None,
                env_data,
            )
            .await;

        Ok(file)
    }

    // ==================== 创建方法 2: 从URL下载远程文件 ====================
    pub async fn create_from_url(
        &self,
        source_url: &str,
        user_id: u64,
        app_id: u64,
        chunks: &[ChunkInfo],
        content_type: Option<&str>,
        env_data: Option<&RequestEnv>,
    ) -> FileResult<u64> {
        use tracing::info;

        info!(
            "create_from_url: starting, user_id={}, url={}",
            user_id, source_url
        );
        let trimmed_url = source_url.trim();
        let source_md5 = FileHelper::compute_str_md5(trimmed_url);
        let now = now_time()?;

        // 查询是否已存在
        if let Some(existing_fu) = self
            .helper
            .find_file_user_by_source_md5(user_id, app_id, &source_md5, FileUserStatus::Normal)
            .await?
        {
            info!(
                "create_from_url: existing file_user found, id={}",
                existing_fu.id
            );
            return Ok(existing_fu.id);
        }

        let total_size = validate_chunks(chunks)?;
        info!(
            "create_from_url: creating new file, chunks={}, total_size={}",
            chunks.len(),
            total_size
        );

        let mut tx = self.db().begin().await?;

        let tx_result: FileResult<u64> = async {
            let file_res = Insert::<FileModel>::new()
                .set(FileModel::STORAGE_TYPE, FileModel::STORAGE_TYPE_LOCAL)
                .set(FileModel::STATUS, FileStatus::Unfinished as i8)
                .set(FileModel::FILE_SIZE, total_size)
                .set(FileModel::FILE_MD5, "")
                .set(
                    FileModel::FILE_NAME,
                    FileHelper::extract_filename_from_url(trimmed_url),
                )
                .set(FileModel::CONTENT_TYPE, content_type.unwrap_or(""))
                .set(FileModel::MODIFY_TIME, 0u64)
                .set(FileModel::FROM_USER_ID, user_id)
                .set(FileModel::ADD_TIME, now)
                .set(FileModel::CHANGE_TIME, 0u64)
                .set(FileModel::COPY_FILE_ID, 0u64)
                .execute(&mut *tx)
                .await?;

            let file_id = file_res.last_insert_id();

            let chunk_total = if chunks.len() > 1 {
                chunks.len() as u32
            } else {
                0u32
            };

            Insert::<FileLocalModel>::new()
                .set(FileLocalModel::FILE_ID, file_id)
                .set(FileLocalModel::SOURCE_TYPE, FileSourceType::Url as i8)
                .set(FileLocalModel::SOURCE_NAME, "")
                .set(FileLocalModel::OSS_FILE_ID, 0u64)
                .set(FileLocalModel::LOCAL_PATH, "")
                .set(FileLocalModel::FILE_CHUNK_TOTAL, chunk_total)
                .set(FileLocalModel::FILE_CHUNK_SUCC, 0u32)
                .set(FileLocalModel::FILE_CHUNK_SIZE, 0u64)
                .set(FileLocalModel::LAST_ERROR, "")
                .execute(&mut *tx)
                .await?;

            // 如果有多个 chunk, 创建 file_local_chunk 记录
            if chunks.len() > 1 {
                for (idx, chunk) in chunks.iter().enumerate() {
                    Insert::<FileLocalChunkModel>::new()
                        .set(FileLocalChunkModel::FILE_ID, file_id)
                        .set(FileLocalChunkModel::CHUNK_INDEX, idx as u32)
                        .set(FileLocalChunkModel::START_OFFSET, chunk.offset)
                        .set(FileLocalChunkModel::CHUNK_MD5, "")
                        .set(FileLocalChunkModel::UPLOAD_MD5, "")
                        .set(FileLocalChunkModel::CHUNK_PATH, "")
                        .set(FileLocalChunkModel::FILE_SIZE, chunk.len)
                        .set(FileLocalChunkModel::COMPLETE_SIZE, 0u64)
                        .set(
                            FileLocalChunkModel::STATUS,
                            FileChunkStatus::Unfinished as i8,
                        )
                        .set(FileLocalChunkModel::ADD_TIME, now)
                        .set(FileLocalChunkModel::CHANGE_TIME, 0u64)
                        .execute(&mut *tx)
                        .await?;
                }
            }

            let fu_res = Insert::<FileUserModel>::new()
                .set(FileUserModel::USER_ID, user_id)
                .set(FileUserModel::APP_ID, app_id)
                .set(FileUserModel::FILE_ID, file_id)
                .set(FileUserModel::STATUS, FileUserStatus::Normal as i8)
                .set(FileUserModel::SOURCE_URL, trimmed_url)
                .set(FileUserModel::SOURCE_MD5, &source_md5)
                .set(FileUserModel::ADD_TIME, now)
                .set(FileUserModel::DELETE_TIME, 0u64)
                .execute(&mut *tx)
                .await?;

            let file_user_id = fu_res.last_insert_id();

            self.log_dao()
                .add(
                    file_id,
                    0,
                    user_id,
                    &format!("create_from_url: file created, chunks={}", chunks.len()),
                    Some(&mut tx),
                )
                .await;

            Ok(file_user_id)
        }
        .await;

        let file_user_id = match tx_result {
            Ok(id) => {
                tx.commit().await?;
                id
            }
            Err(e) => {
                let _ = tx.rollback().await;
                return Err(e);
            }
        };

        // 触发下载
        if chunks.len() > 1 {
            for idx in 0..chunks.len() {
                self.download_manager.push(DownloadTask {
                    file_user_id,
                    chunk_index: idx as u32,
                });
            }
        } else {
            self.download_manager.push(DownloadTask {
                file_user_id,
                chunk_index: 0,
            });
        }

        self.logger
            .add(
                &LogFileCreate {
                    action: "create_from_url",
                    storage_type: FileModel::STORAGE_TYPE_LOCAL,
                    user_id,
                    file_id: 0,
                    file_md5: "",
                },
                None,
                Some(user_id),
                None,
                env_data,
            )
            .await;

        Ok(file_user_id)
    }

    // ==================== 创建方法 3: 本地上传 ====================

    /// 3.1 通过已有 FILE 对象创建 file_user 关联
    pub async fn create_file_user(
        &self,
        file: &FileModel,
        user_id: u64,
        app_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> FileResult<u64> {
        let now = now_time()?;
        let res = Insert::<FileUserModel>::new()
            .set(FileUserModel::USER_ID, user_id)
            .set(FileUserModel::APP_ID, app_id)
            .set(FileUserModel::FILE_ID, file.id)
            .set(FileUserModel::STATUS, FileUserStatus::Normal as i8)
            .set(FileUserModel::SOURCE_URL, "")
            .set(FileUserModel::SOURCE_MD5, "")
            .set(FileUserModel::ADD_TIME, now)
            .set(FileUserModel::DELETE_TIME, 0u64)
            .execute(self.db())
            .await?;

        self.logger
            .add(
                &LogFileCreate {
                    action: "create_file_user",
                    storage_type: &file.storage_type,
                    user_id,
                    file_id: file.id,
                    file_md5: &file.file_md5,
                },
                Some(file.id),
                Some(user_id),
                None,
                env_data,
            )
            .await;

        Ok(res.last_insert_id())
    }

    // ==================== 创建方法 2.5: 根据文件 MD5 秒传 ====================

    /// 根据文件 MD5 查找已有文件并创建关联记录（秒传）
    ///
    /// 客户端先计算文件 MD5，调用此方法判断服务端是否已存在相同文件。
    /// 若存在则直接创建 file_user 关联记录，无需上传文件数据。
    ///
    /// 返回 `Ok(Some(file_user_id))` 表示秒传成功；
    /// 返回 `Ok(None)` 表示文件不存在，需要走正常上传流程。
    pub async fn create_from_md5(
        &self,
        file_md5: &str,
        user_id: u64,
        app_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> FileResult<Option<u64>> {
        if file_md5.is_empty() {
            return Err(FileError::Param(fluent_message!("file-md5-empty")));
        }

        let existing = self
            .helper
            .find_existing_file(FileModel::STORAGE_TYPE_LOCAL, file_md5)
            .await?;

        match existing {
            Some(file) => {
                let file_user_id = self.create_file_user(&file, user_id, app_id, env_data).await?;
                Ok(Some(file_user_id))
            }
            None => Ok(None),
        }
    }

    // ==================== 创建方法 4: 已知本地文件生成 ====================
    #[allow(clippy::too_many_arguments)]
    pub async fn create_from_local_file(
        &self,
        local_file_path: &str,
        user_id: u64,
        app_id: u64,
        file_name: Option<&str>,
        mode: LocalFileMode,
        copy_file_id: Option<u64>,
        env_data: Option<&RequestEnv>,
    ) -> FileResult<FileModel> {
        use tracing::info;

        info!(
            "create_from_local_file: starting, user_id={}, path={}",
            user_id, local_file_path
        );
        let path = PathBuf::from(local_file_path);
        let now = now_time()?;

        // 计算 MD5
        let file_md5 = self.helper.compute_file_md5(&path).await?;
        info!("create_from_local_file: computed md5={}", file_md5);

        // copy_file_id 有值时跳过去重，强制创建独立的新文件记录
        if copy_file_id.is_none() {
        // 辅助函数.1: 检查是否存在
        if let Some(existing) = self
            .helper
            .find_existing_file(FileModel::STORAGE_TYPE_LOCAL, &file_md5)
            .await?
        {
            info!(
                "create_from_local_file: existing file found, id={}",
                existing.id
            );

            // 检查实际物理文件是否存在，不存在则当记录不存在处理
            let physical_exists = if let Some(local_rec) =
                self.helper.find_file_local_by_file_id(existing.id).await?
            {
                if local_rec.local_path.is_empty() {
                    false
                } else {
                    let full = self.helper.get_full_local_path(&local_rec.local_path);
                    tokio::fs::metadata(&full).await.is_ok()
                }
            } else {
                false
            };

            if !physical_exists {
                info!(
                    "create_from_local_file: existing record id={} but physical file missing, treating as new",
                    existing.id
                );
            }

            if physical_exists {
                if let Some(_fu) = self
                    .helper
                    .find_file_user(user_id, app_id, existing.id, FileUserStatus::Normal)
                    .await?
                {
                    // 已存在, Move 模式下删除源文件
                    info!("create_from_local_file: user already linked");
                    if mode == LocalFileMode::Move {
                        info!("create_from_local_file: deleting source file (Move mode)");
                        if let Err(e) = fs::remove_file(local_file_path).await {
                            warn!("create_from_local: remove source file failed: {}", e);
                        }
                    }
                    self.log_dao()
                        .add(
                            existing.id,
                            0,
                            user_id,
                            "create_from_local: existing file+user, deleted source",
                            None,
                        )
                        .await;
                    return Ok(existing);
                }

                // 创建 file_user
                info!("create_from_local_file: creating file_user link to existing file");
                let mut tx = self.db().begin().await?;
                let tx_result: FileResult<()> = async {
                    Insert::<FileUserModel>::new()
                        .set(FileUserModel::USER_ID, user_id)
                        .set(FileUserModel::APP_ID, app_id)
                        .set(FileUserModel::FILE_ID, existing.id)
                        .set(FileUserModel::STATUS, FileUserStatus::Normal as i8)
                        .set(FileUserModel::SOURCE_URL, "")
                        .set(FileUserModel::SOURCE_MD5, "")
                        .set(FileUserModel::ADD_TIME, now)
                        .set(FileUserModel::DELETE_TIME, 0u64)
                        .execute(&mut *tx)
                        .await?;
                    self.log_dao()
                        .add(
                            existing.id,
                            0,
                            user_id,
                            "create_from_local: existing file, new file_user",
                            Some(&mut tx),
                        )
                        .await;
                    Ok(())
                }
                .await;
                match tx_result {
                    Ok(_) => {
                        tx.commit().await?;
                    }
                    Err(e) => {
                        let _ = tx.rollback().await;
                        return Err(e);
                    }
                }

                if mode == LocalFileMode::Move {
                    if let Err(e) = fs::remove_file(local_file_path).await {
                        warn!("create_from_local: remove source file failed: {}", e);
                    }
                }
                return Ok(existing);
            } // end if physical_exists
        }
        } // end if copy_file_id.is_none()

        // 不存在或物理文件丢失: 移动/拷贝文件到存储路径
        info!("create_from_local_file: no existing file, creating new record");
        let actual_name = file_name.unwrap_or_else(|| {
            path.file_name()
                .map(|n| n.to_str().unwrap_or("unknown"))
                .unwrap_or("unknown")
        });

        let relative_path = match mode {
            LocalFileMode::Move => {
                self.helper
                    .move_file_to_storage(local_file_path, Some(actual_name))
                    .await?
            }
            LocalFileMode::Copy => {
                self.helper
                    .copy_file_to_storage(local_file_path, Some(actual_name))
                    .await?
            }
        };

        let full_path = self.helper.get_full_local_path(&relative_path);
        let metadata = tokio::fs::metadata(&full_path).await?;
        let content_type = get_content_type(&full_path).await?;
        let modify_time = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let mut tx = self.db().begin().await?;

        let tx_result: FileResult<u64> = async {
            let file_res = Insert::<FileModel>::new()
                .set(FileModel::STORAGE_TYPE, FileModel::STORAGE_TYPE_LOCAL)
                .set(FileModel::STATUS, FileStatus::Normal as i8)
                .set(FileModel::FILE_MD5, &file_md5)
                .set(FileModel::FILE_SIZE, metadata.len())
                .set(FileModel::FILE_NAME, actual_name)
                .set(FileModel::CONTENT_TYPE, &content_type)
                .set(FileModel::MODIFY_TIME, modify_time)
                .set(FileModel::FROM_USER_ID, user_id)
                .set(FileModel::ADD_TIME, now)
                .set(FileModel::CHANGE_TIME, 0u64)
                .set(FileModel::COPY_FILE_ID, copy_file_id.unwrap_or(0))
                .execute(&mut *tx)
                .await?;

            let file_id = file_res.last_insert_id();

            Insert::<FileLocalModel>::new()
                .set(FileLocalModel::FILE_ID, file_id)
                .set(FileLocalModel::SOURCE_TYPE, FileSourceType::LocalPath as i8)
                .set(FileLocalModel::SOURCE_NAME, local_file_path)
                .set(FileLocalModel::OSS_FILE_ID, 0u64)
                .set(FileLocalModel::LOCAL_PATH, &relative_path)
                .set(FileLocalModel::FILE_CHUNK_TOTAL, 0u32)
                .set(FileLocalModel::FILE_CHUNK_SUCC, 0u32)
                .set(FileLocalModel::FILE_CHUNK_SIZE, 0u64)
                .set(FileLocalModel::LAST_ERROR, "")
                .execute(&mut *tx)
                .await?;

            Insert::<FileUserModel>::new()
                .set(FileUserModel::USER_ID, user_id)
                .set(FileUserModel::APP_ID, app_id)
                .set(FileUserModel::FILE_ID, file_id)
                .set(FileUserModel::STATUS, FileUserStatus::Normal as i8)
                .set(FileUserModel::SOURCE_URL, "")
                .set(FileUserModel::SOURCE_MD5, "")
                .set(FileUserModel::ADD_TIME, now)
                .set(FileUserModel::DELETE_TIME, 0u64)
                .execute(&mut *tx)
                .await?;

            self.log_dao()
                .add(
                    file_id,
                    0,
                    user_id,
                    "create_from_local: new file created",
                    Some(&mut tx),
                )
                .await;

            Ok(file_id)
        }
        .await;

        let file_id = match tx_result {
            Ok(id) => {
                tx.commit().await?;
                id
            }
            Err(e) => {
                let _ = tx.rollback().await;
                // 事务失败时尝试删除已移动的文件
                let rp = relative_path.clone();
                let bp = self.helper.config().storage_base_path.clone();
                tokio::spawn(async move {
                    let full = std::path::Path::new(&bp).join(&rp);
                    if let Err(e) = tokio::fs::remove_file(&full).await {
                        tracing::warn!("create_from_local: rollback remove file failed: {}", e);
                    }
                });
                return Err(e);
            }
        };

        let file = self
            .helper
            .find_file_by_id(file_id)
            .await?
            .ok_or_else(|| FileError::System(fluent_message!("file-create-error")))?;

        self.logger
            .add(
                &LogFileCreate {
                    action: "create_from_local_file",
                    storage_type: FileModel::STORAGE_TYPE_LOCAL,
                    user_id,
                    file_id: file.id,
                    file_md5: &file.file_md5,
                },
                Some(file.id),
                Some(user_id),
                None,
                env_data,
            )
            .await;

        Ok(file)
    }

    // ==================== 操作方法 1: OSS 同步到本地 ====================
    pub async fn sync_oss_to_local(
        &self,
        file: &FileModel,
        oss_provider: &dyn OssProvider,
        env_data: Option<&RequestEnv>,
    ) -> FileResult<FileModel> {
        if file.storage_type == FileModel::STORAGE_TYPE_LOCAL {
            return Err(FileError::Param(fluent_message!("file-must-be-oss-type")));
        }

        // 检查是否已有本地副本: 查找 file_local 中 oss_file_id 等于当前 OSS 文件 ID 的记录
        if let Some(local) = self.helper.find_file_local_by_oss_file_id(file.id).await? {
            // 找到了从该 OSS 文件同步而来的本地文件记录，返回对应的本地 file
            if let Some(local_file) = self.helper.find_file_by_id(local.file_id).await? {
                return Ok(local_file);
            }
        }

        // 下载 OSS 文件到本地
        let file_oss = self
            .helper
            .find_file_oss_by_file_id(file.id)
            .await?
            .ok_or_else(|| FileError::Param(fluent_message!("file-oss-not-found")))?;

        let oss_ext = FileHelper::extract_extension(&file.file_name);
        let (_rel_path, full_path) = self
            .helper
            .create_new_file(&format!("oss_sync.{}", oss_ext))
            .await?;
        oss_provider
            .download_to_local(&file_oss, full_path.to_str().unwrap_or(""))
            .await?;

        // 用创建方法4创建本地文件记录
        let local_file = self
            .create_from_local_file(
                full_path.to_str().unwrap_or(""),
                file.from_user_id,
                0, // 系统内部同步，app_id=0
                Some(&file.file_name),
                LocalFileMode::Move,
                None,
                env_data,
            )
            .await?;

        // 更新 file_local.oss_file_id 指向源 OSS 文件，以便下次查找已有副本
        if let Some(local_rec) = self
            .helper
            .find_file_local_by_file_id(local_file.id)
            .await?
        {
            Update::<FileLocalModel>::new()
                .set(FileLocalModel::OSS_FILE_ID, file.id)
                .execute(
                    SqlSuffix::Where(&sql_format!("id={}", local_rec.id)),
                    self.db(),
                )
                .await?;
        }

        self.logger
            .add(
                &LogFileSync {
                    action: "sync_oss_to_local",
                    user_id: file.from_user_id,
                    file_id: file.id,
                    storage_type: &file.storage_type,
                },
                Some(file.id),
                Some(file.from_user_id),
                None,
                env_data,
            )
            .await;

        Ok(local_file)
    }

    // ==================== 操作方法 2: 本地文件上传到 OSS ====================
    pub async fn upload_local_to_oss(
        &self,
        file: &FileModel,
        storage_type: &str,
        oss_provider: &dyn OssProvider,
        env_data: Option<&RequestEnv>,
    ) -> FileResult<FileModel> {
        if file.storage_type != FileModel::STORAGE_TYPE_LOCAL {
            return Err(FileError::Param(fluent_message!("file-must-be-local-type")));
        }

        // 检查是否已有 OSS 副本
        let existing_oss = sqlx::query_as::<_, FileOssModel>(&sql_format!(
            "SELECT fo.* FROM {} fo INNER JOIN {} f ON fo.file_id=f.id WHERE fo.local_file_id={} AND f.storage_type={} LIMIT 1",
            FileOssModel::table_name(),
            FileModel::table_name(),
            file.id,
            storage_type
        ))
        .fetch_optional(self.db())
        .await?;

        if let Some(oss) = existing_oss {
            if let Some(oss_file) = self.helper.find_file_by_id(oss.file_id).await? {
                return Ok(oss_file);
            }
        }

        // 获取本地文件路径
        let local = self
            .helper
            .find_file_local_by_file_id(file.id)
            .await?
            .ok_or_else(|| FileError::Param(fluent_message!("file-local-not-found")))?;

        let full_path = self.helper.get_full_local_path(&local.local_path);

        // 上传到 OSS
        let mut oss_result = oss_provider
            .upload_from_local(full_path.to_str().unwrap_or(""), file)
            .await?;
        oss_result.local_file_id = Some(file.id);

        // 通过创建方法1创建OSS记录
        let oss_file = self
            .create_from_oss(oss_result, file.from_user_id, 0, storage_type, env_data)
            .await?;

        self.logger
            .add(
                &LogFileSync {
                    action: "upload_local_to_oss",
                    user_id: file.from_user_id,
                    file_id: file.id,
                    storage_type,
                },
                Some(file.id),
                Some(file.from_user_id),
                None,
                env_data,
            )
            .await;

        Ok(oss_file)
    }

    // ==================== 操作方法 3: 转换为URL访问地址 ====================
    pub async fn get_file_url(&self, file: &FileModel) -> FileResult<Option<String>> {
        let urls = self.get_file_urls(std::slice::from_ref(file)).await?;
        Ok(urls.get(&file.id).cloned())
    }

    /// 批量获取文件 URL
    ///
    /// 传入多个 FileModel，一次性查询所有 local / oss 记录，返回 file_id -> url 的映射。
    /// 仅状态为 Normal 的文件才会返回 URL。
    pub async fn get_file_urls(
        &self,
        files: &[FileModel],
    ) -> FileResult<std::collections::HashMap<u64, String>> {
        use std::collections::HashMap;

        let mut result: HashMap<u64, String> = HashMap::new();
        if files.is_empty() {
            return Ok(result);
        }

        // 按存储类型分组，仅处理 Normal 状态
        let mut local_ids: Vec<u64> = Vec::new();
        let mut oss_ids: Vec<u64> = Vec::new();
        for f in files {
            if !FileStatus::Normal.eq(f.status) {
                continue;
            }
            if f.storage_type == FileModel::STORAGE_TYPE_LOCAL {
                local_ids.push(f.id);
            } else {
                oss_ids.push(f.id);
            }
        }

        // 批量查询 local 记录
        if !local_ids.is_empty() {
            let id_str: Vec<String> = local_ids.iter().map(|i| i.to_string()).collect();
            let sql = format!(
                "SELECT * FROM {} WHERE file_id IN ({})",
                FileLocalModel::table_name().sql_quote(),
                id_str.join(",")
            );
            let locals: Vec<FileLocalModel> =
                sqlx::query_as::<_, FileLocalModel>(&sql)
                    .fetch_all(self.db())
                    .await?;
            let prefix = &self.helper.config().local_file_url_prefix;
            for local_rec in &locals {
                if !local_rec.local_path.is_empty() {
                    result.insert(local_rec.file_id, format!("{}{}", prefix, local_rec.local_path));
                }
            }
        }

        // 批量查询 oss 记录
        if !oss_ids.is_empty() {
            let id_str: Vec<String> = oss_ids.iter().map(|i| i.to_string()).collect();
            let sql = format!(
                "SELECT * FROM {} WHERE file_id IN ({})",
                FileOssModel::table_name().sql_quote(),
                id_str.join(",")
            );
            let osses: Vec<FileOssModel> =
                sqlx::query_as::<_, FileOssModel>(&sql)
                    .fetch_all(self.db())
                    .await?;
            for oss_rec in &osses {
                if !oss_rec.object_url.is_empty() {
                    result.insert(oss_rec.file_id, oss_rec.object_url.clone());
                }
            }
        }

        Ok(result)
    }

    // ==================== 操作方法 4: 获取本地文件路径 ====================
    pub async fn get_local_path(
        &self,
        file: &FileModel,
        oss_provider: Option<&dyn OssProvider>,
    ) -> FileResult<Option<PathBuf>> {
        if file.storage_type == FileModel::STORAGE_TYPE_LOCAL {
            let local = self.helper.find_file_local_by_file_id(file.id).await?;
            if let Some(local_rec) = local {
                if local_rec.local_path.is_empty() {
                    return Ok(None);
                }
                return Ok(Some(self.helper.get_full_local_path(&local_rec.local_path)));
            }
            Ok(None)
        } else {
            // 非local类型, 先同步到本地
            if let Some(provider) = oss_provider {
                let local_file = self.sync_oss_to_local(file, provider, None).await?;
                let local = self
                    .helper
                    .find_file_local_by_file_id(local_file.id)
                    .await?;
                if let Some(local_rec) = local {
                    if local_rec.local_path.is_empty() {
                        return Ok(None);
                    }
                    return Ok(Some(self.helper.get_full_local_path(&local_rec.local_path)));
                }
            }
            Ok(None)
        }
    }

    // ==================== 操作方法 5: 拷贝函数 ====================
    pub fn copy_file<'a>(
        &'a self,
        file: &'a FileModel,
        user_id: u64,
        app_id: u64,
        oss_provider: Option<&'a dyn OssProvider>,
        env_data: Option<&'a RequestEnv>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = FileResult<FileModel>> + Send + 'a>>
    {
        Box::pin(async move {
            if !FileStatus::Normal.eq(file.status) {
                return Err(FileError::Param(fluent_message!(
                    "file-status-must-be-normal"
                )));
            }

            if file.storage_type == FileModel::STORAGE_TYPE_LOCAL {
                // 拷贝本地文件
                let local = self
                    .helper
                    .find_file_local_by_file_id(file.id)
                    .await?
                    .ok_or_else(|| FileError::Param(fluent_message!("file-local-not-found")))?;

                let src_full = self.helper.get_full_local_path(&local.local_path);
                let (_rel, dst_full) = self.helper.create_new_file(&file.file_name).await?;
                fs::copy(&src_full, &dst_full).await?;

                let new_file = self
                    .create_from_local_file(
                        dst_full.to_str().unwrap_or(""),
                        user_id,
                        app_id,
                        Some(&file.file_name),
                        LocalFileMode::Move,
                        Some(file.id),
                        env_data,
                    )
                    .await?;

                self.logger
                    .add(
                        &LogFileCopy {
                            user_id,
                            source_file_id: file.id,
                            new_file_id: new_file.id,
                        },
                        Some(new_file.id),
                        Some(user_id),
                        None,
                        env_data,
                    )
                    .await;

                Ok(new_file)
            } else {
                // OSS类型: 先同步到本地再拷贝
                let provider = oss_provider.ok_or_else(|| {
                    FileError::Param(fluent_message!("file-oss-provider-required"))
                })?;
                let local_file = self.sync_oss_to_local(file, provider, env_data).await?;
                self.copy_file(&local_file, user_id, app_id, None, env_data).await
            }
        })
    }

    // ==================== 操作方法 7: 列表接口 ====================

    /// 构建文件列表查询的 WHERE 条件
    ///
    /// 返回 `Ok(None)` 表示 URL 过滤条件匹配不到任何文件，应直接返回空结果。
    /// 返回 `Ok(Some(where_clauses))` 表示成功构建 WHERE 条件列表。
    async fn build_file_list_where(
        &self,
        filter: &FileListFilter,
    ) -> FileResult<Option<Vec<String>>> {
        let mut where_clauses: Vec<String> = vec![];

        // 默认排除已删除的文件和 file_user 记录
        where_clauses.push(sql_format!("f.status!={}", FileStatus::Deleted as i8));
        where_clauses.push(sql_format!("fu.status!={}", FileUserStatus::Deleted as i8));

        // url 过滤
        let mut file_ids_from_url: Option<Vec<u64>> = None;
        if let Some(ref url) = filter.url {
            if !url.is_empty() {
                let prefix = &self.helper.config().local_file_url_prefix;
                if url.starts_with(prefix) {
                    let local_path = &url[prefix.len()..];
                    let rows = sqlx::query_as::<_, FileLocalModel>(&sql_format!(
                        "SELECT * FROM {} WHERE local_path={} LIMIT 100",
                        FileLocalModel::table_name(),
                        local_path
                    ))
                    .fetch_all(self.db())
                    .await?;

                    let ids: Vec<u64> = rows.iter().map(|r| r.file_id).collect();
                    if ids.is_empty() {
                        return Ok(None);
                    }
                    file_ids_from_url = Some(ids);
                }
            }
        }

        if let Some(ref ids) = file_ids_from_url {
            let id_str: Vec<String> = ids.iter().map(|i| i.to_string()).collect();
            where_clauses.push(format!("f.id IN ({})", id_str.join(",")));
        }

        // source_url 过滤
        if let Some(ref source_url) = filter.source_url {
            let trimmed = source_url.trim();
            if !trimmed.is_empty() {
                let source_md5 = FileHelper::compute_str_md5(trimmed);
                where_clauses.push(sql_format!("fu.source_md5={}", &source_md5));
            }
        }

        // user_id 过滤
        if let Some(uid) = filter.user_id {
            where_clauses.push(sql_format!("fu.user_id={}", uid));
        }

        // 时间范围
        if let Some(start) = filter.add_time_start {
            where_clauses.push(sql_format!("fu.add_time>={}", start));
        }
        if let Some(end) = filter.add_time_end {
            where_clauses.push(sql_format!("fu.add_time<={}", end));
        }

        // 状态
        if let Some(s) = filter.status {
            where_clauses.push(sql_format!("f.status={}", s));
        }

        // storage_type
        if let Some(ref st) = filter.storage_type {
            where_clauses.push(sql_format!("f.storage_type={}", st));
        }

        // file_md5
        if let Some(ref md5) = filter.file_md5 {
            where_clauses.push(sql_format!("f.file_md5={}", md5));
        }

        // app_id
        if let Some(aid) = filter.app_id {
            where_clauses.push(sql_format!("fu.app_id={}", aid));
        }

        Ok(Some(where_clauses))
    }

    /// 文件列表查询
    /// 
    /// 此方法基于 FileListAttrParam 参数决定是否查询关联表的详细信息。
    /// 返回数据已将关联表信息摊平到 FileListItemAttr 中。
    /// 
    /// # 参数
    /// - `filter`: 文件列表过滤条件
    /// - `page`: 分页参数
    /// - `attr_param`: attr 参数，指定是否需要查询 local 或 oss 的详细信息
    /// 
    /// # 返回
    /// - `Ok((Vec<FileListItemAttr>, CursorPageData<u64>))`: 查询成功，返回文件列表和分页信息
    /// - `Err(FileError)`: 查询失败
    /// 
    /// # 性能说明
    /// - 如果 attr_local 和 attr_oss 都为 false/None，则不会额外查询 file_local 和 file_oss 表
    /// - 如果需要 attr，会进行批量查询，对于 n 条记录最多进行 2 次额外的 SQL 查询
    /// 
    /// # 使用示例
    /// ```ignore
    /// // 不查询 attr 信息（仅基础数据）
    /// let (items, page) = file_dao.list_files(
    ///     &filter, 
    ///     &page, 
    ///     &FileListAttrParam::default()
    /// ).await?;
    /// 
    /// // 查询 local 和 oss 的详细信息
    /// let (items, page) = file_dao.list_files(
    ///     &filter, 
    ///     &page, 
    ///     &FileListAttrParam {
    ///         attr_local: Some(true),
    ///         attr_oss: Some(true),
    ///     }
    /// ).await?;
    /// 
    /// // 在返回时可以摊平数据
    /// for item in items {
    ///     if let Some(local_attr) = item.attr_local {
    ///         println!("Local path: {}", local_attr.local_path);
    ///     }
    ///     if let Some(oss_attr) = item.attr_oss {
    ///         println!("OSS URL: {}", oss_attr.object_url);
    ///     }
    /// }
    /// ```
    pub async fn list_files(
        &self,
        filter: &FileListFilter,
        page: &CursorPageParam<u64>,
        attr_param: &FileListAttrParam,
    ) -> FileResult<(Vec<FileListItemAttr>, CursorPageData<u64>)> {
        let where_clauses = match self.build_file_list_where(filter).await? {
            Some(clauses) => clauses,
            None => return Ok((vec![], CursorPageData::default())),
        };

        let query_limit = page.page_query("fu.id");
        let where_str = where_clauses.join(" AND ");
        let suff_sql = query_limit.build_query_sql(if where_clauses.is_empty() {
            None
        } else {
            Some(&where_str)
        });

        let sql = format!(
            "SELECT f.id, f.storage_type, f.status, f.file_name, f.file_md5, f.file_size, \
             f.modify_time, f.content_type, f.copy_file_id, f.from_user_id, f.add_time, f.change_time, \
             fu.id AS file_user_id, fu.user_id, fu.app_id, fu.file_id, fu.status AS file_user_status, \
             fu.source_url, fu.source_md5, fu.add_time AS file_user_add_time, fu.delete_time \
             FROM {} f INNER JOIN {} fu ON f.id=fu.file_id {}",
            FileModel::table_name().sql_quote(),
            FileUserModel::table_name().sql_quote(),
            suff_sql
        );

        let mut data = sqlx::query_as::<_, FileListItem>(&sql)
            .fetch_all(self.db())
            .await?;

        let next = query_limit.finalize(&mut data, |d, c| d.file_user_id == *c, |d| d.file_user_id);

        // 如果需要查询 attr，进行额外的数据库查询
        let mut result = Vec::with_capacity(data.len());
        
        // 收集需要查询的 file_id
        let need_attr_local = attr_param.attr_local.unwrap_or(false);
        let need_attr_oss = attr_param.attr_oss.unwrap_or(false);

        let mut local_file_ids: Vec<u64> = Vec::new();
        let mut oss_file_ids: Vec<u64> = Vec::new();

        if need_attr_local || need_attr_oss {
            for item in &data {
                if need_attr_local && item.storage_type == FileModel::STORAGE_TYPE_LOCAL {
                    local_file_ids.push(item.id);
                }
                if need_attr_oss && item.storage_type != FileModel::STORAGE_TYPE_LOCAL {
                    oss_file_ids.push(item.id);
                }
            }
        }

        // 批量查询 file_local 记录
        let mut local_map: std::collections::HashMap<u64, FileLocalAttr> = std::collections::HashMap::new();
        if !local_file_ids.is_empty() {
            let id_str: Vec<String> = local_file_ids.iter().map(|i| i.to_string()).collect();
            let sql = format!(
                "SELECT * FROM {} WHERE file_id IN ({})",
                FileLocalModel::table_name().sql_quote(),
                id_str.join(",")
            );
            let locals: Vec<FileLocalModel> =
                sqlx::query_as::<_, FileLocalModel>(&sql)
                    .fetch_all(self.db())
                    .await?;
            for local in locals {
                local_map.insert(local.file_id, FileLocalAttr {
                    id: local.id,
                    source_type: local.source_type,
                    source_name: local.source_name,
                    oss_file_id: local.oss_file_id,
                    local_path: local.local_path,
                    file_chunk_total: local.file_chunk_total,
                    file_chunk_succ: local.file_chunk_succ,
                    file_chunk_size: local.file_chunk_size,
                    last_error: local.last_error,
                });
            }
        }

        // 批量查询 file_oss 记录
        let mut oss_map: std::collections::HashMap<u64, FileOssAttr> = std::collections::HashMap::new();
        if !oss_file_ids.is_empty() {
            let id_str: Vec<String> = oss_file_ids.iter().map(|i| i.to_string()).collect();
            let sql = format!(
                "SELECT * FROM {} WHERE file_id IN ({})",
                FileOssModel::table_name().sql_quote(),
                id_str.join(",")
            );
            let osses: Vec<FileOssModel> =
                sqlx::query_as::<_, FileOssModel>(&sql)
                    .fetch_all(self.db())
                    .await?;
            for oss in osses {
                oss_map.insert(oss.file_id, FileOssAttr {
                    id: oss.id,
                    object_key: oss.object_key,
                    local_file_id: oss.local_file_id,
                    object_url: oss.object_url,
                    bucket: oss.bucket,
                    region: oss.region,
                    size: oss.size,
                    last_error: oss.last_error,
                });
            }
        }

        // 组合返回结果
        for item in data {
            let attr_local = if need_attr_local {
                local_map.remove(&item.id)
            } else {
                None
            };
            let attr_oss = if need_attr_oss {
                oss_map.remove(&item.id)
            } else {
                None
            };

            result.push(FileListItemAttr {
                item,
                attr_local,
                attr_oss,
            });
        }

        Ok((result, next))
    }


    /// 文件总数统计
    pub async fn count_files(&self, filter: &FileListFilter) -> FileResult<i64> {
        let where_clauses = match self.build_file_list_where(filter).await? {
            Some(clauses) => clauses,
            None => return Ok(0),
        };

        let where_str = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        let sql = format!(
            "SELECT COUNT(*) FROM {} f INNER JOIN {} fu ON f.id=fu.file_id {}",
            FileModel::table_name().sql_quote(),
            FileUserModel::table_name().sql_quote(),
            where_str
        );

        let count = sqlx::query_scalar::<_, i64>(&sql)
            .fetch_one(self.db())
            .await?;

        Ok(count)
    }

    // ==================== 操作方法 8: 删除文件 ====================
    pub async fn delete_file(
        &self,
        user_id: u64,
        app_id: u64,
        file_id: u64,
        oss_provider: Option<&dyn OssProvider>,
        env_data: Option<&RequestEnv>,
    ) -> FileResult<()> {
        use tracing::info;

        info!(
            "delete_file: starting, user_id={}, app_id={}, file_id={}",
            user_id, app_id, file_id
        );
        let now = now_time()?;

        // 查询该 file_id 的正常状态 file_user 数量
        let normal_count = sqlx::query_scalar::<_, i64>(&sql_format!(
            "SELECT COUNT(*) FROM {} WHERE file_id={} AND status={}",
            FileUserModel::table_name(),
            file_id,
            FileUserStatus::Normal as i8
        ))
        .fetch_one(self.db())
        .await?;

        info!(
            "delete_file: file_id={}, normal_count={}",
            file_id, normal_count
        );

        let where_sql = sql_format!(
            "user_id={} AND app_id={} AND file_id={} AND status={}",
            user_id,
            app_id,
            file_id,
            FileUserStatus::Normal as i8
        );

        // 软删除 file_user
        let res = Update::<FileUserModel>::new()
            .set(FileUserModel::STATUS, FileUserStatus::Deleted as i8)
            .set(FileUserModel::DELETE_TIME, now)
            .execute(SqlSuffix::Where(&where_sql), self.db())
            .await?;

        if res.rows_affected() == 0 {
            return Ok(());
        }

        self.log_dao()
            .add(file_id, 0, user_id, "delete_file: file_user deleted", None)
            .await;

        // 如果还有其他引用，不删除 file
        if normal_count > 1 {
            info!("delete_file: other refs exist, skipping file deletion");
        } else {
            // 软删除 file
            let file_res = Update::<FileModel>::new()
                .set(FileModel::STATUS, FileStatus::Deleted as i8)
                .set(FileModel::CHANGE_TIME, now)
                .execute(
                    SqlSuffix::Where(&sql_format!(
                        "id={} AND status={}",
                        file_id,
                        FileStatus::Normal as i8
                    )),
                    self.db(),
                )
                .await?;

            if file_res.rows_affected() > 0 {
                self.log_dao()
                    .add(file_id, 0, user_id, "delete_file: file deleted", None)
                    .await;
                // 物理文件删除判断
                self.try_cleanup_physical_file(file_id, oss_provider).await;
            }
        }

        self.logger
            .add(
                &LogFileDelete { user_id, file_id },
                Some(file_id),
                Some(user_id),
                None,
                env_data,
            )
            .await;

        Ok(())
    }

    /// 尝试清理物理文件
    async fn try_cleanup_physical_file(
        &self,
        file_id: u64,
        oss_provider: Option<&dyn OssProvider>,
    ) {
        use tracing::info;

        info!("try_cleanup_physical_file: checking file_id={}", file_id);

        // 获取文件信息
        let file = match self.helper.find_file_by_id(file_id).await {
            Ok(Some(f)) => f,
            Ok(None) => {
                warn!("try_cleanup_physical_file: file_id={} not found", file_id);
                return;
            }
            Err(e) => {
                warn!(
                    "try_cleanup_physical_file: query file_id={} error: {}",
                    file_id, e
                );
                return;
            }
        };

        if file.storage_type == FileModel::STORAGE_TYPE_LOCAL {
            let local = match self.helper.find_file_local_by_file_id(file_id).await {
                Ok(Some(l)) => l,
                Ok(None) => {
                    warn!(
                        "try_cleanup_physical_file: file_local not found for file_id={}",
                        file_id
                    );
                    return;
                }
                Err(e) => {
                    warn!("try_cleanup_physical_file: query file_local error: {}", e);
                    return;
                }
            };

            if local.local_path.is_empty() {
                info!(
                    "try_cleanup_physical_file: local_path is empty for file_id={}",
                    file_id
                );
                return;
            }

            // 检查是否有其他 local_path 引用
            let local_path_refs = sqlx::query_scalar::<_, i64>(&sql_format!(
                "SELECT COUNT(*) FROM {} fl INNER JOIN {} f ON fl.file_id=f.id \
                 WHERE fl.local_path={} AND f.status={} AND f.id!={}",
                FileLocalModel::table_name(),
                FileModel::table_name(),
                &local.local_path,
                FileStatus::Normal as i8,
                file_id
            ))
            .fetch_one(self.db())
            .await
            .unwrap_or(0);

            // 检查是否有相同 file_md5 的其他文件引用（排除拷贝文件，拷贝文件拥有独立的物理文件）
            let md5_refs = if !file.file_md5.is_empty() {
                sqlx::query_scalar::<_, i64>(&sql_format!(
                    "SELECT COUNT(*) FROM {} WHERE file_md5={} AND storage_type={} AND status={} AND id!={} AND copy_file_id=0",
                    FileModel::table_name(),
                    &file.file_md5,
                    &file.storage_type, 
                    FileStatus::Normal as i8,
                    file_id
                ))
                .fetch_one(self.db())
                .await
                .unwrap_or(0)
            } else {
                0
            };

            info!(
                "try_cleanup_physical_file: file_id={}, local_path_refs={}, md5_refs={}",
                file_id, local_path_refs, md5_refs
            );

            if local_path_refs > 0 || md5_refs > 0 {
                self.log_dao()
                    .add(
                        file_id,
                        0,
                        0,
                        &format!(
                            "delete: skip physical delete, refs exist (local_path={}, md5={})",
                            local_path_refs, md5_refs
                        ),
                        None,
                    )
                    .await;
                return;
            }

            // 删除物理文件
            let full = self.helper.get_full_local_path(&local.local_path);
            info!("try_cleanup_physical_file: deleting file {:?}", full);

            if let Err(e) = tokio::fs::remove_file(&full).await {
                warn!("delete physical file failed: {}", e);
                self.log_dao()
                    .add(
                        file_id,
                        0,
                        0,
                        &format!("delete: physical delete failed: {}", e),
                        None,
                    )
                    .await;
            } else {
                info!("try_cleanup_physical_file: successfully deleted {:?}", full);
                self.log_dao()
                    .add(file_id, 0, 0, "delete: physical file deleted", None)
                    .await;
            }
        } else {
            // OSS 文件删除
            info!(
                "try_cleanup_physical_file: processing OSS file for file_id={}",
                file_id
            );

            if let Some(provider) = oss_provider {
                if let Ok(Some(oss)) = self.helper.find_file_oss_by_file_id(file_id).await {
                    // 检查是否有相同 file_md5 的其他 OSS 文件引用
                    let md5_refs = if !file.file_md5.is_empty() {
                        sqlx::query_scalar::<_, i64>(&sql_format!(
                            "SELECT COUNT(*) FROM {} WHERE file_md5={} AND storage_type={} AND status={} AND id!={}",
                            FileModel::table_name(),
                            &file.file_md5,
                            &file.storage_type,
                            FileStatus::Normal as i8,
                            file_id
                        ))
                        .fetch_one(self.db())
                        .await
                        .unwrap_or(0)
                    } else {
                        0
                    };

                    if md5_refs > 0 {
                        info!(
                            "try_cleanup_physical_file: skip OSS delete, md5_refs={} for file_id={}",
                            md5_refs, file_id
                        );
                        self.log_dao()
                            .add(
                                file_id,
                                0,
                                0,
                                &format!(
                                    "delete: skip OSS delete, md5 refs exist (count={})",
                                    md5_refs
                                ),
                                None,
                            )
                            .await;
                        return;
                    }

                    if let Err(e) = provider.delete_object(&oss).await {
                        warn!("delete OSS object failed: {}", e);
                        self.log_dao()
                            .add(
                                file_id,
                                0,
                                0,
                                &format!("delete: OSS delete failed: {}", e),
                                None,
                            )
                            .await;
                    } else {
                        info!(
                            "try_cleanup_physical_file: OSS object deleted for file_id={}",
                            file_id
                        );
                        self.log_dao()
                            .add(file_id, 0, 0, "delete: OSS object deleted", None)
                            .await;
                    }
                }
            } else {
                info!(
                    "try_cleanup_physical_file: no OSS provider for file_id={}",
                    file_id
                );
            }
        }
    }
}
