use std::path::PathBuf;

use lsys_core::db::{FieldValue, QueryBuilderExt, TableMeta, Update};
use lsys_core::utils::now_time;
use sqlx::{MySql, Pool, Row};
use tracing::warn;

use super::DownloadStatus;
use super::FileResult;
use super::file_helpers::FileHelper;
use super::file_log::FileLogDao;
use crate::common::extract_extension;
use crate::model::*;

/// 下载最大重试次数 (首次 + 重试)
const DOWNLOAD_MAX_RETRIES: u32 = 2;

/// 下载结果
#[derive(Debug, Clone)]
pub enum DownloadResult {
    /// 下载完成 (文件已完成)
    Completed,
    /// 分片下载完成，但文件还有其他分片未完成
    ChunkCompleted,
    /// 下载失败
    Failed(String),
}

/// 文件下载核心逻辑
pub struct FileDownloadCore;

impl FileDownloadCore {
    /// 执行单个下载任务
    ///
    /// 返回:
    /// - `Ok(DownloadResult::Completed)`: 文件下载完成
    /// - `Ok(DownloadResult::ChunkCompleted)`: 分片下载完成，但文件还有其他分片未完成
    /// - `Ok(DownloadResult::Failed(msg))`: 下载失败
    /// - `Err(FileResult)`: 系统错误
    pub async fn execute_download(
        helper: &FileHelper,
        file_ref_id: u64,
        chunk_index: u32,
    ) -> FileResult<DownloadResult> {
        tracing::info!(
            "execute_download: start, file_ref_id={}, chunk_index={}",
            file_ref_id, chunk_index
        );
        let db = &helper.db;
        let log_dao = FileLogDao::new(helper.db.clone());

        // 步骤1: 查询 file_ref 和 file
        let file_ref = match helper.find_file_ref_by_id(file_ref_id).await? {
            Some(fu) => fu,
            None => {
                log_dao
                    .add(0, 0, 0, "download: file_ref not found", None)
                    .await;
                return Ok(DownloadResult::Failed("file_ref not found".to_string()));
            }
        };

        let mut file = match helper.find_file_by_id(file_ref.file_id).await? {
            Some(f) => f,
            None => {
                log_dao
                    .add(
                        file_ref.file_id,
                        0,
                        file_ref.user_id,
                        "download: file not found",
                        None,
                    )
                    .await;
                return Ok(DownloadResult::Failed("file not found".to_string()));
            }
        };

        // source_url 必须是 HTTP 协议
        let source_url = file_ref.source_url.trim().to_string();
        if !source_url.starts_with("http://") && !source_url.starts_with("https://") {
            Self::set_file_error(db, &mut file).await;
            log_dao
                .add(
                    file.id,
                    0,
                    file_ref.user_id,
                    &format!("download: source_url is not HTTP: {}", source_url),
                    None,
                )
                .await;
            return Ok(DownloadResult::Failed(format!(
                "source_url is not HTTP: {}",
                source_url
            )));
        }

        // 文件存储类型必须为 local
        if !file.is_local() {
            Self::set_file_error(db, &mut file).await;
            log_dao
                .add(
                    file.id,
                    0,
                    file_ref.user_id,
                    "download: storage_type is not local",
                    None,
                )
                .await;
            return Ok(DownloadResult::Failed(
                "storage_type is not local".to_string(),
            ));
        }

        // 步骤2: 查询 file_local
        let mut file_local = match helper.find_file_local_by_file_id(file.id).await? {
            Some(fl) => fl,
            None => {
                Self::set_file_error(db, &mut file).await;
                log_dao
                    .add(
                        file.id,
                        0,
                        file_ref.user_id,
                        "download: file_local not found",
                        None,
                    )
                    .await;
                return Ok(DownloadResult::Failed("file_local not found".to_string()));
            }
        };

        // 步骤3: 根据 file 状态判断
        if FileStatus::Normal.eq(file.status) {
            if !file_local.local_path.is_empty() {
                let full_path = helper
                    .get_full_local_path(&file.storage_type, &file_local.local_path)
                    .await
                    .unwrap_or_else(|_| PathBuf::from(&file_local.local_path));
                if tokio::fs::metadata(&full_path).await.is_ok() {
                    log_dao
                        .add(
                            file.id,
                            0,
                            file_ref.user_id,
                            "download: file already completed",
                            None,
                        )
                        .await;
                    return Ok(DownloadResult::Completed);
                }
            }
            // 完成但文件不存在, 设置为失败
            Self::set_file_and_local_error(
                db,
                &mut file,
                &mut file_local,
                "download: file completed but local file missing",
            )
            .await;
            log_dao
                .add(
                    file.id,
                    0,
                    file_ref.user_id,
                    "download: file completed but local file missing",
                    None,
                )
                .await;
            return Ok(DownloadResult::Failed(
                "file completed but local file missing".to_string(),
            ));
        }

        if !FileStatus::Unfinished.eq(file.status) || !file_local.local_path.is_empty() {
            log_dao
                .add(
                    file.id,
                    0,
                    file_ref.user_id,
                    &format!("download: unexpected file status={}", file.status),
                    None,
                )
                .await;
            return Ok(DownloadResult::Failed(format!(
                "unexpected file status={}",
                file.status
            )));
        }

        // 步骤4/5: 根据 file_chunk_total 决定下载方式
        log_dao
            .add(
                file.id,
                0,
                file_ref.user_id,
                &format!(
                    "download: checks passed, start downloading, source_url={}, chunk_total={}",
                    source_url, file_local.file_chunk_total
                ),
                None,
            )
            .await;

        let result = if file_local.file_chunk_total > 0 {
            Self::download_chunked(
                helper,
                &source_url,
                &mut file,
                &mut file_local,
                chunk_index,
                file_ref.user_id,
                file_ref.app_id,
            )
            .await
        } else {
            Self::download_single(
                helper,
                db,
                &source_url,
                &mut file,
                &mut file_local,
                file_ref.user_id,
                file_ref.app_id,
            )
            .await
        };

        // 下载到达终态（完成或失败）时统一清理进度数据
        match &result {
            Ok(DownloadResult::Completed) => {
                helper.progress_tracker.clear_progress(file.id, DownloadStatus::Completed);
            }
            Ok(DownloadResult::Failed(_)) => {
                helper.progress_tracker.clear_progress(file.id, DownloadStatus::Failed);
            }
            _ => {}
        }

        result
    }

