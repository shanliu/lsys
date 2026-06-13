use std::path::PathBuf;
use std::time::Duration;

use fs2::FileExt;
use lsys_core::db::{FieldValue, Insert, QueryBuilderExt, Update};
use lsys_core::dist_lock::{DistLockError, WatchdogConfig};
use lsys_core::fluent_message;
use lsys_core::utils::{RequestEnv, STRING_CLEAR_FORMAT, StringClear, now_time, string_clear};
use tracing::warn;

use super::super::file_helpers::ChunkInfo;
use super::super::logger::*;
use super::super::*;
use crate::model::*;

/// 写文件句柄包装
pub struct FileWriteHandle {
    pub file: FileModel,
    pub file_local: FileLocalModel,
    pub file_local_chunk: Option<FileLocalChunkModel>,
    pub handle: tokio::fs::File,
    pub app_id: u64,
    /// 上次状态检查时间，用于节流 DB 查询
    pub(crate) last_status_check: std::time::Instant,
}

impl FileDao {
    /// 创建上传函数
    /// 返回 (file_id, file_ref_id)
    ///
    /// - `user_id`: 文件属于的用户ID,0=系统
    /// - `add_user_id`: 文件添加(上传)用户ID
    #[allow(clippy::too_many_arguments)]
    pub async fn create_upload(
        &self,
        user_id: u64,
        add_user_id: u64,
        app_id: u64,
        storage_type: &str,
        chunks: &[ChunkInfo],
        file_name: &str,
        tag_names: &[&str],
        expire_time: Option<u64>,
        env_data: Option<&RequestEnv>,
    ) -> FileResult<(u64, u64)> {
        if chunks.is_empty() {
            return Err(FileError::Param(fluent_message!("file-chunks-empty")));
        }
        if !crate::model::FileModel::is_local_key(storage_type) {
            return Err(FileError::Param(fluent_message!("file-storage-type-invalid")));
        }

        let total_size: u64 = chunks.iter().map(|c| c.len).sum();
        let upload_max_file_size = self.runtime_setting.get_upload_max_file_size().await?;
        if upload_max_file_size > 0 && total_size > upload_max_file_size {
            return Err(FileError::Param(fluent_message!(
                "file-error",
                &format!(
                    "upload file size exceeded: total_size={}, max_size={}",
                    total_size, upload_max_file_size
                )
            )));
        }

        //非重要信息不校验过滤即可
        let file_name = string_clear(
            file_name,
            StringClear::Option(STRING_CLEAR_FORMAT),
            Some(254),
        );
        let now = now_time()?;
        let mut tx = self.helper.db.begin().await?;

        let tx_result: FileResult<(u64, u64)> = async {
            let file_res;
            if chunks.len() == 1 {
                let chunk = &chunks[0];
                let chunk_md5 = chunk.md5.as_deref().unwrap_or("");

                file_res = Insert::<_, FileModel>::new()
                    .set(FileModel::STORAGE_TYPE, storage_type)
                    .set(FileModel::STATUS, FileStatus::Unfinished as i8)
                    .set(FileModel::FILE_MD5, chunk_md5)
                    .set(FileModel::FILE_SIZE, chunk.len)
                    .set(FileModel::ORIGIN_NAME, &file_name)
                    .set(FileModel::CONTENT_TYPE, "")
                    .set(FileModel::MODIFY_TIME, 0u64)
                    .set(FileModel::FROM_USER_ID, add_user_id)
                    .set(FileModel::ADD_TIME, now)
                    .set(FileModel::CHANGE_TIME, 0u64)
                    .set(FileModel::LOCAL_PATH_OWNER_ID, 0u64)
                    .execute(&mut *tx)
                    .await?;

                let file_id = file_res.last_insert_id();

                Insert::<_, FileLocalModel>::new()
                    .set(FileLocalModel::FILE_ID, file_id)
                    .set(FileLocalModel::SOURCE_TYPE, FileSourceType::Upload as i8)
                    .set(FileLocalModel::SOURCE_NAME, "")
                    .set(FileLocalModel::LOCAL_PATH, "")
                    .set(FileLocalModel::FILE_CHUNK_TOTAL, 0u32)
                    .set(FileLocalModel::FILE_CHUNK_SUCC, 0u32)
                    .set(FileLocalModel::FILE_CHUNK_SIZE, 0u64)
                    .set(FileLocalModel::LAST_ERROR, "")
                    .execute(&mut *tx)
                    .await?;

                let fu_res = Insert::<_, FileRefModel>::new()
                    .set(FileRefModel::USER_ID, user_id)
                    .set(FileRefModel::ADD_USER_ID, add_user_id)
                    .set(FileRefModel::APP_ID, app_id)
                    .set(FileRefModel::FILE_ID, file_id)
                    .set(FileRefModel::STATUS, FileUserStatus::Normal as i8)
                    .set(FileRefModel::SOURCE_URL, "")
                    .set(FileRefModel::SOURCE_MD5, "")
                    .set(FileRefModel::FILE_NAME, &file_name)
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
                        add_user_id,
                        "create_upload: file created",
                        Some(&mut tx),
                    )
                    .await;

                self.tag_dao
                    .batch_add_tags(file_id, user_id, app_id, tag_names, Some(&mut tx))
                    .await?;

                Ok((file_id, file_ref_id))
            } else {
                file_res = Insert::<_, FileModel>::new()
                    .set(FileModel::STORAGE_TYPE, storage_type)
                    .set(FileModel::STATUS, FileStatus::Unfinished as i8)
                    .set(FileModel::FILE_MD5, "")
                    .set(FileModel::FILE_SIZE, total_size)
                    .set(FileModel::ORIGIN_NAME, &file_name)
                    .set(FileModel::CONTENT_TYPE, "")
                    .set(FileModel::MODIFY_TIME, 0u64)
                    .set(FileModel::FROM_USER_ID, add_user_id)
                    .set(FileModel::ADD_TIME, now)
                    .set(FileModel::CHANGE_TIME, 0u64)
                    .set(FileModel::LOCAL_PATH_OWNER_ID, 0u64)
                    .execute(&mut *tx)
                    .await?;

                let file_id = file_res.last_insert_id();

                Insert::<_, FileLocalModel>::new()
                    .set(FileLocalModel::FILE_ID, file_id)
                    .set(FileLocalModel::SOURCE_TYPE, FileSourceType::Upload as i8)
                    .set(FileLocalModel::SOURCE_NAME, "")
                    .set(FileLocalModel::LOCAL_PATH, "")
                    .set(FileLocalModel::FILE_CHUNK_TOTAL, chunks.len() as u32)
                    .set(FileLocalModel::FILE_CHUNK_SUCC, 0u32)
                    .set(FileLocalModel::FILE_CHUNK_SIZE, 0u64)
                    .set(FileLocalModel::LAST_ERROR, "")
                    .execute(&mut *tx)
                    .await?;

                for (idx, chunk) in chunks.iter().enumerate() {
                    Insert::<_, FileLocalChunkModel>::new()
                        .set(FileLocalChunkModel::FILE_ID, file_id)
                        .set(FileLocalChunkModel::CHUNK_INDEX, idx as u32)
                        .set(FileLocalChunkModel::START_OFFSET, chunk.offset)
                        .set(FileLocalChunkModel::CHUNK_MD5, "")
                        .set(
                            FileLocalChunkModel::UPLOAD_MD5,
                            chunk.md5.as_deref().unwrap_or(""),
                        )
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

                let fu_res = Insert::<_, FileRefModel>::new()
                    .set(FileRefModel::USER_ID, user_id)
                    .set(FileRefModel::APP_ID, app_id)
                    .set(FileRefModel::ADD_USER_ID, add_user_id)
                    .set(FileRefModel::FILE_ID, file_id)
                    .set(FileRefModel::STATUS, FileUserStatus::Normal as i8)
                    .set(FileRefModel::SOURCE_URL, "")
                    .set(FileRefModel::SOURCE_MD5, "")
                    .set(FileRefModel::FILE_NAME, &file_name)
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
                        "create_upload: file created",
                        Some(&mut tx),
                    )
                    .await;

                self.tag_dao
                    .batch_add_tags(file_id, user_id, app_id, tag_names, Some(&mut tx))
                    .await?;

                Ok((file_id, file_ref_id))
            }
        }
        .await;

