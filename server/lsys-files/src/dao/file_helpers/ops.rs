use std::path::{Path, PathBuf};

use chrono::Local;
use lsys_core::db::SqlQuote;
use tokio::fs;
use tracing::warn;

use super::super::{FileError, FileResult};
use super::FileHelper;
use crate::model::*;

impl FileHelper {
    /// 新增文件函数: 在存储基础路径建立年月子目录, 创建可写入文件句柄
    /// 返回 (相对路径, 完整路径)
    ///
    /// - `prefix`: 文件名前缀 (如 "{app_id}_{user_id}_{type}")
    /// - `ext`: 文件扩展名 (不含点号), 为空时使用 "dat"
    ///
    /// 生成文件名格式: {sub_dir}/{prefix}_{random}.{ext}
    /// 其中 sub_dir 为年月目录 (如 "202603"), 用于文件归档和 MV 移动后的分类
    /// 生成后检查是否已存在, 存在时重新生成
    pub async fn create_new_file(&self, prefix: &str, ext: &str) -> FileResult<(String, PathBuf)> {
        let now = Local::now();
        let sub_dir = now.format("%Y%m%d").to_string();
        let base = Path::new(&self.config.storage_base_path);
        let dir = base.join(&sub_dir);

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

    /// 从文件名中提取扩展名，拿不到则返回 "dat"
    pub fn extract_extension(file_name: &str) -> &str {
        Path::new(file_name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("dat")
    }

    /// 获取完整本地路径
    pub fn get_full_local_path(&self, relative_path: &str) -> PathBuf {
        Path::new(&self.config.storage_base_path).join(relative_path)
    }

    /// 移动文件到存储路径
    /// 返回相对于存储基础路径的相对路径
    pub async fn move_file_to_storage(
        &self,
        source_path: &str,
        prefix: &str,
        target_name: Option<&str>,
    ) -> FileResult<String> {
        let src_path = Path::new(source_path);
        let file_name = target_name.map(|n| n.to_string()).unwrap_or_else(|| {
            src_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string())
        });
        let ext = Self::extract_extension(&file_name);

        let (relative_path, full_path) = self.create_new_file(prefix, ext).await?;

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
    pub async fn copy_file_to_storage(
        &self,
        source_path: &str,
        prefix: &str,
        target_name: Option<&str>,
    ) -> FileResult<String> {
        let src_path = Path::new(source_path);
        let file_name = target_name.map(|n| n.to_string()).unwrap_or_else(|| {
            src_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string())
        });
        let ext = Self::extract_extension(&file_name);

        let (relative_path, full_path) = self.create_new_file(prefix, ext).await?;

        fs::copy(source_path, &full_path).await.map_err(|e| {
            warn!("copy_file_to_storage: copy failed: {}", e);
            FileError::Io(e)
        })?;

        Ok(relative_path)
    }

    /// 计算文件 MD5
    pub async fn compute_file_md5(&self, path: &PathBuf) -> FileResult<String> {
        let data = fs::read(path).await?;
        let digest = md5::compute(&data);
        Ok(format!("{:x}", digest))
    }

    /// 计算字符串 MD5
    pub fn compute_str_md5(input: &str) -> String {
        let digest = md5::compute(input.as_bytes());
        format!("{:x}", digest)
    }

    /// 合并分片文件
    pub async fn merge_chunk_files(
        &self,
        chunks: &[FileLocalChunkModel],
        target_path: &PathBuf,
    ) -> FileResult<()> {
        use tokio::io::AsyncWriteExt;

        let mut target_file = fs::File::create(target_path).await?;

        // 按 chunk_index 排序
        let mut sorted: Vec<&FileLocalChunkModel> = chunks.iter().collect();
        sorted.sort_by_key(|c| c.chunk_index);

        for chunk in sorted {
            let chunk_full_path = self.get_full_local_path(&chunk.chunk_path);
            let data = fs::read(&chunk_full_path).await?;
            target_file.write_all(&data).await?;
        }

        target_file.flush().await?;
        Ok(())
    }

    /// 清理已合并的chunk文件（异步执行，不阻塞调用者）
    pub fn cleanup_merged_chunks(&self, chunk_ids: Vec<u64>) {
        if chunk_ids.is_empty() {
            return;
        }

        let db = self.db.clone();
        let config = self.config.clone();

        // spawn 独立任务异步执行，不阻塞调用者
        tokio::spawn(async move {
            Self::cleanup_chunks_impl(&db, &config, &chunk_ids).await;
        });
    }

    /// 清理chunk文件的核心实现
    async fn cleanup_chunks_impl(
        db: &sqlx::Pool<sqlx::MySql>,
        config: &super::super::file_config::FileConfig,
        chunk_ids: &[u64],
    ) {
        use lsys_core::db::{SqlSuffix, TableMeta, Update};
        use lsys_core::sql_format;
        use lsys_core::utils::now_time;
        use tracing::{info, warn};

        for &chunk_id in chunk_ids {
            let chunk = match sqlx::query_as::<_, FileLocalChunkModel>(&sql_format!(
                "SELECT * FROM {} WHERE id={} LIMIT 1",
                FileLocalChunkModel::table_name(),
                chunk_id
            ))
            .fetch_optional(db)
            .await
            {
                Ok(Some(c)) => c,
                Ok(None) => {
                    warn!("cleanup: chunk id {} not found", chunk_id);
                    continue;
                }
                Err(e) => {
                    warn!("cleanup: query chunk {} error: {}", chunk_id, e);
                    continue;
                }
            };

            if chunk.chunk_path.is_empty() {
                continue;
            }

            if !config.cleanup_enabled {
                info!("cleanup disabled, skip chunk path: {}", chunk.chunk_path);
                continue;
            }

            let full_path = Path::new(&config.storage_base_path).join(&chunk.chunk_path);
            match fs::remove_file(&full_path).await {
                Ok(_) => info!("cleanup: deleted chunk file {}", chunk.chunk_path),
                Err(e) => warn!(
                    "cleanup: delete chunk file {} failed: {}",
                    chunk.chunk_path, e
                ),
            }

            let now = now_time().unwrap_or_default();
            if let Err(e) = Update::<_, FileLocalChunkModel>::new()
                .set(FileLocalChunkModel::STATUS, FileChunkStatus::Cleaned as i8)
                .set(FileLocalChunkModel::CHANGE_TIME, now)
                .execute(SqlSuffix::Where(&sql_format!("id={}", chunk_id)), db)
                .await
            {
                warn!("cleanup: update chunk {} status error: {}", chunk_id, e);
            }
        }
    }
}

/// 生成高质量的随机数字符串：结合当天秒数 + 微秒 + 随机数，确保完全不重复
/// 格式: ((当天秒数+1) * 1000000) + 微秒(0-999999) + 随机数(0-9)
/// 返回固定 10 位长度的字符串（前导零补齐），可用于同一秒内的高频调用
fn rand_simple() -> String {
    use chrono::Timelike;
    
    // 获取当前时间
    let now = chrono::Local::now();
    
    // 计算当前时间距离今天0点的秒数（0-86399），加1后为 1-86400
    let seconds_today = ((now.hour() * 3600 + now.minute() * 60 + now.second()) as u64) + 1;
    
    // 获取微秒部分（0-999999）
    let microseconds = now.timestamp_subsec_micros() as u64;
    
    // 生成 0-9 的随机数（1位）
    let random_digit = (rand::random::<u32>() % 10) as u64;
    
    // 合并：(秒数+1) * 1000000 + 微秒 + 随机数
    // 利用微秒（0-999999）来填充同一秒内的时间差异
    // 最小值：1 * 1000000 + 0 + 0 = 1000000
    // 最大值：86400 * 1000000 + 999999 + 9 = 86,400,999,999+9（11位）
    let combined = (seconds_today * 1_000_000) + microseconds + random_digit;
    
    // 返回最后10位，确保长度一致
    format!("{:0>10}", combined % 10_000_000_000)
}

/// 清理文件名中的危险字符
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
        .collect::<String>()
        .chars()
        .take(200)
        .collect()
}