    /// 分片下载 (步骤 4)
    #[allow(clippy::too_many_arguments)]
    async fn download_chunked(
        helper: &FileHelper,
        source_url: &str,
        file: &mut FileModel,
        file_local: &mut FileLocalModel,
        chunk_index: u32,
        user_id: u64,
        app_id: u64,
    ) -> FileResult<DownloadResult> {
        let db = &helper.db;
        let log_dao = FileLogDao::new(helper.db.clone());

        // 4.1 查询 file_local_chunk
        let mut chunk = match helper
            .find_chunk_by_file_and_index(file.id, chunk_index)
            .await?
        {
            Some(c) => c,
            None => {
                let err_msg = format!("download: chunk not found index={}", chunk_index);
                Self::set_file_and_local_error(db, file, file_local, &err_msg).await;
                log_dao.add(file.id, 0, user_id, &err_msg, None).await;
                return Ok(DownloadResult::Failed(err_msg));
            }
        };

        // 判断状态
        if FileChunkStatus::Normal.eq(chunk.status) {
            // 已完成, 检查文件存在
            if !chunk.chunk_path.is_empty() {
                let full = helper
                    .get_full_local_path(&file.storage_type, &chunk.chunk_path)
                    .await
                    .unwrap_or_else(|_| PathBuf::from(&chunk.chunk_path));
                if tokio::fs::metadata(&full).await.is_ok() {
                    log_dao
                        .add(
                            file.id,
                            chunk.id,
                            user_id,
                            "download: chunk already done",
                            None,
                        )
                        .await;
                    // 进入 4.4
                    return Self::on_chunk_complete(
                        helper, db, file, file_local, &mut chunk, user_id, app_id,
                    )
                    .await;
                }
                // 文件不存在, 进入 4.2 重新下载 (fallthrough)
            }
        } else if !FileChunkStatus::Unfinished.eq(chunk.status) {
            log_dao
                .add(
                    file.id,
                    chunk.id,
                    user_id,
                    &format!("download: unexpected chunk status={}", chunk.status),
                    None,
                )
                .await;
            return Ok(DownloadResult::Failed(format!(
                "unexpected chunk status={}",
                chunk.status
            )));
        }

        // 4.2 创建新文件并下载
        let ext = extract_extension(Some(&file.origin_name));
        let prefix = format!("{}_{}_dlchunk{}", app_id, user_id, chunk_index);
        let (rel_path, full_path) = helper
            .create_new_file(&file.storage_type, &prefix, ext)
            .await?;

        // 预先写入 chunk_path，供下载过程日志及失败路径使用
        chunk.chunk_path = rel_path.clone();
        tracing::debug!(
            "download: chunk file created, file_id={}, chunk_index={}, path={}",
            file.id,
            chunk_index,
            rel_path
        );

        // 下载 (带重试)
        let download_result = Self::download_range(
            helper,
            source_url,
            &full_path,
            chunk.start_offset,
            chunk.file_size,
            file,
            &mut chunk,
            &log_dao,
            user_id,
        )
        .await;

        if let Err(error_msg) = download_result {
            // 4.3 分片下载失败 — 删除临时分片文件
            if let Err(e) = tokio::fs::remove_file(&full_path).await {
                warn!(
                    "download: failed to remove partial chunk file, path={:?}: {}",
                    full_path, e
                );
            }
            let now = now_time().unwrap_or_default();
            if let Err(e) = Update::<_, FileLocalChunkModel>::new()
                .set(FileLocalChunkModel::STATUS, FileChunkStatus::Failed as i8)
                .set(FileLocalChunkModel::CHANGE_TIME, now)
                .execute(db, |qb| {
                    qb.push_where().field_eq("id", chunk.id);
                })
                .await
            {
                log_dao
                    .add(
                        file.id,
                        chunk.id,
                        user_id,
                        &format!(
                            "download: update chunk failed status error, chunk_id={}: {}",
                            chunk.id, e
                        ),
                        None,
                    )
                    .await;
            }
            let full_error_msg = format!("download failed: {}", error_msg);
            Self::set_file_and_local_error(db, file, file_local, &full_error_msg).await;
            log_dao
                .add(file.id, chunk.id, user_id, &full_error_msg, None)
                .await;
            return Ok(DownloadResult::Failed(full_error_msg));
        }

        // 下载成功，持久化 chunk_path
        if let Err(e) = Update::<sqlx::MySql, FileLocalChunkModel>::new()
            .set(FileLocalChunkModel::CHUNK_PATH, &rel_path)
            .execute(db, |qb| {
                qb.push_where().field_eq("id", chunk.id);
            })
            .await
        {
            log_dao
                .add(
                    file.id,
                    chunk.id,
                    user_id,
                    &format!(
                        "download: update chunk_path failed, chunk_id={}: {}",
                        chunk.id, e
                    ),
                    None,
                )
                .await;
        }

        // 4.4 分片下载完成
        Self::on_chunk_complete(helper, db, file, file_local, &mut chunk, user_id, app_id).await
    }