        let (file_id, file_ref_id) = match tx_result {
            Ok(id) => {
                tx.commit().await?;
                id
            }
            Err(e) => {
                if let Err(rb_err) = tx.rollback().await {
                    warn!("create_upload: rollback failed: {}", rb_err);
                }
                return Err(e);
            }
        };

        self.logger
            .add(
                &LogFileUpload {
                    action: "create_upload",
                    user_id,
                    file_id,
                    file_name: &file_name,
                    chunk_count: chunks.len(),
                },
                Some(file_id),
                Some(user_id),
                None,
                env_data,
            )
            .await;

        Ok((file_id, file_ref_id))
    }

    /// 通过 file_ref_id 获取上传文件写句柄
    /// 前端传入 file_ref_id 而非 file_id, 内部解析 file_ref → file → get_upload_handle
    /// app_id 从 file_ref 记录中获取, 无需外部传入
    pub async fn get_upload_handle_by_file_ref_id(
        &self,
        file_ref_id: u64,
        chunk_index: u32,
    ) -> FileResult<FileWriteHandle> {
        let file_ref = self
            .helper
            .find_file_ref_by_id(file_ref_id)
            .await?
            .ok_or_else(|| FileError::Param(fluent_message!("file-ref-not-found")))?;

        let file = self
            .helper
            .find_file_by_id(file_ref.file_id)
            .await?
            .ok_or_else(|| FileError::Param(fluent_message!("file-not-found")))?;

        // 校验文件上传用户 (from_user_id 是上传者, add_user_id 是添加记录的用户)
        if file.from_user_id != file_ref.add_user_id {
            return Err(FileError::Param(fluent_message!("file-user-mismatch")));
        }

        self.get_upload_handle(&file, chunk_index, file_ref.app_id)
            .await
    }

    /// 获取上传文件写句柄 (内部方法)
    pub(crate) async fn get_upload_handle(
        &self,
        file: &FileModel,
        chunk_index: u32,
        app_id: u64,
    ) -> FileResult<FileWriteHandle> {
        let result: FileResult<FileWriteHandle> = async {
            // 上传时限检查：add_time + upload_max_duration < now() 则拒绝继续上传
            let now = lsys_core::utils::now_time()?;
            let deadline = file.add_time.saturating_add(self.helper.config.upload_max_duration);
            if now > deadline {
                return Err(FileError::Param(fluent_message!("file-upload-timeout")));
            }

            let mut file_local = self
                .helper
                .find_file_local_by_file_id(file.id)
                .await?
                .ok_or_else(|| FileError::Param(fluent_message!("file-local-not-found")))?;

            if file_local.file_chunk_total > 1 {
                // 多分片
                if chunk_index >= file_local.file_chunk_total {
                    return Err(FileError::Param(fluent_message!(
                        "file-chunk-index-out-of-range"
                    )));
                }

                let mut chunk = self
                    .helper
                    .find_chunk_by_file_and_index(file.id, chunk_index)
                    .await?
                    .ok_or_else(|| FileError::Param(fluent_message!("file-chunk-not-found")))?;

                let (handle, _rel_path) = if !chunk.chunk_path.is_empty() {
                    // 已存在路径, 获取写锁
                    let full = self
                        .helper
                        .get_full_local_path(&file.storage_type, &chunk.chunk_path)
                        .await
                        .unwrap_or_else(|_| PathBuf::from(&chunk.chunk_path));
                    let f = tokio::fs::OpenOptions::new()
                        .write(true)
                        .create(true)
                        .truncate(false)
                        .open(&full)
                        .await?;
                    (f, chunk.chunk_path.clone())
                } else {
                    // 新增文件
                    let chunk_ext = crate::common::extract_extension(Some(&file.origin_name));
                    let prefix = format!("{}_{}_chunk{}", app_id, file.from_user_id, chunk_index);
                    let (rel, full) = self
                        .helper
                        .create_new_file(&file.storage_type, &prefix, chunk_ext)
                        .await?;
                    let f = match tokio::fs::OpenOptions::new()
                        .write(true)
                        .open(&full)
                        .await
                    {
                        Ok(f) => f,
                        Err(e) => {
                            // create_new_file 成功但 open 失败，清理孤儿文件
                            if let Err(re) = tokio::fs::remove_file(&full).await {
                                warn!("get_upload_handle: remove chunk file after open failure: {}", re);
                            }
                            return Err(e.into());
                        }
                    };

                    // 保存路径；若 DB 写入失败则清理已创建的物理文件
                    chunk.chunk_path = rel.clone();
                    if let Err(e) = Update::<sqlx::MySql, FileLocalChunkModel>::new()
                        .set(FileLocalChunkModel::CHUNK_PATH, &rel)
                        .execute(&self.helper.db, |qb| {
                            qb.push_where().field_eq("id", chunk.id);
                        })
                        .await
                    {
                        if let Err(re) = tokio::fs::remove_file(&full).await {
                            warn!("get_upload_handle: remove chunk file after db failure: {}", re);
                        }
                        return Err(e.into());
                    }
                    (f, rel)
                };

                // 获取排他文件锁，防止并发写入冲突
                let handle = {
                    let std_file = handle
                        .try_into_std()
                        .map_err(|_| FileError::System(fluent_message!("file-lock-error")))?;
                    std_file.try_lock_exclusive()?;
                    tokio::fs::File::from_std(std_file)
                };

                Ok(FileWriteHandle {
                    file: file.clone(),
                    file_local,
                    file_local_chunk: Some(chunk),
                    handle,
                    app_id,
                    last_status_check: std::time::Instant::now(),
                })
            } else {
                // 单文件
                let (handle, _rel_path) = if !file_local.local_path.is_empty() {
                    let full = self
                        .helper
                        .get_full_local_path(&file.storage_type, &file_local.local_path)
                        .await
                        .unwrap_or_else(|_| PathBuf::from(&file_local.local_path));
                    let f = tokio::fs::OpenOptions::new()
                        .write(true)
                        .create(true)
                        .truncate(false)
                        .open(&full)
                        .await?;
                    (f, file_local.local_path.clone())
                } else {
                    let upload_ext = crate::common::extract_extension(Some(&file.origin_name));
                    let prefix = format!("{}_{}_upload", app_id, file.from_user_id);
                    let (rel, full) = self
                        .helper
                        .create_new_file(&file.storage_type, &prefix, upload_ext)
                        .await?;
                    let f = match tokio::fs::OpenOptions::new()
                        .write(true)
                        .open(&full)
                        .await
                    {
                        Ok(f) => f,
                        Err(e) => {
                            // create_new_file 成功但 open 失败，清理孤儿文件
                            if let Err(re) = tokio::fs::remove_file(&full).await {
                                warn!("get_upload_handle: remove file after open failure: {}", re);
                            }
                            return Err(e.into());
                        }
                    };

                    // 保存路径；若 DB 写入失败则清理已创建的物理文件
                    file_local.local_path = rel.clone();
                    if let Err(e) = Update::<sqlx::MySql, FileLocalModel>::new()
                        .set(FileLocalModel::LOCAL_PATH, &rel)
                        .execute(&self.helper.db, |qb| {
                            qb.push_where().field_eq("id", file_local.id);
                        })
                        .await
                    {
                        if let Err(re) = tokio::fs::remove_file(&full).await {
                            warn!("get_upload_handle: remove file after db failure: {}", re);
                        }
                        return Err(e.into());
                    }
                    (f, rel)
                };

                // 获取排他文件锁，防止并发写入冲突
                let handle = {
                    let std_file = handle
                        .try_into_std()
                        .map_err(|_| FileError::System(fluent_message!("file-lock-error")))?;
                    std_file.try_lock_exclusive()?;
                    tokio::fs::File::from_std(std_file)
                };

                Ok(FileWriteHandle {
                    file: file.clone(),
                    file_local,
                    file_local_chunk: None,
                    handle,
                    app_id,
                    last_status_check: std::time::Instant::now(),
                })
            }
        }
        .await;

        match &result {
            Ok(h) => {
                let chunk_id = h.file_local_chunk.as_ref().map(|c| c.id).unwrap_or(0);
                self.log_dao
                    .add(
                        file.id,
                        chunk_id,
                        file.from_user_id,
                        &format!(
                            "get_upload_handle: file initialized, chunk_index={}",
                            chunk_index
                        ),
                        None,
                    )
                    .await;
            }
            Err(e) => {
                self.log_dao
                    .add(
                        file.id,
                        0,
                        file.from_user_id,
                        &format!("get_upload_handle: failed: {}", e),
                        None,
                    )
                    .await;
            }
        }

        result
    }

    /// 写入文件函数
    pub async fn write_file(
        &self,
        write_handle: &mut FileWriteHandle,
        data: &[u8],
    ) -> FileResult<usize> {
        use tokio::io::AsyncWriteExt;

        if write_handle.file_local.file_chunk_total > 1 {
            let chunk = write_handle
                .file_local_chunk
                .as_mut()
                .ok_or_else(|| FileError::Param(fluent_message!("file-chunk-required")))?;
            tracing::trace!(
                "write_file: file_id={}, chunk_index={}, path={}, bytes={}",
                write_handle.file_local.file_id,
                chunk.chunk_index,
                chunk.chunk_path,
                data.len()
            );
        } else {
            if write_handle.file_local_chunk.is_some() {
                return Err(FileError::Param(fluent_message!("file-chunk-unexpected")));
            }
            tracing::trace!(
                "write_file: file_id={}, path={}, bytes={}",
                write_handle.file_local.file_id,
                write_handle.file_local.local_path,
                data.len()
            );
        }

        write_handle.handle.write_all(data).await?;

        // 每秒检查一次 file/chunk 状态，防止上传途中被外部删除/取消
        const STATUS_CHECK_INTERVAL: std::time::Duration = std::time::Duration::from_secs(1);
        if write_handle.last_status_check.elapsed() >= STATUS_CHECK_INTERVAL {
            write_handle.last_status_check = std::time::Instant::now();
            let file_id = write_handle.file.id;
            let chunk_id = write_handle
                .file_local_chunk
                .as_ref()
                .map(|c| c.id)
                .unwrap_or(0);

            if self.helper.is_file_aborted(file_id).await {
                self.log_dao
                    .add(
                        file_id,
                        chunk_id,
                        write_handle.file.from_user_id,
                        "write_file: aborted, file status changed",
                        None,
                    )
                    .await;
                return Err(FileError::System(fluent_message!("file-upload-aborted")));
            }

            if chunk_id > 0 && self.helper.is_chunk_aborted(chunk_id).await {
                self.log_dao
                    .add(
                        file_id,
                        chunk_id,
                        write_handle.file.from_user_id,
                        "write_file: aborted, chunk status changed",
                        None,
                    )
                    .await;
                return Err(FileError::System(fluent_message!("file-upload-aborted")));
            }
        }

        Ok(data.len())
    }

    /// 完成文件函数
    pub async fn complete_upload(
        &self,
        mut write_handle: FileWriteHandle,
        env_data: Option<&RequestEnv>,
    ) -> FileResult<FileModel> {
        use tokio::io::AsyncWriteExt;
        write_handle.handle.flush().await?;
        // 显式解锁文件后再关闭句柄
        let std_file = write_handle
            .handle
            .try_into_std()
            .map_err(|_| FileError::System(fluent_message!("file-lock-error")))?;
        std_file.unlock()?;
        drop(std_file);

        let now = now_time()?;

        if write_handle.file_local.file_chunk_total > 1 {
            // 多分片完成
            let mut chunk = write_handle
                .file_local_chunk
                .ok_or_else(|| FileError::Param(fluent_message!("file-chunk-required")))?;

            let chunk_full = self
                .helper
                .get_full_local_path(&write_handle.file.storage_type, &chunk.chunk_path)
                .await
                .unwrap_or_else(|_| PathBuf::from(&chunk.chunk_path));
            let metadata = tokio::fs::metadata(&chunk_full).await?;
            let chunk_data = tokio::fs::read(&chunk_full).await?;
            let chunk_md5 = format!("{:x}", md5::compute(&chunk_data));

            let actual_size = metadata.len();

            // 校验单分片实际大小不超过上传分片上限
            let chunk_limit = self.helper.config.upload_chunk_max;
            if chunk_limit > 0 && actual_size > chunk_limit {
                if let Err(re) = tokio::fs::remove_file(&chunk_full).await {
                    warn!("complete_upload: remove oversized chunk file failed: {}", re);
                }
                Update::<_, FileLocalChunkModel>::new()
                    .set(FileLocalChunkModel::STATUS, FileChunkStatus::Failed as i8)
                    .set(FileLocalChunkModel::CHANGE_TIME, now)
                    .execute(&self.helper.db, |qb| {
                        qb.push_where().field_eq("id", chunk.id);
                    })
                    .await
                    .ok();
                return Err(FileError::Param(fluent_message!(
                    "file-error",
                    &format!(
                        "upload chunk size exceeded: actual_size={}, max_chunk={}",
                        actual_size, chunk_limit
                    )
                )));
            }

            let status = if !chunk.upload_md5.is_empty() && chunk_md5 != chunk.upload_md5 {
                // MD5 校验失败，删除已落盘的分片文件
                if let Err(re) = tokio::fs::remove_file(&chunk_full).await {
                    warn!("complete_upload: remove md5-mismatch chunk file failed: {}", re);
                }
                FileChunkStatus::Failed as i8
            } else {
                FileChunkStatus::Normal as i8
            };

            chunk.chunk_md5 = chunk_md5.clone();
            chunk.complete_size = actual_size;
            chunk.status = status;

            Update::<_, FileLocalChunkModel>::new()
                .set(FileLocalChunkModel::FILE_SIZE, actual_size)
                .set(FileLocalChunkModel::CHUNK_MD5, &chunk_md5)
                .set(FileLocalChunkModel::STATUS, status)
                .set(FileLocalChunkModel::COMPLETE_SIZE, actual_size)
                .set(FileLocalChunkModel::CHANGE_TIME, now)
                .execute(&self.helper.db, |qb| {
                    qb.push_where().field_eq("id", chunk.id);
                })
                .await?;

            // file_local: file_chunk_succ+1, file_chunk_size+=actual_size (原子操作)
            Update::<_, FileLocalModel>::new()
                .set(
                    FileLocalModel::FILE_CHUNK_SUCC,
                    FieldValue::Expr("file_chunk_succ + 1".into()),
                )
                .set(
                    FileLocalModel::FILE_CHUNK_SIZE,
                    FieldValue::Expr(format!("file_chunk_size + {}", actual_size).into()),
                )
                .execute(&self.helper.db, |qb| {
                    qb.push_where().field_eq("id", write_handle.file_local.id);
                })
                .await?;

            self.log_dao
                .add(
                    write_handle.file.id,
                    chunk.id,
                    write_handle.file.from_user_id,
                    &format!("complete_upload: chunk {} done", chunk.chunk_index),
                    None,
                )
                .await;

            // 检查所有 chunk 是否都=1(正常)
            let helper = &self.helper;
            let all_chunks = helper.find_chunks_by_file_id(write_handle.file.id).await?;
            let all_normal = all_chunks
                .iter()
                .all(|c| FileChunkStatus::Normal.eq(c.status));

            if all_normal {
                let file_id = write_handle.file.id;

                // 全部分片就绪，校验整体上传大小不超过 max_upload_size
                let total_size: u64 = all_chunks.iter().map(|c| c.complete_size).sum();
                let max_upload_size = self.runtime_setting.get_upload_max_file_size().await?;
                if max_upload_size > 0 && total_size > max_upload_size {
                    for c in &all_chunks {
                        if !c.chunk_path.is_empty() {
                            let full = helper
                                .get_full_local_path(&write_handle.file.storage_type, &c.chunk_path)
                                .await
                                .unwrap_or_else(|_| PathBuf::from(&c.chunk_path));
                            if let Err(re) = tokio::fs::remove_file(&full).await {
                                warn!("complete_upload: remove oversized chunk file failed: {}", re);
                            }
                        }
                    }
                    Update::<sqlx::MySql, FileModel>::new()
                        .set(FileModel::STATUS, FileStatus::Failed as i8)
                        .set(FileModel::CHANGE_TIME, now)
                        .execute(&self.helper.db, |qb| {
                            qb.push_where().field_eq("id", write_handle.file.id);
                        })
                        .await
                        .ok();
                    return Err(FileError::Param(fluent_message!(
                        "file-error",
                        &format!(
                            "upload file size exceeded: total_size={}, max_size={}",
                            total_size, max_upload_size
                        )
                    )));
                }

                // 幂等检查：若文件已完成（并发 winner 已完结），直接跳过合并
                let still_unfinished = helper
                    .find_file_by_id(file_id)
                    .await?
                    .map(|f| FileStatus::Unfinished.eq(f.status))
                    .unwrap_or(false);

                if still_unfinished {
                    // 非阻塞分布式锁，防止多节点并发重复合并
                    // AcquireFailed 时立即返回，由前端稍后重试，不阻塞任何线程
                    let lock_key = format!("file_merge:{}", file_id);
                    let lock_ttl = Duration::from_secs(self.helper.config.upload_max_duration);
                    let _merge_guard = match self
                        .helper
                        .sync_locker
                        .try_lock_with_watchdog(
                            &lock_key,
                            lock_ttl,
                            WatchdogConfig {
                                max_duration: Some(lock_ttl),
                                ..Default::default()
                            },
                        )
                        .await
                    {
                        Ok(guard) => guard,
                        Err(DistLockError::AcquireFailed { .. }) => {
                            return Err(FileError::Lock(DistLockError::AcquireFailed {
                                key: lock_key,
                            }));
                        }
                        Err(e) => return Err(FileError::Lock(e)),
                    };

                    // 锁内二次确认（防止步骤1与加锁之间 winner 已完成）
                    let confirmed_unfinished = helper
                        .find_file_by_id(file_id)
                        .await?
                        .map(|f| FileStatus::Unfinished.eq(f.status))
                        .unwrap_or(false);

                    if confirmed_unfinished {
                        // 合并文件
                        let merge_ext =
                            crate::common::extract_extension(Some(&write_handle.file.origin_name));
                        let merge_prefix = format!(
                            "{}_{}_merge",
                            write_handle.app_id, write_handle.file.from_user_id
                        );
                        let (merge_rel, merge_full) = helper
                            .create_new_file(
                                &write_handle.file.storage_type,
                                &merge_prefix,
                                merge_ext,
                            )
                            .await?;
                        match helper
                            .merge_chunk_files(
                                &write_handle.file.storage_type,
                                &all_chunks,
                                &merge_full,
                            )
                            .await
                        {
                            Err(e) => {
                                // 合并失败，清理临时合并文件
                                if let Err(re) = tokio::fs::remove_file(&merge_full).await {
                                    warn!("complete_upload: remove failed merge file failed: {}", re);
                                }

                                write_handle.file.status = FileStatus::Failed as i8;
                                write_handle.file.change_time = now;

                                Update::<sqlx::MySql, FileModel>::new()
                                    .set(FileModel::STATUS, FileStatus::Failed as i8)
                                    .set(FileModel::CHANGE_TIME, now)
                                    .execute(&self.helper.db, |qb| {
                                        qb.push_where().field_eq("id", write_handle.file.id);
                                    })
                                    .await?;

                                self.log_dao
                                    .add(
                                        write_handle.file.id,
                                        0,
                                        write_handle.file.from_user_id,
                                        &format!("complete_upload: merge failed: {}", e),
                                        None,
                                    )
                                    .await;
                            }
                            Ok(_) => {
                                // 更新所有 chunk status=已合并
                                let chunk_ids: Vec<u64> =
                                    all_chunks.iter().map(|c| c.id).collect();
                                if let Err(e) = Update::<_, FileLocalChunkModel>::new()
                                    .set(FileLocalChunkModel::STATUS, FileChunkStatus::Merged as i8)
                                    .set(FileLocalChunkModel::CHANGE_TIME, now)
                                    .execute(&self.helper.db, |qb| {
                                        qb.push_where()
                                            .field_eq("file_id", write_handle.file.id);
                                    })
                                    .await
                                {
                                    // DB 更新失败，merge_full 尚未被任何记录追踪，立即清理
                                    if let Err(re) = tokio::fs::remove_file(&merge_full).await {
                                        warn!("complete_upload: remove merge file after merged-status update failure: {}", re);
                                    }
                                    return Err(e.into());
                                }

                                // 清理已合并的chunk文件
                                self.helper
                                    .cleanup_merged_chunks(chunk_ids, self.log_dao.clone());

                                let result = match helper
                                    .complete_file_and_local(
                                        &mut write_handle.file,
                                        &mut write_handle.file_local,
                                        &merge_rel,
                                    )
                                    .await
                                {
                                    Ok(r) => r,
                                    Err(e) => {
                                        // complete_file_and_local 失败，清理孤儿合并文件
                                        if let Err(re) = tokio::fs::remove_file(&merge_full).await {
                                            warn!("complete_upload: remove merge file after complete failure: {}", re);
                                        }
                                        return Err(e);
                                    }
                                };

                                if result.is_some() {
                                    if let Err(e) = tokio::fs::remove_file(&merge_full).await {
                                        warn!(
                                            "complete_upload: remove merge file failed: {}",
                                            e
                                        );
                                    }
                                    self.log_dao
                                        .add(
                                            write_handle.file.id,
                                            0,
                                            write_handle.file.from_user_id,
                                            "complete_upload: merged, duplicate found",
                                            None,
                                        )
                                        .await;
                                }
                            }
                        }
                    }
                    // _merge_guard drops here：锁自动释放
                }
            }
        } else {
            // 单文件完成
            if write_handle.file_local_chunk.is_some() {
                return Err(FileError::Param(fluent_message!("file-chunk-unexpected")));
            }
            let local_path = write_handle.file_local.local_path.clone();
            let local_full = self
                .helper
                .get_full_local_path(&write_handle.file.storage_type, &local_path)
                .await
                .unwrap_or_else(|_| PathBuf::from(&local_path));
            let duplicate = self
                .helper
                .complete_file_and_local(
                    &mut write_handle.file,
                    &mut write_handle.file_local,
                    &local_path,
                )
                .await?;
            // 秒传命中：本次上传的物理文件是重复副本，删除
            if duplicate.is_some() {
                if let Err(e) = tokio::fs::remove_file(&local_full).await {
                    warn!("complete_upload: remove duplicate single file failed: {}", e);
                }
            }
        }

        // 返回最新的文件记录
        let file = self
            .helper
            .find_file_by_id(write_handle.file.id)
            .await?
            .ok_or_else(|| FileError::System(fluent_message!("file-not-found")))?;

        self.file_url_cache.clear(&file.id).await;

        self.logger
            .add(
                &LogFileUpload {
                    action: "complete_upload",
                    user_id: file.from_user_id,
                    file_id: file.id,
                    file_name: &file.origin_name,
                    chunk_count: 0,
                },
                Some(file.id),
                Some(file.from_user_id),
                None,
                env_data,
            )
            .await;

        Ok(file)
    }

    /// 失败文件处理函数 chunk
    pub async fn fail_upload(
        &self,
        mut write_handle: FileWriteHandle,
        env_data: Option<&RequestEnv>,
    ) -> FileResult<()> {
        use tokio::io::AsyncWriteExt;
        if let Err(e) = write_handle.handle.flush().await {
            warn!("fail_upload: flush failed: {}", e);
        }
        // 显式解锁文件后再关闭句柄
        if let Ok(std_file) = write_handle.handle.try_into_std()
            && let Err(e) = std_file.unlock()
        {
            warn!("fail_upload: unlock failed: {}", e);
        }

        let now = now_time()?;

        if write_handle.file_local.file_chunk_total > 1 {
            let chunk = write_handle
                .file_local_chunk
                .ok_or_else(|| FileError::Param(fluent_message!("file-chunk-required")))?;

            // best-effort：即便 DB 更新失败也要清理物理文件
            if let Err(e) = Update::<_, FileLocalChunkModel>::new()
                .set(FileLocalChunkModel::STATUS, FileChunkStatus::Failed as i8)
                .set(FileLocalChunkModel::CHANGE_TIME, now)
                .execute(&self.helper.db, |qb| {
                    qb.push_where().field_eq("id", chunk.id);
                })
                .await
            {
                warn!("fail_upload: update chunk status failed: {}", e);
            }

            if !chunk.chunk_path.is_empty() {
                let full = self
                    .helper
                    .get_full_local_path(&write_handle.file.storage_type, &chunk.chunk_path)
                    .await
                    .unwrap_or_else(|_| PathBuf::from(&chunk.chunk_path));
                if let Err(e) = tokio::fs::remove_file(&full).await {
                    warn!("fail_upload: remove chunk file failed: {}", e);
                }
            }

            self.log_dao
                .add(
                    write_handle.file.id,
                    chunk.id,
                    write_handle.file.from_user_id,
                    "fail_upload: chunk failed",
                    None,
                )
                .await;
        } else {
            if write_handle.file_local_chunk.is_some() {
                return Err(FileError::Param(fluent_message!("file-chunk-unexpected")));
            }

            // best-effort：即便 DB 更新失败也要清理物理文件
            if let Err(e) = Update::<_, FileModel>::new()
                .set(FileModel::STATUS, FileStatus::Failed as i8)
                .set(FileModel::CHANGE_TIME, now)
                .execute(&self.helper.db, |qb| {
                    qb.push_where().field_eq("id", write_handle.file.id);
                })
                .await
            {
                warn!("fail_upload: update file status failed: {}", e);
            }

            if !write_handle.file_local.local_path.is_empty() {
                let full = self
                    .helper
                    .get_full_local_path(
                        &write_handle.file.storage_type,
                        &write_handle.file_local.local_path,
                    )
                    .await
                    .unwrap_or_else(|_| PathBuf::from(&write_handle.file_local.local_path));
                if let Err(e) = tokio::fs::remove_file(&full).await {
                    warn!("fail_upload: remove file failed: {}", e);
                }
            }
        }

        self.logger
            .add(
                &LogFileUpload {
                    action: "fail_upload",
                    user_id: write_handle.file.from_user_id,
                    file_id: write_handle.file.id,
                    file_name: &write_handle.file.origin_name,
                    chunk_count: 0,
                },
                Some(write_handle.file.id),
                Some(write_handle.file.from_user_id),
                None,
                env_data,
            )
            .await;

        Ok(())
    }

    // ==================== 创建方法 2.5: 根据文件 MD5 秒传 ====================

    /// 根据文件 MD5 查找已有文件并创建关联记录（秒传）
    ///
    /// 客户端先计算文件 MD5，调用此方法判断服务端是否已存在相同文件。
    /// 若存在则直接创建 file_ref 关联记录，无需上传文件数据。
    ///
    /// 返回 `Ok(Some(file_ref_id))` 表示秒传成功；
    /// 返回 `Ok(None)` 表示文件不存在，需要走正常上传流程。
    /// - `user_id`: 文件属于的用户ID,0=系统
    /// - `add_user_id`: 文件添加(上传)用户ID
    #[allow(clippy::too_many_arguments)]
    pub async fn create_from_md5(
        &self,
        file_md5: &str,
        user_id: u64,
        add_user_id: u64,
        app_id: u64,
        file_name: &str,
        tag_names: &[&str],
        env_data: Option<&RequestEnv>,
    ) -> FileResult<Option<u64>> {
        if file_md5.is_empty() {
            return Err(FileError::Param(fluent_message!("file-md5-empty")));
        }

        let existing = self
            .helper
            .find_existing_local_file(FileModel::STORAGE_TYPE_LOCAL_PUBLIC, file_md5)
            .await?;

        match existing {
            Some(file) => {
                let file_ref_id = self
                    .file_ops
                    .create_file_ref(&file, user_id, add_user_id, app_id, file_name, env_data)
                    .await?;
                self.file_url_cache.clear(&file.id).await;
                self.tag_dao
                    .batch_add_tags(file.id, user_id, app_id, tag_names, None)
                    .await?;
                Ok(Some(file_ref_id))
            }
            None => Ok(None),
        }
    }
}
