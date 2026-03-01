use lsys_core::db::Insert;
use lsys_core::utils::{now_time, RequestEnv};

use super::file_download::DownloadTask;
use super::file_helpers::{ChunkInfo, FileHelper};
use super::logger::*;
use super::*;
use crate::model::*;

impl FileDao {
    // ==================== 创建方法 2: 从URL下载远程文件 ====================
    #[allow(clippy::too_many_arguments)]
    pub async fn create_from_url(
        &self,
        source_url: &str,
        user_id: u64,
        app_id: u64,
        chunks: &[ChunkInfo],
        content_type: Option<&str>,
        tag_names: &[&str],
        wait_timeout: Option<u64>,
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
            for tag_name in tag_names {
                self.tag_dao
                    .add_tag(existing_fu.file_id, user_id, app_id, tag_name, None)
                    .await?;
            }
            return Ok(existing_fu.id);
        }

        let total_size = FileHelper::validate_chunks(chunks)?;
        info!(
            "create_from_url: creating new file, chunks={}, total_size={}",
            chunks.len(),
            total_size
        );

        let mut tx = self.helper.db.begin().await?;

        let tx_result: FileResult<u64> = async {
            let file_res = Insert::<_, FileModel>::new()
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

            Insert::<_, FileLocalModel>::new()
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
                    Insert::<_, FileLocalChunkModel>::new()
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

            let fu_res = Insert::<_, FileUserModel>::new()
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

            self.log_dao
                .add(
                    file_id,
                    0,
                    user_id,
                    &format!("create_from_url: file created, chunks={}", chunks.len()),
                    Some(&mut tx),
                )
                .await;

            for tag_name in tag_names {
                self.tag_dao
                    .add_tag(file_id, user_id, app_id, tag_name, Some(&mut tx))
                    .await?;
            }

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

        // 创建等待通道 (如果需要同步等待下载完成)
        let done_rx = if wait_timeout.is_some() {
            let (tx, rx) = tokio::sync::mpsc::channel::<Result<(), String>>(1);
            // 触发下载
            if chunks.len() > 1 {
                for idx in 0..chunks.len() {
                    self.download_manager.push(DownloadTask {
                        file_user_id,
                        chunk_index: idx as u32,
                        done_tx: Some(tx.clone()),
                    });
                }
            } else {
                self.download_manager.push(DownloadTask {
                    file_user_id,
                    chunk_index: 0,
                    done_tx: Some(tx.clone()),
                });
            }
            drop(tx); // 释放原始 sender, 只保留 task 中的 clone
            Some(rx)
        } else {
            // 触发下载 (无需等待)
            if chunks.len() > 1 {
                for idx in 0..chunks.len() {
                    self.download_manager.push(DownloadTask {
                        file_user_id,
                        chunk_index: idx as u32,
                        done_tx: None,
                    });
                }
            } else {
                self.download_manager.push(DownloadTask {
                    file_user_id,
                    chunk_index: 0,
                    done_tx: None,
                });
            }
            None
        };

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

        // 如果需要等待下载完成
        if let Some(mut rx) = done_rx {
            let timeout_secs = wait_timeout.unwrap_or(0);
            if timeout_secs > 0 {
                // 有超时时间, 超时后返回错误
                match tokio::time::timeout(std::time::Duration::from_secs(timeout_secs), rx.recv())
                    .await
                {
                    Err(_elapsed) => {
                        return Err(FileError::DownloadTimeout(timeout_secs, file_user_id));
                    }
                    Ok(None) => {
                        // channel 关闭但未收到完成通知, 说明下载失败
                        return Err(FileError::DownloadFailed(
                            file_user_id,
                            "channel closed without completion".to_string(),
                        ));
                    }
                    Ok(Some(Err(msg))) => {
                        // 下载器返回失败
                        return Err(FileError::DownloadFailed(file_user_id, msg));
                    }
                    Ok(Some(Ok(()))) => {
                        // 下载完成
                    }
                }
            } else {
                // 超时时间为0, 一直等待直到下载完成
                match rx.recv().await {
                    None => {
                        // channel 关闭但未收到完成通知, 说明下载失败
                        return Err(FileError::DownloadFailed(
                            file_user_id,
                            "channel closed without completion".to_string(),
                        ));
                    }
                    Some(Err(msg)) => {
                        // 下载器返回失败
                        return Err(FileError::DownloadFailed(file_user_id, msg));
                    }
                    Some(Ok(())) => {
                        // 下载完成
                    }
                }
            }
        }

        Ok(file_user_id)
    }
}