    /// 4.4 分片下载完成
    #[allow(clippy::too_many_arguments)]
    async fn on_chunk_complete(
        helper: &FileHelper,
        db: &Pool<MySql>,
        file: &mut FileModel,
        file_local: &mut FileLocalModel,
        chunk: &mut FileLocalChunkModel,
        user_id: u64,
        app_id: u64,
    ) -> FileResult<DownloadResult> {
        let log_dao = FileLogDao::new(helper.db.clone());
        let now = now_time().unwrap_or_default();

        // 计算 chunk md5 和 complete_size
        let full = helper
            .get_full_local_path(&file.storage_type, &chunk.chunk_path)
            .await
            .unwrap_or_else(|_| PathBuf::from(&chunk.chunk_path));
        tracing::debug!(
            "download: computing md5, file_id={}, chunk_id={}, path={}",
            file.id,
            chunk.id,
            chunk.chunk_path
        );
        let metadata = tokio::fs::metadata(&full).await?;
        let file_data = tokio::fs::read(&full).await?;
        let chunk_md5 = format!("{:x}", md5::compute(&file_data));
        let complete_size = metadata.len();

        chunk.chunk_md5 = chunk_md5.clone();
        chunk.complete_size = complete_size;
        chunk.status = FileChunkStatus::Normal as i8;
        chunk.change_time = now;

        // 使用事务更新 chunk、file_local、file，确保数据一致性
        let mut tx = db.begin().await?;

        // tx_result = true 表示当前分片是最后一个完成的（merge winner），否则 false
        let tx_result: FileResult<bool> = async {
            // 更新 chunk
            Update::<_, FileLocalChunkModel>::new()
                .set(FileLocalChunkModel::STATUS, FileChunkStatus::Normal as i8)
                .set(FileLocalChunkModel::CHANGE_TIME, now)
                .set(FileLocalChunkModel::CHUNK_MD5, &chunk_md5)
                .set(FileLocalChunkModel::COMPLETE_SIZE, complete_size)
                .execute(&mut *tx, |qb| {
                    qb.push_where().field_eq("id", chunk.id);
                })
                .await?;

            // 更新 file_local: file_chunk_succ+1, file_chunk_size+=complete_size (原子操作)
            Update::<_, FileLocalModel>::new()
                .set(
                    FileLocalModel::FILE_CHUNK_SUCC,
                    FieldValue::Expr("file_chunk_succ + 1".into()),
                )
                .set(
                    FileLocalModel::FILE_CHUNK_SIZE,
                    FieldValue::Expr(format!("file_chunk_size + {}", complete_size).into()),
                )
                .execute(&mut *tx, |qb| {
                    qb.push_where().field_eq("id", file_local.id);
                })
                .await?;

            // 注意：分片下载的 file.file_size 在 create_from_url 时已由调用方传入总大小写入，
            // 此处不再累加，避免重复计入

            // 原子判断：是否是最后一个完成的分片。
            // UPDATE 已持有该行的排他锁，FOR UPDATE SELECT 在同一事务中读取自身写入的最新值，
            // 并防止并发事务在 commit 前读取到相同的 succ==total 状态。
            let row = sqlx::query(&format!(
                "SELECT file_chunk_succ, file_chunk_total FROM {} WHERE id = ? FOR UPDATE",
                FileLocalModel::table_name()
            ))
            .bind(file_local.id)
            .fetch_one(&mut *tx)
            .await?;
            let succ: u32 = row.try_get("file_chunk_succ")?;
            let total: u32 = row.try_get("file_chunk_total")?;

            Ok(total > 0 && succ == total)
        }
        .await;

        let is_merge_winner = match tx_result {
            Ok(v) => {
                tx.commit().await?;
                v
            }
            Err(e) => {
                if let Err(rb_err) = tx.rollback().await {
                    warn!("on_chunk_complete: rollback failed: {}", rb_err);
                }
                log_dao
                    .add(
                        file.id,
                        chunk.id,
                        user_id,
                        &format!("download: update chunk complete info failed: {}", e),
                        None,
                    )
                    .await;
                return Err(e);
            }
        };

        log_dao
            .add(
                file.id,
                chunk.id,
                user_id,
                &format!(
                    "download: chunk download done, chunk_index={}, md5={}, size={}",
                    chunk.chunk_index, chunk_md5, complete_size
                ),
                None,
            )
            .await;

        // is_merge_winner=true 表示当前分片是最后一个完成的，可以触发合并；否则等待。
        if !is_merge_winner {
            log_dao
                .add(
                    file.id,
                    chunk.id,
                    user_id,
                    "download: chunk complete, waiting for others",
                    None,
                )
                .await;
            return Ok(DownloadResult::ChunkCompleted);
        }

        // 所有分片都完成（本分片是 merge winner），读取分片列表后合并
        let all_chunks = helper.find_chunks_by_file_id(file.id).await?;
        let ext = extract_extension(Some(&file.origin_name));
        let merge_prefix = format!("{}_{}_dlmerge", app_id, user_id);
        let (merge_rel, merge_full) = helper
            .create_new_file(&file.storage_type, &merge_prefix, ext)
            .await?;
        log_dao
            .add(
                file.id,
                0,
                user_id,
                &format!(
                    "download: all chunks done, merging, file_id={}, chunk_count={}, merge_path={}",
                    file.id,
                    all_chunks.len(),
                    merge_rel
                ),
                None,
            )
            .await;

        match helper
            .merge_chunk_files(&file.storage_type, &all_chunks, &merge_full)
            .await
        {
            Err(e) => {
                // 合并失败
                file_local.local_path = merge_rel.clone();
                file.status = FileStatus::Failed as i8;
                file.change_time = now;
                let err_msg = format!("download: merge chunks failed: {}", e);
                Self::set_file_error(db, file).await;
                if let Err(e) = Update::<sqlx::MySql, FileLocalModel>::new()
                    .set(FileLocalModel::LOCAL_PATH, &merge_rel)
                    .set(FileLocalModel::LAST_ERROR, err_msg.as_str())
                    .execute(db, |qb| {
                        qb.push_where().field_eq("id", file_local.id);
                    })
                    .await
                {
                    log_dao.add(file.id, 0, user_id, &format!("download: update file_local merge error info failed, file_local_id={}: {}", file_local.id, e), None).await;
                }
                log_dao.add(file.id, 0, user_id, &err_msg, None).await;
                Ok(DownloadResult::Failed(err_msg))
            }
            Ok(_) => {
                // 更新所有 chunk status=已合并
                let chunk_ids: Vec<u64> = all_chunks.iter().map(|c| c.id).collect();
                if let Err(e) = Update::<sqlx::MySql, FileLocalChunkModel>::new()
                    .set(FileLocalChunkModel::STATUS, FileChunkStatus::Merged as i8)
                    .set(FileLocalChunkModel::CHANGE_TIME, now)
                    .execute(db, |qb| {
                        qb.push_where().field_eq("file_id", file.id);
                    })
                    .await
                {
                    log_dao
                        .add(
                            file.id,
                            0,
                            user_id,
                            &format!(
                                "download: update chunks merged status failed, file_id={}: {}",
                                file.id, e
                            ),
                            None,
                        )
                        .await;
                }

                // 清理已合并的chunk文件
                helper.cleanup_merged_chunks(chunk_ids, log_dao.clone());

                // 使用辅助函数完成文件
                let result = helper
                    .complete_file_and_local(file, file_local, &merge_rel)
                    .await;

                match result {
                    Ok(Some(_other)) => {
                        // 已有相同文件, 删除合并文件
                        if let Err(e) = tokio::fs::remove_file(&merge_full).await {
                            warn!("download: remove merge file failed: {}", e);
                        }
                        log_dao
                            .add(
                                file.id,
                                0,
                                user_id,
                                "download: merge done, duplicate found, cleaned",
                                None,
                            )
                            .await;
                        Ok(DownloadResult::Completed)
                    }
                    Ok(None) => {
                        log_dao
                            .add(
                                file.id,
                                0,
                                user_id,
                                "download: merge and complete done",
                                None,
                            )
                            .await;
                        Ok(DownloadResult::Completed)
                    }
                    Err(e) => {
                        warn!("download: complete_file_and_local error: {}", e);
                        log_dao
                            .add(
                                file.id,
                                0,
                                user_id,
                                &format!("download: complete error: {}", e),
                                None,
                            )
                            .await;
                        Ok(DownloadResult::Failed(format!("complete error: {}", e)))
                    }
                }
            }
        }
    }

