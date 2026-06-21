use std::path::PathBuf;

use chrono::Local;
use lsys_core::db::QueryBuilderExt;
use lsys_core::db::{TableMeta, Update};
use lsys_core::utils::now_time;
use tokio::fs;
use tracing::{info, warn};

use super::super::super::common::{rand_simple, sanitize_filename};
use super::super::{FileError, FileResult};
use super::FileHelper;
use crate::model::*;

impl FileHelper {
    /// 新增文件函数: 在存储基础路径建立年月子目录, 创建可写入文件句柄
    /// 返回 (相对路径, 完整路径)
    ///
    /// - `storage_type`: 存储类型 (如 STORAGE_TYPE_LOCAL_PUBLIC)
    /// - `prefix`: 文件名前缀 (如 "{app_id}_{user_id}_{type}")
    /// - `ext`: 文件扩展名 (不含点号), 为空时使用 "dat"
    ///
    /// 生成文件名格式: {sub_dir}/{prefix}_{random}.{ext}
    /// 其中 sub_dir 为年月目录 (如 "202603"), 用于文件归档和 MV 移动后的分类
    /// 生成后检查是否已存在, 存在时重新生成
    pub async fn create_new_file(
        &self,
        storage_type: &str,
        prefix: &str,
        ext: &str,
    ) -> FileResult<(String, PathBuf)> {
        let now = Local::now();
        let sub_dir = now.format("%Y%m%d").to_string();

        let base_path = self.config.get_base_path(storage_type).await.map_err(|e| {
            FileError::System(
                lsys_core::fluent_message!("file-storage-error", {"error": e.to_string()}),
            )
        })?;
        let dir = base_path.join(&sub_dir);

        // 确保目录存在
        fs::create_dir_all(&dir).await?;

        let extension = ext.trim_start_matches('.');
        let extension = if extension.is_empty() {
            "dat"
        } else {
            extension
        };
        let safe_prefix = sanitize_filename(prefix);

        loop {
            let rand_val: String = rand_simple();
            let new_name = if safe_prefix.is_empty() {
                format!("{}_{}.{}", sub_dir, rand_val, extension)
            } else {
                format!("{}_{}_{}.{}", safe_prefix, sub_dir, rand_val, extension)
            };

            let full_path = dir.join(&new_name);

            // 检查文件是否已存在, 存在时重新生成
            if full_path.exists() {
                continue;
            }

            let relative_path = format!("{}/{}", sub_dir, new_name);

            // 创建文件
            fs::File::create(&full_path).await?;

            return Ok((relative_path, full_path));
        }
    }

    /// 获取完整本地路径
    ///
    /// - `storage_type`: 存储类型 (如 STORAGE_TYPE_LOCAL_PUBLIC)
    /// - `relative_path`: 相对路径
    pub async fn get_full_local_path(
        &self,
        storage_type: &str,
        relative_path: &str,
    ) -> FileResult<PathBuf> {
        let base_path = self.config.get_base_path(storage_type).await.map_err(|e| {
            FileError::System(
                lsys_core::fluent_message!("file-storage-error", {"error": e.to_string()}),
            )
        })?;
        Ok(base_path.join(relative_path))
    }

    /// 移动文件到存储路径
    /// 返回相对于存储基础路径的相对路径
    ///
    /// - `storage_type`: 存储类型 (如 STORAGE_TYPE_LOCAL_PUBLIC)
    pub async fn move_file_to_storage(
        &self,
        storage_type: &str,
        source_path: &str,
        prefix: &str,
        target_name: Option<&str>,
    ) -> FileResult<String> {
        let file_name = target_name.unwrap_or(source_path);
        let ext = crate::common::extract_extension(Some(file_name));

        let (relative_path, full_path) = self.create_new_file(storage_type, prefix, ext).await?;

        // 移动文件
        if let Err(rename_err) = fs::rename(source_path, &full_path).await {
            // rename 跨文件系统时可能失败, 尝试 copy + remove
            fs::copy(source_path, &full_path)
                .await
                .map_err(|copy_err| {
                    warn!(
                        "move file by copy failed: {}, original rename error: {}",
                        copy_err, rename_err
                    );
                    FileError::Io(copy_err)
                })?;
            if let Err(e) = fs::remove_file(source_path).await {
                warn!(
                    "move_file_to_storage: remove source after copy failed: {}",
                    e
                );
            }
        }

        Ok(relative_path)
    }

