use std::path::PathBuf;
use std::sync::Arc;

use lsys_core::db::{FieldValue, QueryBuilderExt, TableMeta, Update};
use lsys_core::utils::now_time;
use sqlx::{MySql, Pool};
use tokio::sync::mpsc;
use tracing::{info, warn};

use super::FileResult;
use super::file_helpers::FileHelper;
use super::file_log::FileLogDao;
use crate::common::extract_extension;
use crate::model::*;

/// 下载任务项
#[derive(Debug, Clone)]
pub struct DownloadTask {
    /// file_user.id
    pub file_user_id: u64,
    /// chunk 索引, 对于非分片下载传 0
    pub chunk_index: u32,
    /// 下载完成通知的发送端 (可选, 用于同步等待下载完成)
    pub done_tx: Option<tokio::sync::mpsc::Sender<Result<(), String>>>,
}

/// 下载最大重试次数 (首次 + 重试)
const DOWNLOAD_MAX_RETRIES: u32 = 2;

/// 文件下载管理器
pub struct FileDownloadManager {
    sender: mpsc::UnboundedSender<DownloadTask>,
    receiver: std::sync::Mutex<Option<mpsc::UnboundedReceiver<DownloadTask>>>,
    helper: Arc<FileHelper>,
}

impl FileDownloadManager {
    /// 创建下载管理器。
    pub fn new(helper: Arc<FileHelper>) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            sender: tx,
            receiver: std::sync::Mutex::new(Some(rx)),
            helper,
        }
    }

    /// 运行下载监听后台循环。
    /// 每个实例只能调用一次，通常通过 `tokio::spawn` 调用：
    /// ```rust,ignore
    /// let download = Arc::new(FileDownloadManager::new(helper));
    /// tokio::spawn({ let d = download.clone(); async move { d.listen().await; } });
    /// ```
    pub async fn listen(&self) {
        //@todo 要改成REDIS派发,这样可以多个节点同时处理下载任务,可以引入lsys-core 的多节点运行模块
        let rx = self.receiver.lock().ok().and_then(|mut g| g.take());
        if let Some(rx) = rx {
            Self::download_listener(rx, self.helper.clone()).await;
        }
    }

    /// 推送下载任务
    pub fn push(&self, task: DownloadTask) {
        if let Err(e) = self.sender.send(task) {
            warn!("push download task failed: {}", e);
            // 从失败的 SendError 中取回 done_tx, 及时通知调用方
            if let Some(done_tx) = e.0.done_tx {
                let _ = done_tx.try_send(Err(
                    "push download task failed: download queue closed".to_string()
                ));
            }
        }
    }

    /// 下载监听协程: 多进一出 channel, 控制并发数量
    async fn download_listener(
        mut rx: mpsc::UnboundedReceiver<DownloadTask>,
        helper: Arc<FileHelper>,
    ) {
        info!("file download listener started");
        let mut tasks = tokio::task::JoinSet::new();
        let max_concurrency = helper.config.max_download_concurrency;

        loop {
            // 如果当前执行下载协程数量 >= 最大, 等待一个完成
            while tasks.len() >= max_concurrency {
                if let Some(Err(e)) = tasks.join_next().await {
                    warn!("download task join error: {}", e);
                } else {
                    info!(
                        "download task completed 0, current concurrency: {}",
                        tasks.len()
                    );
                }
            }

            // 使用 select 同时等待新任务和已有任务完成
            tokio::select! {
                task = rx.recv() => {
                    match task {
                        Some(download_task) => {
                            let h = helper.clone();
                            tasks.spawn(async move {
                                // 备份 done_tx, 兜底: 若 execute_download 内部通过 ? 返回 Err
                                // 导致 done_tx 被 drop 而未发送通知, 这里会捕获并发送错误
                                let done_tx_backup = download_task.done_tx.clone();
                                match Self::execute_download(&h, download_task).await {
                                    Ok(()) => {
                                        // 内部已处理通知, 或是分片等待中, 正常释放备份
                                    }
                                    Err(e) => {
                                        warn!("download task error: {}", e);
                                        Self::notify_done(
                                            &done_tx_backup,
                                            Err(format!("download task error: {}", e)),
                                        )
                                        .await;
                                    }
                                }
                            });
                        }
                        None => {
                            // channel 被关闭, 等待所有已启动的下载协程完成
                            info!("download task channel closed, waiting for remaining tasks to complete");
                            break;
                        }
                    }
                }
                Some(result) = tasks.join_next(), if !tasks.is_empty() => {
                    if let Err(e) = result {
                        warn!("download task join error 1: {}", e);
                    }else {
                        info!(
                            "download task completed 1, current concurrency: {}",
                            tasks.len()
                        );
                    }
                }
            }
        }

        // 等待剩余任务完成
        while let Some(result) = tasks.join_next().await {
            if let Err(e) = result {
                warn!("download task join error 2: {}", e);
            } else {
                info!(
                    "download task completed 2, current concurrency: {}",
                    tasks.len()
                );
            }
        }
        info!("file download listener stopped");
    }

    /// 执行单个下载任务
    async fn execute_download(helper: &FileHelper, task: DownloadTask) -> FileResult<()> {
        let db = &helper.db;
        let log_dao = FileLogDao::new(helper.db.clone());

        // 步骤1: 查询 file_user 和 file
        let file_user = match helper.find_file_user_by_id(task.file_user_id).await? {
            Some(fu) => fu,
            None => {
                log_dao
                    .add(0, 0, 0, "download: file_user not found", None)
                    .await;
                Self::notify_done(&task.done_tx, Err("file_user not found".to_string())).await;
                return Ok(());
            }
        };

        let mut file = match helper.find_file_by_id(file_user.file_id).await? {
            Some(f) => f,
            None => {
                log_dao
                    .add(
                        file_user.file_id,
                        0,
                        file_user.user_id,
                        "download: file not found",
                        None,
                    )
                    .await;
                Self::notify_done(&task.done_tx, Err("file not found".to_string())).await;
                return Ok(());
            }
        };

        // source_url 必须是 HTTP 协议
        let source_url = file_user.source_url.trim().to_string();
        if !source_url.starts_with("http://") && !source_url.starts_with("https://") {
            Self::set_file_error(db, &mut file).await;
            log_dao
                .add(
                    file.id,
                    0,
                    file_user.user_id,
                    &format!("download: source_url is not HTTP: {}", source_url),
                    None,
                )
                .await;
            Self::notify_done(
                &task.done_tx,
                Err(format!("source_url is not HTTP: {}", source_url)),
            )
            .await;
            return Ok(());
        }

        // 文件存储类型必须为 local
        if !file.is_local() {
            Self::set_file_error(db, &mut file).await;
            log_dao
                .add(
                    file.id,
                    0,
                    file_user.user_id,
                    "download: storage_type is not local",
                    None,
                )
                .await;
            Self::notify_done(&task.done_tx, Err("storage_type is not local".to_string())).await;
            return Ok(());
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
                        file_user.user_id,
                        "download: file_local not found",
                        None,
                    )
                    .await;
                Self::notify_done(&task.done_tx, Err("file_local not found".to_string())).await;
                return Ok(());
            }
        };

        // 步骤3: 根据 file 状态判断
        if FileStatus::Normal.eq(file.status) {
            if !file_local.local_path.is_empty() {
                let full_path = helper.get_full_local_path(&file.storage_type, &file_local.local_path).await
                    .unwrap_or_else(|_| PathBuf::from(&file_local.local_path));
                if tokio::fs::metadata(&full_path).await.is_ok() {
                    log_dao
                        .add(
                            file.id,
                            0,
                            file_user.user_id,
                            "download: file already completed",
                            None,
                        )
                        .await;
                    Self::notify_done(&task.done_tx, Ok(())).await;
                    return Ok(());
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
                    file_user.user_id,
                    "download: file completed but local file missing",
                    None,
                )
                .await;
            Self::notify_done(
                &task.done_tx,
                Err("file completed but local file missing".to_string()),
            )
            .await;
            return Ok(());
        }

        if !FileStatus::Unfinished.eq(file.status) || !file_local.local_path.is_empty() {
            log_dao
                .add(
                    file.id,
                    0,
                    file_user.user_id,
                    &format!("download: unexpected file status={}", file.status),
                    None,
                )
                .await;
            Self::notify_done(
                &task.done_tx,
                Err(format!("unexpected file status={}", file.status)),
            )
            .await;
            return Ok(());
        }

        // 步骤4/5: 根据 file_chunk_total 决定下载方式
        log_dao
            .add(
                file.id,
                0,
                file_user.user_id,
                &format!(
                    "download: checks passed, start downloading, source_url={}, chunk_total={}",
                    source_url, file_local.file_chunk_total
                ),
                None,
            )
            .await;

        if file_local.file_chunk_total > 0 {
            Self::download_chunked(
                helper,
                &source_url,
                &mut file,
                &mut file_local,
                task.chunk_index,
                file_user.user_id,
                file_user.app_id,
                task.done_tx,
            )
            .await?;
        } else {
            Self::download_single(
                helper,
                db,
                &source_url,
                &mut file,
                &mut file_local,
                file_user.user_id,
                file_user.app_id,
                task.done_tx,
            )
            .await?;
        }

        Ok(())
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
        done_tx: Option<tokio::sync::mpsc::Sender<Result<(), String>>>,
    ) -> FileResult<()> {
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
                Self::notify_done(&done_tx, Err(err_msg)).await;
                return Ok(());
            }
        };

        // 判断状态
        if FileChunkStatus::Normal.eq(chunk.status) {
            // 已完成, 检查文件存在
            if !chunk.chunk_path.is_empty() {
                let full = helper.get_full_local_path(&file.storage_type, &chunk.chunk_path).await
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
                    Self::on_chunk_complete(
                        helper,
                        db,
                        file,
                        file_local,
                        &mut chunk,
                        user_id,
                        app_id,
                        done_tx.clone(),
                    )
                    .await?;
                    return Ok(());
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
            Self::notify_done(
                &done_tx,
                Err(format!("unexpected chunk status={}", chunk.status)),
            )
            .await;
            return Ok(());
        }

        // 4.2 创建新文件并下载
        let ext = extract_extension(Some(&file.file_name));
        let prefix = format!("{}_{}_dlchunk{}", app_id, user_id, chunk_index);
        let (rel_path, full_path) = helper.create_new_file(&file.storage_type, &prefix, ext).await?;

        // 下载 (带重试)
        let download_ok = Self::download_range(
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

        if !download_ok {
            // 4.3 分片下载失败
            chunk.chunk_path = rel_path.clone();
            let now = now_time().unwrap_or_default();
            if let Err(e) = Update::<_, FileLocalChunkModel>::new()
                .set(FileLocalChunkModel::CHUNK_PATH, &rel_path)
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
            Self::set_file_and_local_error(db, file, file_local, "download: chunk download failed")
                .await;
            log_dao
                .add(
                    file.id,
                    chunk.id,
                    user_id,
                    "download: chunk download failed",
                    None,
                )
                .await;
            Self::notify_done(&done_tx, Err("chunk download failed".to_string())).await;
            return Ok(());
        }

        // 设置 chunk_path
        if let Err(e) = Update::<_, FileLocalChunkModel>::new()
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
        chunk.chunk_path = rel_path;

        // 4.4 分片下载完成
        Self::on_chunk_complete(
            helper, db, file, file_local, &mut chunk, user_id, app_id, done_tx,
        )
        .await?;

        Ok(())
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
        done_tx: Option<tokio::sync::mpsc::Sender<Result<(), String>>>,
    ) -> FileResult<()> {
        let log_dao = FileLogDao::new(helper.db.clone());
        let now = now_time().unwrap_or_default();

        // 计算 chunk md5 和 complete_size
        let full = helper.get_full_local_path(&file.storage_type, &chunk.chunk_path).await
            .unwrap_or_else(|_| PathBuf::from(&chunk.chunk_path));
        let metadata = tokio::fs::metadata(&full).await?;
        let file_data = tokio::fs::read(&full).await?;
        let chunk_md5 = format!("{:x}", md5::compute(&file_data));
        let complete_size = metadata.len();

        chunk.chunk_md5 = chunk_md5.clone();
        chunk.complete_size = complete_size;
        chunk.status = FileChunkStatus::Normal as i8;
        chunk.change_time = now;

        // 更新 chunk
        if let Err(e) = Update::<_, FileLocalChunkModel>::new()
            .set(FileLocalChunkModel::STATUS, FileChunkStatus::Normal as i8)
            .set(FileLocalChunkModel::CHANGE_TIME, now)
            .set(FileLocalChunkModel::CHUNK_MD5, &chunk_md5)
            .set(FileLocalChunkModel::COMPLETE_SIZE, complete_size)
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
                        "download: update chunk complete info failed, chunk_id={}: {}",
                        chunk.id, e
                    ),
                    None,
                )
                .await;
        }

        // 更新 file_local: file_chunk_succ+1, file_chunk_size+=complete_size (原子操作)
        if let Err(e) = Update::<_, FileLocalModel>::new()
            .set(
                FileLocalModel::FILE_CHUNK_SUCC,
                FieldValue::Expr("file_chunk_succ + 1".into()),
            )
            .set(
                FileLocalModel::FILE_CHUNK_SIZE,
                FieldValue::Expr(format!("file_chunk_size + {}", complete_size).into()),
            )
            .execute(db, |qb| {
                qb.push_where().field_eq("id", file_local.id);
            })
            .await
        {
            log_dao
                .add(
                    file.id,
                    chunk.id,
                    user_id,
                    &format!(
                        "download: update file_local chunk progress failed, file_local_id={}: {}",
                        file_local.id, e
                    ),
                    None,
                )
                .await;
        }

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

        // 检查所有 chunk 是否都已完成
        let all_chunks = helper.find_chunks_by_file_id(file.id).await?;
        let all_complete = all_chunks
            .iter()
            .all(|c| FileChunkStatus::Normal.eq(c.status));

        if !all_complete {
            log_dao
                .add(
                    file.id,
                    chunk.id,
                    user_id,
                    "download: chunk complete, waiting for others",
                    None,
                )
                .await;
            return Ok(());
        }

        // 所有分片都完成, 合并文件
        let ext = extract_extension(Some(&file.file_name));
        let merge_prefix = format!("{}_{}_dlmerge", app_id, user_id);
        let (merge_rel, merge_full) = helper.create_new_file(&file.storage_type, &merge_prefix, ext).await?;
        match helper.merge_chunk_files(&file.storage_type, &all_chunks, &merge_full).await {
            Err(e) => {
                // 合并失败
                file_local.local_path = merge_rel.clone();
                file.status = FileStatus::Failed as i8;
                file.change_time = now;
                let err_msg = format!("download: merge chunks failed: {}", e);
                Self::set_file_error(db, file).await;
                if let Err(e) = Update::<_, FileLocalModel>::new()
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
                Self::notify_done(&done_tx, Err(err_msg)).await;
            }
            Ok(_) => {
                // 更新所有 chunk status=已合并
                let chunk_ids: Vec<u64> = all_chunks.iter().map(|c| c.id).collect();
                if let Err(e) = Update::<_, FileLocalChunkModel>::new()
                    .set(FileLocalChunkModel::STATUS, FileChunkStatus::Merged as i8)
                    .set(FileLocalChunkModel::CHANGE_TIME, now)
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
                                "download: update chunks merged status failed, file_id={}: {}",
                                file.id, e
                            ),
                            None,
                        )
                        .await;
                }

                // 清理已合并的chunk文件
                helper.cleanup_merged_chunks(chunk_ids, log_dao.clone());

                // 使用辅助函数.2 完成文件
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
                        // 通知下载完成
                        Self::notify_done(&done_tx, Ok(())).await;
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
                        // 通知下载完成
                        Self::notify_done(&done_tx, Ok(())).await;
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
                        Self::notify_done(&done_tx, Err(format!("complete error: {}", e))).await;
                    }
                }
            }
        }

        Ok(())
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
        done_tx: Option<tokio::sync::mpsc::Sender<Result<(), String>>>,
    ) -> FileResult<()> {
        let log_dao = FileLogDao::new(helper.db.clone());

        // 创建新文件
        let ext = extract_extension(Some(&file.file_name));
        let prefix = format!("{}_{}_dl", app_id, user_id);
        let (rel_path, full_path) = helper.create_new_file(&file.storage_type, &prefix, ext).await?;

        // 下载
        let download_ok =
            Self::download_full(helper, source_url, &full_path, file, &log_dao, user_id).await;

        if !download_ok {
            // 步骤6: 下载失败
            file_local.local_path = rel_path.clone();
            if let Err(e) = Update::<_, FileLocalModel>::new()
                .set(FileLocalModel::LOCAL_PATH, &rel_path)
                .execute(db, |qb| {
                    qb.push_where().field_eq("id", file_local.id);
                })
                .await
            {
                log_dao
                    .add(
                        file.id,
                        0,
                        user_id,
                        &format!(
                            "download: update file_local local_path failed, file_local_id={}: {}",
                            file_local.id, e
                        ),
                        None,
                    )
                    .await;
            }
            Self::set_file_and_local_error(
                db,
                file,
                file_local,
                "download: single download failed",
            )
            .await;
            log_dao
                .add(
                    file.id,
                    0,
                    user_id,
                    "download: single download failed",
                    None,
                )
                .await;
            Self::notify_done(&done_tx, Err("single download failed".to_string())).await;
            return Ok(());
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
                // 通知下载完成
                Self::notify_done(&done_tx, Ok(())).await;
            }
            Ok(None) => {
                log_dao
                    .add(file.id, 0, user_id, "download: complete", None)
                    .await;
                // 通知下载完成
                Self::notify_done(&done_tx, Ok(())).await;
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
                Self::notify_done(&done_tx, Err(format!("complete error: {}", e))).await;
            }
        }

        Ok(())
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
    ) -> bool {
        for attempt in 0..DOWNLOAD_MAX_RETRIES {
            if attempt > 0 {
                // 重试: 清空文件
                let _ = tokio::fs::write(target, b"").await;
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
                    return true;
                }
                Ok(false) => continue,
                Err(e) => {
                    log_dao
                        .add(
                            file.id,
                            chunk.id,
                            user_id,
                            &format!("download: range attempt {} error: {}", attempt, e),
                            None,
                        )
                        .await;
                    continue;
                }
            }
        }
        false
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

        let db = &helper.db;
        let timeout_secs = helper.config.download_timeout_secs;
        let client = reqwest::Client::new();
        let end_offset = start_offset + expected_size - 1;
        let range_header = format!("bytes={}-{}", start_offset, end_offset);

        let resp = client
            .get(url)
            .header("Range", &range_header)
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .send()
            .await?;

        if resp.status().as_u16() != 206 {
            return Ok(false);
        }

        let mut file_handle = tokio::fs::File::create(target).await?;
        let mut stream = resp.bytes_stream();
        let mut downloaded: u64 = 0;

        while let Some(bytes_result) = stream.next().await {
            let bytes = bytes_result?;
            file_handle.write_all(&bytes).await?;
            downloaded += bytes.len() as u64;

            // 更新 chunk complete_size
            let now = now_time().unwrap_or_default();
            chunk.complete_size = downloaded;
            if let Err(e) = Update::<_, FileLocalChunkModel>::new()
                .set(FileLocalChunkModel::COMPLETE_SIZE, downloaded)
                .set(FileLocalChunkModel::CHANGE_TIME, now)
                .execute(db, |qb| {
                    qb.push_where().field_eq("id", chunk.id);
                })
                .await
            {
                warn!(
                    "download: update chunk complete_size failed, chunk_id={}: {}",
                    chunk.id, e
                );
            }

            // 原子更新 file.file_size
            if let Err(e) = Update::<_, FileModel>::new()
                .set(
                    FileModel::FILE_SIZE,
                    FieldValue::Expr(format!("file_size + {}", bytes.len() as u64).into()),
                )
                .execute(db, |qb| {
                    qb.push_where().field_eq("id", file.id);
                })
                .await
            {
                warn!(
                    "download: update file.file_size failed, file_id={}: {}",
                    file.id, e
                );
            }

            // 检查文件状态是否已变为错误或删除
            let current_file = sqlx::query_as::<_, FileModel>(&format!(
                "SELECT * FROM {} WHERE id=? LIMIT 1",
                FileModel::table_name(),
            ))
            .bind(file.id)
            .fetch_optional(db)
            .await;

            if let Ok(Some(cf)) = current_file
                && (FileStatus::Failed.eq(cf.status) || FileStatus::Deleted.eq(cf.status))
            {
                return Ok(false);
            }

            if downloaded >= expected_size {
                if downloaded > expected_size {
                    let excess = downloaded - expected_size;
                    let msg = format!(
                        "download: range download received more data than expected, \
                         chunk_id={}, downloaded={}, expected={}, excess={}",
                        chunk.id, downloaded, expected_size, excess
                    );
                    log_dao.add(file.id, chunk.id, user_id, &msg, None).await;
                }
                break;
            }
        }

        file_handle.flush().await?;
        Ok(true)
    }

    /// 完整下载 (非分片), 带重试
    async fn download_full(
        helper: &FileHelper,
        url: &str,
        target: &std::path::PathBuf,
        file: &mut FileModel,
        log_dao: &FileLogDao,
        user_id: u64,
    ) -> bool {
        for attempt in 0..DOWNLOAD_MAX_RETRIES {
            if attempt > 0 {
                let _ = tokio::fs::write(target, b"").await;
            }

            match Self::do_full_download(helper, url, target, file, log_dao, user_id).await {
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
                    return true;
                }
                Ok(false) => continue,
                Err(e) => {
                    log_dao
                        .add(
                            file.id,
                            0,
                            user_id,
                            &format!("download: full attempt {} error: {}", attempt, e),
                            None,
                        )
                        .await;
                    continue;
                }
            }
        }
        false
    }

    async fn do_full_download(
        helper: &FileHelper,
        url: &str,
        target: &std::path::PathBuf,
        file: &mut FileModel,
        log_dao: &FileLogDao,
        user_id: u64,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        use futures_util::StreamExt;
        use tokio::io::AsyncWriteExt;

        let db = &helper.db;
        let timeout_secs = helper.config.download_timeout_secs;
        let client = reqwest::Client::new();
        let resp = client
            .get(url)
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .send()
            .await?;

        if resp.status().as_u16() != 200 {
            return Ok(false);
        }

        let mut file_handle = tokio::fs::File::create(target).await?;
        let mut stream = resp.bytes_stream();

        while let Some(bytes_result) = stream.next().await {
            let bytes = bytes_result?;
            file_handle.write_all(&bytes).await?;

            // 原子更新 file_size
            if let Err(e) = Update::<_, FileModel>::new()
                .set(
                    FileModel::FILE_SIZE,
                    FieldValue::Expr(format!("file_size + {}", bytes.len() as u64).into()),
                )
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
                            "download: update file.file_size failed, file_id={}: {}",
                            file.id, e
                        ),
                        None,
                    )
                    .await;
            }

            // 检查文件状态
            let current_file = sqlx::query_as::<_, FileModel>(&format!(
                "SELECT * FROM {} WHERE id=? LIMIT 1",
                FileModel::table_name(),
            ))
            .bind(file.id)
            .fetch_optional(db)
            .await;

            if let Ok(Some(cf)) = current_file
                && (FileStatus::Failed.eq(cf.status) || FileStatus::Deleted.eq(cf.status))
            {
                return Ok(false);
            }
        }

        file_handle.flush().await?;
        Ok(true)
    }

    /// 发送下载完成/失败通知
    async fn notify_done(
        done_tx: &Option<tokio::sync::mpsc::Sender<Result<(), String>>>,
        result: Result<(), String>,
    ) {
        if let Some(tx) = done_tx
            && let Err(e) = tx.send(result).await
        {
            warn!("download: done_tx send failed: {}", e);
        }
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
        if let Err(e) = Update::<_, FileLocalModel>::new()
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