    /// 步骤5: 非分片下载
    #[allow(clippy::too_many_arguments)]
    async fn download_single(
        helper: &FileHelper,
        db: &Pool<MySql>,
        source_url: &str,
        file: &mut FileModel,
        file_local: &mut FileLocalModel,
        user_id: u64,
        app_id: u64,
    ) -> FileResult<DownloadResult> {
        let log_dao = FileLogDao::new(helper.db.clone());

        // 创建新文件
        let ext = extract_extension(Some(&file.origin_name));
        let prefix = format!("{}_{}_dl", app_id, user_id);
        let (rel_path, full_path) = helper
            .create_new_file(&file.storage_type, &prefix, ext)
            .await?;
        tracing::debug!(
            "download: single file created, file_id={}, path={}",
            file.id,
            rel_path
        );

        // 下载
        let download_result = Self::download_full(
            helper, db, source_url, &full_path, &rel_path, file, &log_dao, user_id,
        )
        .await;

        if let Err(error_msg) = download_result {
            // 步骤6: 下载失败 — 删除临时文件，不将失败路径写入 DB
            if let Err(e) = tokio::fs::remove_file(&full_path).await {
                warn!(
                    "download: failed to remove partial file, path={:?}: {}",
                    full_path, e
                );
            }
            let full_error_msg = format!("download failed: {}", error_msg);
            Self::set_file_and_local_error(db, file, file_local, &full_error_msg).await;
            log_dao
                .add(file.id, 0, user_id, &full_error_msg, None)
                .await;
            return Ok(DownloadResult::Failed(full_error_msg));
        }

        // 步骤7: 完成下载
        let result = helper
            .complete_file_and_local(file, file_local, &rel_path)
            .await;
        match result {
            Ok(Some(_other)) => {
                // 已有相同文件, 删除下载文件
                if let Err(e) = tokio::fs::remove_file(&full_path).await {
                    warn!("download: remove duplicate file failed: {}", e);
                }
                log_dao
                    .add(
                        file.id,
                        0,
                        user_id,
                        "download: complete, duplicate found, cleaned",
                        None,
                    )
                    .await;
                Ok(DownloadResult::Completed)
            }
            Ok(None) => {
                log_dao
                    .add(file.id, 0, user_id, "download: complete", None)
                    .await;
                Ok(DownloadResult::Completed)
            }
            Err(e) => {
                warn!("download: complete_file_and_local error: {}", e);
                log_dao
                    .add(
                        file.id,
                        0,
                        user_id,
                        &format!("download: complete error: {}", e),
                        None,
                    )
                    .await;
                Ok(DownloadResult::Failed(format!("complete error: {}", e)))
            }
        }
    }

