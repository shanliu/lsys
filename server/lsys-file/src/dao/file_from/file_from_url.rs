use lsys_core::db::Insert;
use lsys_core::fluent_message;
use lsys_core::fluents::IntoFluentMessage;
use lsys_core::utils::{RequestEnv, STRING_CLEAR_FORMAT, StringClear, now_time, string_clear};
use lsys_core::valid_key;
use lsys_core::valid_param::{ValidParam, ValidParamCheck, ValidUrl};
use tracing::warn;

use super::super::file_helpers::{ChunkInfo, FileHelper};
use super::super::logger::*;
use super::super::*;
use crate::model::*;

impl FileDao {
    // ==================== 创建方法 2: 从URL下载远程文件 ====================
    ///
    /// - `user_id`: 文件属于的用户ID,0=系统
    /// - `add_user_id`: 文件添加(上传)用户ID
    #[allow(clippy::too_many_arguments)]
    pub async fn create_from_url(
        &self,
        source_url: &str,
        user_id: u64,
        add_user_id: u64,
        app_id: u64,
        storage_type: &str,
        chunks: &[ChunkInfo],
        content_type: Option<&str>,
        tag_names: &[&str],
        expire_time: Option<u64>,
        wait_timeout: Option<u64>,
        env_data: Option<&RequestEnv>,
    ) -> FileResult<u64> {
        use tracing::info;

        info!(
            "create_from_url: starting, user_id={}, url={}",
            user_id, source_url
        );
        let trimmed_url = source_url.trim();

        // 校验 URL 格式
        let trimmed_url_string = trimmed_url.to_string();
        ValidParam::default()
            .add(
                valid_key!("source_url"),
                &trimmed_url_string,
                &ValidParamCheck::default().add_rule(ValidUrl::default()),
            )
            .check()
            .map_err(|e| FileError::Param(e.to_fluent_message()))?;

        let source_md5 = FileHelper::compute_str_md5(trimmed_url);
        let now = now_time()?;
        let content_type = string_clear(
            content_type.unwrap_or(""),
            StringClear::Option(STRING_CLEAR_FORMAT),
            Some(127),
        );

        // 查询是否已存在
        if let Some(existing_fu) = self
            .helper
            .find_file_ref_by_source_md5(user_id, app_id, &source_md5, FileUserStatus::Normal)
            .await?
        {
            // Check whether the associated file actually succeeded — if it Failed, fall through
            // so a fresh download is created instead of returning the stuck failed file_ref.
            let file_status = match self.helper.find_file_by_id(existing_fu.file_id).await? {
                Some(f) => {
                    if FileStatus::Failed.eq(f.status) {
                        FileStatus::Failed
                    } else if FileStatus::Unfinished.eq(f.status) {
                        FileStatus::Unfinished
                    } else {
                        FileStatus::Normal
                    }
                }
                None => FileStatus::Failed, // orphaned file_ref — treat same as failed
            };
            match file_status {
                FileStatus::Normal => {
                    info!(
                        "create_from_url: existing file_ref found (completed), id={}",
                        existing_fu.id
                    );
                    self.tag_dao
                        .batch_add_tags(existing_fu.file_id, user_id, app_id, tag_names, None)
                        .await?;
                    return Ok(existing_fu.id);
                }
                FileStatus::Unfinished => {
                    // 文件仍在下载中，重新触发下载通知（防止任务卡死）
                    info!(
                        "create_from_url: existing file_ref found (unfinished), re-notifying, id={}, file_id={}, trigger_host={}",
                        existing_fu.id, existing_fu.file_id, existing_fu.trigger_host
                    );
                    self.tag_dao
                        .batch_add_tags(existing_fu.file_id, user_id, app_id, tag_names, None)
                        .await?;
                    match self.download_manager.task_notify.notify().await {
                        Ok(()) => info!("create_from_url: re-notify success, file_ref_id={}", existing_fu.id),
                        Err(e) => warn!(
                            "re-notify download task failed for file_ref_id={}: {}",
                            existing_fu.id,
                            e.to_fluent_message().default_format()
                        ),
                    }
                    return Ok(existing_fu.id);
                }
                _ => {
                    info!(
                        "create_from_url: existing file_ref found but file failed (file_id={}), \
                         creating new download",
                        existing_fu.file_id
                    );
                }
            }
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
                .set(FileModel::STORAGE_TYPE, storage_type)
                .set(FileModel::STATUS, FileStatus::Unfinished as i8)
                .set(FileModel::FILE_SIZE, total_size)
                .set(FileModel::FILE_MD5, "")
                .set(
                    FileModel::ORIGIN_NAME,
                    FileHelper::extract_filename_from_url(trimmed_url),
                )
                .set(FileModel::CONTENT_TYPE, content_type)
                .set(FileModel::MODIFY_TIME, 0u64)
                .set(FileModel::FROM_USER_ID, add_user_id)
                .set(FileModel::ADD_TIME, now)
                .set(FileModel::CHANGE_TIME, 0u64)
                .set(FileModel::LOCAL_PATH_OWNER_ID, 0u64)
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

            let fu_res = Insert::<_, FileRefModel>::new()
                .set(FileRefModel::USER_ID, user_id)
                .set(FileRefModel::ADD_USER_ID, add_user_id)
                .set(FileRefModel::APP_ID, app_id)
                .set(FileRefModel::FILE_ID, file_id)
                .set(FileRefModel::STATUS, FileUserStatus::Normal as i8)
                .set(FileRefModel::SOURCE_URL, trimmed_url)
                .set(FileRefModel::SOURCE_MD5, &source_md5)
                .set(
                    FileRefModel::FILE_NAME,
                    FileHelper::extract_filename_from_url(trimmed_url),
                )
                .set(
                    FileRefModel::TRIGGER_HOST,
                    hostname::get()
                        .ok()
                        .and_then(|h| h.into_string().ok())
                        .unwrap_or_default(),
                )
                .set(FileRefModel::ADD_TIME, now)
                .set(FileRefModel::DELETE_TIME, 0u64)
                .set(FileRefModel::EXPIRE_TIME, expire_time.unwrap_or(0))
                .execute(&mut *tx)
                .await?;

            let file_ref_id = fu_res.last_insert_id();

            self.log_dao
                .add(
                    file_id,
                    0,
                    user_id,
                    &format!("create_from_url: file created, chunks={}", chunks.len()),
                    Some(&mut tx),
                )
                .await;

            self.tag_dao
                .batch_add_tags(file_id, user_id, app_id, tag_names, Some(&mut tx))
                .await?;

            Ok(file_ref_id)
        }
        .await;

        let file_ref_id = match tx_result {
            Ok(id) => {
                tx.commit().await?;
                id
            }
            Err(e) => {
                if let Err(rb_err) = tx.rollback().await {
                    warn!("create_from_url: rollback failed: {}", rb_err);
                }
                return Err(e);
            }
        };

        self.logger
            .add(
                &LogFileCreate {
                    action: "create_from_url",
                    storage_type,
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

        // 如果需要等待下载完成，先注册等待监听器（避免竞态条件）
        let wait_receiver = if wait_timeout.is_some() {
            Some(
                self.download_manager
                    .wait_notify
                    .wait_download(file_ref_id)
                    .await,
            )
        } else {
            None
        };

        // 触发下载任务（在注册等待监听器之后）
        // 如果通知失败且用户在等待，直接返回错误
        let trigger_host = hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_default();
        info!(
            "create_from_url: notifying task, file_ref_id={}, trigger_host={}, wait_timeout={:?}",
            file_ref_id, trigger_host, wait_timeout
        );
        match self.download_manager.task_notify.notify().await {
            Ok(()) => info!("create_from_url: notify success, file_ref_id={}", file_ref_id),
            Err(e) => {
                if wait_receiver.is_some() {
                    // 用户在等待同步结果，通知失败应该立即返回错误
                    return Err(FileError::AppCore(e));
                }
                // 用户不等待，只记录警告，任务可能在后台轮询时处理
                warn!(
                    "push download task failed for file_ref_id={}: {}",
                    file_ref_id,
                    e.to_fluent_message().default_format()
                );
            }
        }

        // 等待下载完成
        if let Some(rx) = wait_receiver {
            match self.download_manager.wait_notify.wait_timeout(rx).await {
                Ok(Ok(true)) => {
                    // 下载成功
                }
                Ok(Ok(false)) => {
                    return Err(FileError::DownloadFailed(
                        file_ref_id,
                        "download failed".to_string(),
                    ));
                }
                Ok(Err(msg)) => {
                    return Err(FileError::DownloadFailed(file_ref_id, msg));
                }
                Err(lsys_core::listen_notify::WaitNotifyError::TimeOut) => {
                    return Err(FileError::DownloadTimeout(
                        wait_timeout.unwrap_or(0),
                        file_ref_id,
                    ));
                }
                Err(e) => {
                    return Err(FileError::System(fluent_message!(
                        "file-download-wait-error",
                        e.to_fluent_message()
                    )));
                }
            }
        }

        Ok(file_ref_id)
    }

    /// 从 URL 创建文件（自动探测+分片）
    ///
    /// 内部探测 URL 总大小并校验大小上限，按 `download_chunk_max` 平均拆分分片后调用
    /// [`create_from_url`](Self::create_from_url)。调用方无需自行决定分片数量。
    ///
    /// - 总大小已知且支持 206 Range：按总大小平均拆分多片
    /// - 总大小已知但不支持 Range：单分片（记录总大小）
    /// - 总大小未知：单分片（未知总大小）
    #[allow(clippy::too_many_arguments)]
    pub async fn create_from_url_auto(
        &self,
        source_url: &str,
        user_id: u64,
        add_user_id: u64,
        app_id: u64,
        storage_type: &str,
        tag_names: &[&str],
        expire_time: Option<u64>,
        wait_timeout: Option<u64>,
        env_data: Option<&RequestEnv>,
    ) -> FileResult<u64> {
        // 探测 URL 信息
        let url_info = self.helper.get_url_file_info(source_url).await?;

        // 文件大小校验
        let max_upload_size = self.runtime_setting.get_upload_max_file_size().await?;
        if let Some(file_size) = url_info.file_size
            && max_upload_size > 0
            && file_size > max_upload_size
        {
            return Err(FileError::Param(fluent_message!(
                "file-size-too-large",
                {"size": file_size, "max": max_upload_size}
            )));
        }

        // 根据探测信息构建分片
        let chunks = match url_info.file_size {
            Some(file_size) if file_size > 0 => {
                if url_info.supports_range {
                    self.helper.create_download_chunks(file_size)?
                } else {
                    vec![ChunkInfo {
                        offset: 0,
                        len: file_size,
                        md5: None,
                    }]
                }
            }
            // 未知文件大小，使用单分片
            _ => vec![ChunkInfo {
                offset: 0,
                len: 0,
                md5: None,
            }],
        };

        self.create_from_url(
            source_url,
            user_id,
            add_user_id,
            app_id,
            storage_type,
            &chunks,
            url_info.content_type.as_deref(),
            tag_names,
            expire_time,
            wait_timeout,
            env_data,
        )
        .await
    }
}