    /// 拷贝文件到存储路径（保留源文件）
    /// 返回相对于存储基础路径的相对路径
    ///
    /// - `storage_type`: 存储类型 (如 STORAGE_TYPE_LOCAL_PUBLIC)
    pub async fn copy_file_to_storage(
        &self,
        storage_type: &str,
        source_path: &str,
        prefix: &str,
        target_name: Option<&str>,
    ) -> FileResult<String> {
        let file_name = target_name.unwrap_or(source_path);
        let ext = crate::common::extract_extension(Some(file_name));

        let (relative_path, full_path) = self.create_new_file(storage_type, prefix, ext).await?;

        fs::copy(source_path, &full_path).await.map_err(|e| {
            warn!("copy_file_to_storage: copy failed: {}", e);
            FileError::Io(e)
        })?;

        Ok(relative_path)
    }

    /// 计算文件 MD5
    pub async fn compute_file_md5(&self, path: &PathBuf) -> FileResult<String> {
        use tokio::io::AsyncReadExt;

        // 128 KB 堆缓冲：对齐 Linux readahead，减少 syscall 次数
        // 直接读入自管缓冲区，避免 BufReader 的内部缓冲造成双重拷贝
        const BUF_SIZE: usize = 128 * 1024;
        let mut file = fs::File::open(path).await?;
        let mut hasher = md5::Context::new();
        let mut buffer = vec![0u8; BUF_SIZE];

        loop {
            let bytes_read = file.read(&mut buffer).await?;
            if bytes_read == 0 {
                break;
            }
            hasher.consume(&buffer[..bytes_read]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    /// 计算字符串 MD5
    pub fn compute_str_md5(input: &str) -> String {
        let digest = md5::compute(input.as_bytes());
        format!("{:x}", digest)
    }

    /// 合并分片文件
    ///
    /// - `storage_type`: 存储类型 (如 STORAGE_TYPE_LOCAL_PUBLIC)
    pub async fn merge_chunk_files(
        &self,
        storage_type: &str,
        chunks: &[FileLocalChunkModel],
        target_path: &PathBuf,
    ) -> FileResult<()> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};

        // 128 KB：对齐 Linux readahead 默认值，显著减少 syscall 次数
        // 读写两端使用相同容量，避免一侧成为瓶颈
        // 相比默认 8 KB，单次 I/O 吞吐提升 16 倍，适合分片合并的大块顺序读写场景
        const BUF_SIZE: usize = 128 * 1024;

        // 按 chunk_index 排序
        let mut sorted: Vec<&FileLocalChunkModel> = chunks.iter().collect();
        sorted.sort_by_key(|c| c.chunk_index);

        let target_file = fs::File::create(target_path).await?;
        let mut writer = BufWriter::with_capacity(BUF_SIZE, target_file);

        for chunk in &sorted {
            let chunk_full_path = self
                .get_full_local_path(storage_type, &chunk.chunk_path)
                .await?;
            let src = fs::File::open(&chunk_full_path).await?;
            // BufReader 将读端也提升为 128 KB，与写端对称
            // 使用 take(file_size) 限制只读取约定大小，防止实际文件超过约定大小时写入多余数据
            let mut reader = BufReader::with_capacity(BUF_SIZE, src).take(chunk.file_size);
            tokio::io::copy(&mut reader, &mut writer).await?;
        }

        writer.flush().await?;
        Ok(())
    }

    /// 清理已合并的chunk文件（异步执行，不阻塞调用者）
    pub fn cleanup_merged_chunks(
        &self,
        chunk_ids: Vec<u64>,
        log_dao: super::super::file_log::FileLogDao,
    ) {
        if chunk_ids.is_empty() {
            return;
        }

        let db = self.db.clone();
        let config = self.config.clone();

        // spawn 独立任务异步执行，不阻塞调用者
        tokio::spawn(async move {
            Self::cleanup_chunks_impl(&db, &config, &log_dao, &chunk_ids).await;
        });
    }

    /// 清理chunk文件的核心实现
    async fn cleanup_chunks_impl(
        db: &sqlx::Pool<sqlx::MySql>,
        config: &super::super::file_config::FileConfig,
        log_dao: &super::super::file_log::FileLogDao,
        chunk_ids: &[u64],
    ) {
        use tracing::Instrument;
        for &chunk_id in chunk_ids {
            Self::cleanup_single_chunk(db, config, log_dao, chunk_id)
                .instrument(tracing::info_span!(
                    "background_task",
                    task = "file-cleanup-chunk",
                    task_id = chunk_id
                ))
                .await;
        }
    }

    /// 清理单个 chunk 文件
    ///
    /// 从 `cleanup_chunks_impl` 的循环体提取，使每次清理拥有独立 span。
    /// 原循环体中的 `continue` 在此处变为 `return`。
    async fn cleanup_single_chunk(
        db: &sqlx::Pool<sqlx::MySql>,
        config: &super::super::file_config::FileConfig,
        log_dao: &super::super::file_log::FileLogDao,
        chunk_id: u64,
    ) {
        let chunk = match sqlx::query_as::<_, FileLocalChunkModel>(&format!(
            "SELECT * FROM {} WHERE id=? LIMIT 1",
            FileLocalChunkModel::table_name()
        ))
        .bind(chunk_id)
        .fetch_optional(db)
        .await
        {
            Ok(Some(c)) => c,
            Ok(None) => {
                warn!("cleanup: chunk id {} not found", chunk_id);
                return;
            }
            Err(e) => {
                warn!("cleanup: query chunk {} error: {}", chunk_id, e);
                return;
            }
        };

        if chunk.chunk_path.is_empty() {
            return;
        }

        if !config.cleanup_enabled {
            info!("cleanup disabled, skip chunk path: {}", chunk.chunk_path);
            log_dao
                .add(
                    chunk.file_id,
                    chunk.id,
                    0,
                    &format!("chunk cleanup skipped (disabled): {}", chunk.chunk_path),
                    None,
                )
                .await;
            return;
        }

        // 从 chunk 关联的 file_id 获取存储类型
        let storage_type = match sqlx::query_scalar::<_, String>(&format!(
            "SELECT storage_type FROM {} WHERE id=? LIMIT 1",
            FileModel::table_name()
        ))
        .bind(chunk.file_id)
        .fetch_optional(db)
        .await
        {
            Ok(Some(st)) => st,
            Ok(None) => {
                warn!(
                    "cleanup: file id {} not found for chunk {}",
                    chunk.file_id, chunk_id
                );
                return;
            }
            Err(e) => {
                warn!("cleanup: query file {} error: {}", chunk.file_id, e);
                return;
            }
        };

        let base_path = match config.get_base_path(&storage_type).await {
            Ok(p) => p,
            Err(e) => {
                warn!(
                    "cleanup: get base_path error for storage_type {}: {}",
                    storage_type, e
                );
                return;
            }
        };

        let full_path = base_path.join(&chunk.chunk_path);
        match fs::remove_file(&full_path).await {
            Ok(_) => {
                info!("cleanup: deleted chunk file {}", chunk.chunk_path);
                log_dao
                    .add(
                        chunk.file_id,
                        chunk.id,
                        0,
                        &format!("chunk file deleted: {}", chunk.chunk_path),
                        None,
                    )
                    .await;
            }
            Err(e) => warn!(
                "cleanup: delete chunk file {} failed: {}",
                chunk.chunk_path, e
            ),
        }

        let now = now_time().unwrap_or_default();
        if let Err(e) = Update::<_, FileLocalChunkModel>::new()
            .set(FileLocalChunkModel::STATUS, FileChunkStatus::Cleaned as i8)
            .set(FileLocalChunkModel::CHANGE_TIME, now)
            .execute(db, |qb| {
                qb.push_where().field_eq("id", chunk_id);
            })
            .await
        {
            warn!("cleanup: update chunk {} status error: {}", chunk_id, e);
        }
    }

    /// 获取下载超时时间（秒），确保不超过任务超时时间
    ///
    /// 返回值保证：download_timeout < task_timeout
    /// 如果配置的 download_timeout_secs >= task_timeout，则返回 task_timeout - 1
    pub async fn get_safe_download_timeout(&self) -> FileResult<u64> {
        let download_timeout = self.runtime_setting.get_download_timeout_secs().await?;
        let task_timeout = self.config.download_config.task_timeout as u64;

        if download_timeout >= task_timeout {
            warn!(
                "download_timeout_secs ({}) >= task_timeout ({}), using task_timeout - 1",
                download_timeout, task_timeout
            );
            Ok(task_timeout.saturating_sub(1).max(1))
        } else {
            Ok(download_timeout)
        }
    }

    /// 检查文件状态是否已被外部修改为失败或删除
    /// 返回 `true` 表示应中止操作
    pub async fn is_file_aborted(&self, file_id: u64) -> bool {
        let result: Result<Option<i8>, _> = sqlx::query_scalar(&format!(
            "SELECT status FROM {} WHERE id=? LIMIT 1",
            FileModel::table_name(),
        ))
        .bind(file_id)
        .fetch_optional(&self.db)
        .await;
        matches!(result, Ok(Some(s)) if FileStatus::Failed.eq(s) || FileStatus::Deleted.eq(s))
    }

    /// 检查分片状态是否已被外部修改为失败或已合并
    /// 返回 `true` 表示应中止操作
    pub async fn is_chunk_aborted(&self, chunk_id: u64) -> bool {
        let result: Result<Option<i8>, _> = sqlx::query_scalar(&format!(
            "SELECT status FROM {} WHERE id=? LIMIT 1",
            FileLocalChunkModel::table_name(),
        ))
        .bind(chunk_id)
        .fetch_optional(&self.db)
        .await;
        matches!(result, Ok(Some(s)) if FileChunkStatus::Failed.eq(s) || FileChunkStatus::Merged.eq(s))
    }
}