    /// 范围下载 (用于分片), 带重试
    #[allow(clippy::too_many_arguments)]
    async fn download_range(
        helper: &FileHelper,
        url: &str,
        target: &std::path::PathBuf,
        start_offset: u64,
        expected_size: u64,
        file: &mut FileModel,
        chunk: &mut FileLocalChunkModel,
        log_dao: &FileLogDao,
        user_id: u64,
    ) -> Result<(), String> {
        let mut last_error = String::new();

        for attempt in 0..DOWNLOAD_MAX_RETRIES {
            if attempt > 0 {
                log_dao
                    .add(
                        file.id,
                        chunk.id,
                        user_id,
                        &format!(
                            "download: range retry attempt={}/{}, chunk_index={}, path={}",
                            attempt + 1,
                            DOWNLOAD_MAX_RETRIES,
                            chunk.chunk_index,
                            chunk.chunk_path
                        ),
                        None,
                    )
                    .await;
                // 清空文件准备重试
                if let Err(e) = tokio::fs::write(target, b"").await {
                    warn!("download_range: failed to clear file for retry: {}", e);
                }
            }

            match Self::do_range_download(
                helper,
                url,
                target,
                start_offset,
                expected_size,
                file,
                chunk,
                log_dao,
                user_id,
            )
            .await
            {
                Ok(true) => {
                    log_dao.add(file.id, chunk.id, user_id, &format!("download: range download data received successfully, chunk_index={}", chunk.chunk_index), None).await;
                    return Ok(());
                }
                Ok(false) => {
                    last_error = format!(
                        "chunk {} download failed at attempt {}",
                        chunk.chunk_index,
                        attempt + 1
                    );
                    continue;
                }
                Err(e) => {
                    last_error =
                        format!("chunk {} attempt {}: {}", chunk.chunk_index, attempt + 1, e);
                    log_dao
                        .add(
                            file.id,
                            chunk.id,
                            user_id,
                            &format!("download: range attempt {} error: {}", attempt + 1, e),
                            None,
                        )
                        .await;
                    continue;
                }
            }
        }
        Err(last_error)
    }

    #[allow(clippy::too_many_arguments)]
    async fn do_range_download(
        helper: &FileHelper,
        url: &str,
        target: &std::path::PathBuf,
        start_offset: u64,
        expected_size: u64,
        file: &mut FileModel,
        chunk: &mut FileLocalChunkModel,
        log_dao: &FileLogDao,
        user_id: u64,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        use futures_util::StreamExt;
        use tokio::io::AsyncWriteExt;

        let timeout_secs = helper.get_safe_download_timeout().await.unwrap_or(300);
        let client = reqwest::Client::new();
        let end_offset = start_offset + expected_size - 1;
        let range_header = format!("bytes={}-{}", start_offset, end_offset);

        tracing::debug!(
            "download: sending range request, file_id={}, chunk_id={}, range={}, path={}",
            file.id,
            chunk.id,
            range_header,
            chunk.chunk_path
        );
        let resp = client
            .get(url)
            .header("Range", &range_header)
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .send()
            .await?;

        let http_status = resp.status().as_u16();
        if http_status == 200 && start_offset > 0 {
            // 服务器不支持 Range，返回了完整内容，但当前分片不是从头开始，数据不可用
            log_dao.add(file.id, chunk.id, user_id, &format!(
                "download: server returned 200 (no range support) for non-first chunk, chunk_index={}, start_offset={}, path={}",
                chunk.chunk_index, start_offset, chunk.chunk_path
            ), None).await;
            return Ok(false);
        }
        if http_status != 206 && http_status != 200 {
            log_dao
                .add(
                    file.id,
                    chunk.id,
                    user_id,
                    &format!(
                        "download: unexpected range response status={}, chunk_index={}, path={}",
                        http_status, chunk.chunk_index, chunk.chunk_path
                    ),
                    None,
                )
                .await;
            return Ok(false);
        }
        tracing::debug!(
            "download: range response status={}, file_id={}, chunk_id={}, expected_size={}",
            http_status,
            file.id,
            chunk.id,
            expected_size
        );

        let mut file_handle = tokio::fs::File::create(target).await?;
        let mut stream = resp.bytes_stream();
        let mut downloaded: u64 = 0;
        let mut last_status_check = std::time::Instant::now();
        const STATUS_CHECK_INTERVAL: std::time::Duration = std::time::Duration::from_secs(1);

        while let Some(bytes_result) = stream.next().await {
            let bytes = bytes_result?;
            file_handle.write_all(&bytes).await?;
            downloaded += bytes.len() as u64;

            tracing::trace!(
                "download: file_id={}, chunk_id={}, path={}, downloaded={}",
                chunk.file_id,
                chunk.id,
                chunk.chunk_path,
                downloaded
            );

            // 每秒检查一次文件/分片状态是否已被外部修改
            if last_status_check.elapsed() >= STATUS_CHECK_INTERVAL {
                last_status_check = std::time::Instant::now();

                // 更新进度并推送到 Redis
                helper.progress_tracker.record_bytes(
                    file.id,
                    chunk.id,
                    downloaded,
                    expected_size,  // 本分片大小
                    file.file_size, // 文件总大小（分片初始化时已写入 DB）
                );

                if helper.is_file_aborted(file.id).await {
                    log_dao.add(file.id, chunk.id, user_id, &format!(
                        "download: aborted, file status changed, chunk_index={}, downloaded={}",
                        chunk.chunk_index, downloaded
                    ), None).await;
                    return Ok(false);
                }

                if helper.is_chunk_aborted(chunk.id).await {
                    log_dao.add(file.id, chunk.id, user_id, &format!(
                        "download: aborted, chunk status changed, chunk_index={}, downloaded={}",
                        chunk.chunk_index, downloaded
                    ), None).await;
                    return Ok(false);
                }
            }

            if downloaded >= expected_size {
                if downloaded > expected_size {
                    let excess = downloaded - expected_size;
                    log_dao
                        .add(
                            file.id,
                            chunk.id,
                            user_id,
                            &format!(
                                "download: range received more data than expected, \
                         chunk_id={}, downloaded={}, expected={}, excess={}",
                                chunk.id, downloaded, expected_size, excess
                            ),
                            None,
                        )
                        .await;
                }
                break;
            }
        }

        file_handle.flush().await?;
        Ok(true)
    }

    /// 完整下载 (非分片), 带重试
    #[allow(clippy::too_many_arguments)]
    async fn download_full(
        helper: &FileHelper,
        db: &Pool<MySql>,
        url: &str,
        target: &std::path::PathBuf,
        rel_path: &str,
        file: &mut FileModel,
        log_dao: &FileLogDao,
        user_id: u64,
    ) -> Result<(), String> {
        let mut last_error = String::new();

        for attempt in 0..DOWNLOAD_MAX_RETRIES {
            if attempt > 0 {
                log_dao
                    .add(
                        file.id,
                        0,
                        user_id,
                        &format!(
                            "download: full retry attempt={}/{}, path={}",
                            attempt + 1,
                            DOWNLOAD_MAX_RETRIES,
                            rel_path
                        ),
                        None,
                    )
                    .await;
                if let Err(e) = tokio::fs::write(target, b"").await {
                    warn!("download_full: failed to clear file for retry: {}", e);
                }
            }

            match Self::do_full_download(helper, db, url, target, rel_path, file, log_dao, user_id)
                .await
            {
                Ok(true) => {
                    log_dao
                        .add(
                            file.id,
                            0,
                            user_id,
                            "download: full download data received successfully",
                            None,
                        )
                        .await;
                    return Ok(());
                }
                Ok(false) => {
                    last_error = format!("download failed at attempt {}", attempt + 1);
                    continue;
                }
                Err(e) => {
                    last_error = format!("attempt {}: {}", attempt + 1, e);
                    log_dao
                        .add(
                            file.id,
                            0,
                            user_id,
                            &format!("download: full attempt {} error: {}", attempt + 1, e),
                            None,
                        )
                        .await;
                    continue;
                }
            }
        }
        Err(last_error)
    }

    #[allow(clippy::too_many_arguments)]
    async fn do_full_download(
        helper: &FileHelper,
        db: &Pool<MySql>,
        url: &str,
        target: &std::path::PathBuf,
        rel_path: &str,
        file: &mut FileModel,
        log_dao: &FileLogDao,
        user_id: u64,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        use futures_util::StreamExt;
        use tokio::io::AsyncWriteExt;

        let timeout_secs = helper.get_safe_download_timeout().await.unwrap_or(300);
        let client = reqwest::Client::new();

        tracing::info!(
            "download: sending full request, file_id={}, url={}, timeout={}s",
            file.id, url, timeout_secs
        );
        let resp = client
            .get(url)
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .send()
            .await?;

        let http_status = resp.status().as_u16();
        tracing::info!(
            "download: got HTTP response, file_id={}, status={}",
            file.id, http_status
        );
        if http_status != 200 {
            log_dao
                .add(
                    file.id,
                    0,
                    user_id,
                    &format!(
                        "download: unexpected full response status={}, path={}",
                        http_status, rel_path
                    ),
                    None,
                )
                .await;
            return Ok(false);
        }

        // 从 HTTP 头获取文件总大小，并写入 file.file_size
        let content_length = resp
            .headers()
            .get(reqwest::header::CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);
        if content_length > 0 && file.file_size == 0 {
            file.file_size = content_length;
            if let Err(e) = Update::<_, FileModel>::new()
                .set(FileModel::FILE_SIZE, content_length)
                .execute(db, |qb| {
                    qb.push_where().field_eq("id", file.id);
                })
                .await
            {
                log_dao
                    .add(
                        file.id,
                        0,
                        user_id,
                        &format!(
                            "download: update file_size from Content-Length failed: {}",
                            e
                        ),
                        None,
                    )
                    .await;
            } else {
                tracing::debug!(
                    "download: got Content-Length={}, updated file.file_size, file_id={}",
                    content_length,
                    file.id
                );
            }
        }

        tracing::debug!(
            "download: full response ok(200), file_id={}, content_length={}, path={}",
            file.id,
            content_length,
            rel_path
        );

        let mut file_handle = tokio::fs::File::create(target).await?;
        let mut stream = resp.bytes_stream();
        let mut downloaded: u64 = 0;
        let mut last_status_check = std::time::Instant::now();
        const STATUS_CHECK_INTERVAL: std::time::Duration = std::time::Duration::from_secs(1);

        while let Some(bytes_result) = stream.next().await {
            let bytes = bytes_result?;
            file_handle.write_all(&bytes).await?;
            downloaded += bytes.len() as u64;
            tracing::trace!(
                "download_full: file_id={}, path={}, downloaded={}",
                file.id,
                rel_path,
                downloaded
            );

            // 每秒检查一次文件状态
            if last_status_check.elapsed() >= STATUS_CHECK_INTERVAL {
                last_status_check = std::time::Instant::now();

                // 更新进度并推送到 Redis
                // 非分片下载 chunk_id=0；有 Content-Length 时传入，无则传 0
                helper.progress_tracker.record_bytes(
                    file.id,
                    0,
                    downloaded,
                    content_length, // chunk_total_size = file 大小
                    content_length, // file_total_size
                );

                if helper.is_file_aborted(file.id).await {
                    log_dao
                        .add(
                            file.id,
                            0,
                            user_id,
                            &format!(
                                "download: aborted, file status changed, downloaded={}, path={}",
                                downloaded, rel_path
                            ),
                            None,
                        )
                        .await;
                    return Ok(false);
                }
            }
        }

        file_handle.flush().await?;
        Ok(true)
    }

    /// 设置文件为错误状态
    async fn set_file_error(db: &Pool<MySql>, file: &mut FileModel) {
        let now = now_time().unwrap_or_default();
        file.status = FileStatus::Failed as i8;
        file.change_time = now;
        if let Err(e) = Update::<_, FileModel>::new()
            .set(FileModel::STATUS, FileStatus::Failed as i8)
            .set(FileModel::CHANGE_TIME, now)
            .execute(db, |qb| {
                qb.push_where().field_eq("id", file.id);
            })
            .await
        {
            warn!(
                "download: set_file_error update failed, file_id={}: {}",
                file.id, e
            );
        }
    }

    /// 设置文件和本地记录为错误状态
    async fn set_file_and_local_error(
        db: &Pool<MySql>,
        file: &mut FileModel,
        file_local: &mut FileLocalModel,
        error_msg: &str,
    ) {
        Self::set_file_error(db, file).await;
        if let Err(e) = Update::<sqlx::MySql, FileLocalModel>::new()
            .set(FileLocalModel::LAST_ERROR, error_msg)
            .execute(db, |qb| {
                qb.push_where().field_eq("id", file_local.id);
            })
            .await
        {
            warn!(
                "download: set_file_and_local_error update failed, file_local_id={}: {}",
                file_local.id, e
            );
        }
    }
}
