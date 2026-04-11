use std::path::PathBuf;

use fs2::FileExt;
use lsys_core::db::{FieldValue, Insert, QueryBuilderExt, Update};
use lsys_core::fluent_message;
use lsys_core::utils::{RequestEnv, now_time};
use tracing::warn;

use super::file_helpers::ChunkInfo;
use super::logger::*;
use super::*;
use crate::model::*;

/// 写文件句柄包装
pub struct FileWriteHandle {
    pub file: FileModel,
    pub file_local: FileLocalModel,
    pub file_local_chunk: Option<FileLocalChunkModel>,
    pub handle: tokio::fs::File,
    pub app_id: u64,
}

impl FileWriteHandle {
    pub fn into_handle(self) -> tokio::fs::File {
        self.handle
    }
}

impl FileDao {
    /// 创建上传函数
    /// 返回 (file_id, file_user_id)
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
        env_data: Option<&RequestEnv>,
    ) -> FileResult<(u64, u64)> {
        if chunks.is_empty() {
            return Err(FileError::Param(fluent_message!("file-chunks-empty")));
        }

        let now = now_time()?;
        let mut tx = self.helper.db.begin().await?;

        let tx_result: FileResult<(u64, u64)> = async {
            let file_res;
            if chunks.len() == 1 {
                let chunk = &chunks[0];
                let chunk_md5 = chunk.md5.as_deref().unwrap_or("");

                file_res = Insert::<_, FileModel>::new()
                    .set(
                        FileModel::STORAGE_TYPE,
                        storage_type,
                    )
                    .set(FileModel::STATUS, FileStatus::Unfinished as i8)
                    .set(FileModel::FILE_MD5, chunk_md5)
                    .set(FileModel::FILE_SIZE, chunk.len)
                    .set(FileModel::FILE_NAME, file_name)
                    .set(FileModel::CONTENT_TYPE, "")
                    .set(FileModel::MODIFY_TIME, 0u64)
                    .set(FileModel::FROM_USER_ID, add_user_id)
                    .set(FileModel::ADD_TIME, now)
                    .set(FileModel::CHANGE_TIME, 0u64)
                    .set(FileModel::COPY_FILE_ID, 0u64)
                    .execute(&mut *tx)
                    .await?;

                let file_id = file_res.last_insert_id();

                Insert::<_, FileLocalModel>::new()
                    .set(FileLocalModel::FILE_ID, file_id)
                    .set(FileLocalModel::SOURCE_TYPE, FileSourceType::Upload as i8)
                    .set(FileLocalModel::SOURCE_NAME, "")
                    .set(FileLocalModel::FROM_OSS_FILE_ID, 0u64)
                    .set(FileLocalModel::LOCAL_PATH, "")
                    .set(FileLocalModel::FILE_CHUNK_TOTAL, 0u32)
                    .set(FileLocalModel::FILE_CHUNK_SUCC, 0u32)
                    .set(FileLocalModel::FILE_CHUNK_SIZE, 0u64)
                    .set(FileLocalModel::LAST_ERROR, "")
                    .execute(&mut *tx)
                    .await?;

                let fu_res = Insert::<_, FileUserModel>::new()
                    .set(FileUserModel::USER_ID, user_id)
                    .set(FileUserModel::ADD_USER_ID, add_user_id)
                    .set(FileUserModel::APP_ID, app_id)
                    .set(FileUserModel::FILE_ID, file_id)
                    .set(FileUserModel::STATUS, FileUserStatus::Normal as i8)
                    .set(FileUserModel::SOURCE_URL, "")
                    .set(FileUserModel::SOURCE_MD5, "")
                    .set(FileUserModel::ADD_TIME, now)
                    .set(FileUserModel::DELETE_TIME, 0u64)
                    .execute(&mut *tx)
                    .await?;

                let file_user_id = fu_res.last_insert_id();

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

                Ok((file_id, file_user_id))
            } else {
                let total_size: u64 = chunks.iter().map(|c| c.len).sum();

                file_res = Insert::<_, FileModel>::new()
                    .set(
                        FileModel::STORAGE_TYPE,
                        storage_type,
                    )
                    .set(FileModel::STATUS, FileStatus::Unfinished as i8)
                    .set(FileModel::FILE_MD5, "")
                    .set(FileModel::FILE_SIZE, total_size)
                    .set(FileModel::FILE_NAME, file_name)
                    .set(FileModel::CONTENT_TYPE, "")
                    .set(FileModel::MODIFY_TIME, 0u64)
                    .set(FileModel::FROM_USER_ID, add_user_id)
                    .set(FileModel::ADD_TIME, now)
                    .set(FileModel::CHANGE_TIME, 0u64)
                    .set(FileModel::COPY_FILE_ID, 0u64)
                    .execute(&mut *tx)
                    .await?;

                let file_id = file_res.last_insert_id();

                Insert::<_, FileLocalModel>::new()
                    .set(FileLocalModel::FILE_ID, file_id)
                    .set(FileLocalModel::SOURCE_TYPE, FileSourceType::Upload as i8)
                    .set(FileLocalModel::SOURCE_NAME, "")
                    .set(FileLocalModel::FROM_OSS_FILE_ID, 0u64)
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

                let fu_res = Insert::<_, FileUserModel>::new()
                    .set(FileUserModel::USER_ID, user_id)
                    .set(FileUserModel::APP_ID, app_id)
                    .set(FileUserModel::ADD_USER_ID, add_user_id)
                    .set(FileUserModel::FILE_ID, file_id)
                    .set(FileUserModel::STATUS, FileUserStatus::Normal as i8)
                    .set(FileUserModel::SOURCE_URL, "")
                    .set(FileUserModel::SOURCE_MD5, "")
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
                        "create_upload: file created",
                        Some(&mut tx),
                    )
                    .await;

                self.tag_dao
                    .batch_add_tags(file_id, user_id, app_id, tag_names, Some(&mut tx))
                    .await?;

                Ok((file_id, file_user_id))
            }
        }
        .await;

        let (file_id, file_user_id) = match tx_result {
            Ok(id) => {
                tx.commit().await?;
                id
            }
            Err(e) => {
                let _ = tx.rollback().await;
                return Err(e);
            }
        };

        self.logger
            .add(
                &LogFileUpload {
                    action: "create_upload",
                    user_id,
                    file_id,
                    file_name,
                    chunk_count: chunks.len(),
                },
                Some(file_id),
                Some(user_id),
                None,
                env_data,
            )
            .await;

        Ok((file_id, file_user_id))
    }

    /// 通过 file_user_id 获取上传文件写句柄
    /// 前端传入 file_user_id 而非 file_id, 内部解析 file_user → file → get_upload_handle
    /// app_id 从 file_user 记录中获取, 无需外部传入
    pub async fn get_upload_handle_by_file_user_id(
        &self,
        file_user_id: u64,
        chunk_index: u32,
    ) -> FileResult<FileWriteHandle> {
        let file_user = self
            .helper
            .find_file_user_by_id(file_user_id)
            .await?
            .ok_or_else(|| FileError::Param(fluent_message!("file-user-not-found")))?;

        let file = self
            .helper
            .find_file_by_id(file_user.file_id)
            .await?
            .ok_or_else(|| FileError::Param(fluent_message!("file-not-found")))?;

        // 校验文件上传用户 (from_user_id 是上传者, add_user_id 是添加记录的用户)
        if file.from_user_id != file_user.add_user_id {
            return Err(FileError::Param(fluent_message!("file-user-mismatch")));
        }

        self.get_upload_handle(&file, chunk_index, file_user.app_id)
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
                    let full = self.helper.get_full_local_path(&file.storage_type, &chunk.chunk_path).await
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
                    let chunk_ext = crate::common::extract_extension(Some(&file.file_name));
                    let prefix = format!("{}_{}_chunk{}", app_id, file.from_user_id, chunk_index);
                    let (rel, full) = self.helper.create_new_file(&file.storage_type, &prefix, chunk_ext).await?;
                    let f = tokio::fs::OpenOptions::new()
                        .write(true)
                        .open(&full)
                        .await?;

                    // 保存路径
                    chunk.chunk_path = rel.clone();
                    Update::<_, FileLocalChunkModel>::new()
                        .set(FileLocalChunkModel::CHUNK_PATH, &rel)
                        .execute(&self.helper.db, |qb| {
                            qb.push_where().field_eq("id", chunk.id);
                        })
                        .await?;
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
                })
            } else {
                // 单文件
                let (handle, _rel_path) = if !file_local.local_path.is_empty() {
                    let full = self.helper.get_full_local_path(&file.storage_type, &file_local.local_path).await
                        .unwrap_or_else(|_| PathBuf::from(&file_local.local_path));
                    let f = tokio::fs::OpenOptions::new()
                        .write(true)
                        .create(true)
                        .truncate(false)
                        .open(&full)
                        .await?;
                    (f, file_local.local_path.clone())
                } else {
                    let upload_ext = crate::common::extract_extension(Some(&file.file_name));
                    let prefix = format!("{}_{}_upload", app_id, file.from_user_id);
                    let (rel, full) = self.helper.create_new_file(&file.storage_type, &prefix, upload_ext).await?;
                    let f = tokio::fs::OpenOptions::new()
                        .write(true)
                        .open(&full)
                        .await?;

                    file_local.local_path = rel.clone();
                    Update::<_, FileLocalModel>::new()
                        .set(FileLocalModel::LOCAL_PATH, &rel)
                        .execute(&self.helper.db, |qb| {
                            qb.push_where().field_eq("id", file_local.id);
                        })
                        .await?;
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
            chunk.complete_size += data.len() as u64;

            // 立即同步已写入数据量到数据库
            Update::<_, FileLocalChunkModel>::new()
                .set(FileLocalChunkModel::COMPLETE_SIZE, chunk.complete_size)
                .execute(&self.helper.db, |qb| {
                    qb.push_where().field_eq("id", chunk.id);
                })
                .await?;
        } else {
            if write_handle.file_local_chunk.is_some() {
                return Err(FileError::Param(fluent_message!("file-chunk-unexpected")));
            }
            write_handle.file_local.file_chunk_size += data.len() as u64;

            // 立即同步已写入数据量到数据库
            Update::<_, FileLocalModel>::new()
                .set(
                    FileLocalModel::FILE_CHUNK_SIZE,
                    write_handle.file_local.file_chunk_size,
                )
                .execute(&self.helper.db, |qb| {
                    qb.push_where().field_eq("id", write_handle.file_local.id);
                })
                .await?;
        }

        write_handle.handle.write_all(data).await?;
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

            let chunk_full = self.helper.get_full_local_path(&write_handle.file.storage_type, &chunk.chunk_path).await
                .unwrap_or_else(|_| PathBuf::from(&chunk.chunk_path));
            let metadata = tokio::fs::metadata(&chunk_full).await?;
            let chunk_data = tokio::fs::read(&chunk_full).await?;
            let chunk_md5 = format!("{:x}", md5::compute(&chunk_data));

            let actual_size = metadata.len();
            let status = if !chunk.upload_md5.is_empty() && chunk_md5 != chunk.upload_md5 {
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
                // 合并文件
                let merge_ext =
                    crate::common::extract_extension(Some(&write_handle.file.file_name));
                let merge_prefix = format!(
                    "{}_{}_merge",
                    write_handle.app_id, write_handle.file.from_user_id
                );
                let (merge_rel, merge_full) =
                    helper.create_new_file(&write_handle.file.storage_type, &merge_prefix, merge_ext).await?;
                match helper.merge_chunk_files(&write_handle.file.storage_type, &all_chunks, &merge_full).await {
                    Err(e) => {
                        // 合并失败
                        write_handle.file_local.local_path = merge_rel.clone();
                        write_handle.file.status = FileStatus::Failed as i8;
                        write_handle.file.change_time = now;

                        Update::<_, FileLocalModel>::new()
                            .set(FileLocalModel::LOCAL_PATH, &merge_rel)
                            .execute(&self.helper.db, |qb| {
                                qb.push_where().field_eq("id", write_handle.file_local.id);
                            })
                            .await?;

                        Update::<_, FileModel>::new()
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
                        let chunk_ids: Vec<u64> = all_chunks.iter().map(|c| c.id).collect();
                        Update::<_, FileLocalChunkModel>::new()
                            .set(FileLocalChunkModel::STATUS, FileChunkStatus::Merged as i8)
                            .set(FileLocalChunkModel::CHANGE_TIME, now)
                            .execute(&self.helper.db, |qb| {
                                qb.push_where().field_eq("file_id", write_handle.file.id);
                            })
                            .await?;

                        // 清理已合并的chunk文件
                        self.helper
                            .cleanup_merged_chunks(chunk_ids, self.log_dao.clone());

                        // 辅助函数.2
                        let result = helper
                            .complete_file_and_local(
                                &mut write_handle.file,
                                &mut write_handle.file_local,
                                &merge_rel,
                            )
                            .await?;

                        if result.is_some() {
                            if let Err(e) = tokio::fs::remove_file(&merge_full).await {
                                warn!("complete_upload: remove merge file failed: {}", e);
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
        } else {
            // 单文件完成
            if write_handle.file_local_chunk.is_some() {
                return Err(FileError::Param(fluent_message!("file-chunk-unexpected")));
            }
            let local_path = write_handle.file_local.local_path.clone();
            self.helper
                .complete_file_and_local(
                    &mut write_handle.file,
                    &mut write_handle.file_local,
                    &local_path,
                )
                .await?;
        }

        // 返回最新的文件记录
        let file = self
            .helper
            .find_file_by_id(write_handle.file.id)
            .await?
            .ok_or_else(|| FileError::System(fluent_message!("file-not-found")))?;

        self.logger
            .add(
                &LogFileUpload {
                    action: "complete_upload",
                    user_id: file.from_user_id,
                    file_id: file.id,
                    file_name: &file.file_name,
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
        let _ = write_handle.handle.flush().await;
        // 显式解锁文件后再关闭句柄
        if let Ok(std_file) = write_handle.handle.try_into_std() {
            let _ = std_file.unlock();
        }

        let now = now_time()?;

        if write_handle.file_local.file_chunk_total > 1 {
            let chunk = write_handle
                .file_local_chunk
                .ok_or_else(|| FileError::Param(fluent_message!("file-chunk-required")))?;

            Update::<_, FileLocalChunkModel>::new()
                .set(FileLocalChunkModel::STATUS, FileChunkStatus::Failed as i8)
                .set(FileLocalChunkModel::CHANGE_TIME, now)
                .execute(&self.helper.db, |qb| {
                    qb.push_where().field_eq("id", chunk.id);
                })
                .await?;

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

            Update::<_, FileModel>::new()
                .set(FileModel::STATUS, FileStatus::Failed as i8)
                .set(FileModel::CHANGE_TIME, now)
                .execute(&self.helper.db, |qb| {
                    qb.push_where().field_eq("id", write_handle.file.id);
                })
                .await?;
        }

        self.logger
            .add(
                &LogFileUpload {
                    action: "fail_upload",
                    user_id: write_handle.file.from_user_id,
                    file_id: write_handle.file.id,
                    file_name: &write_handle.file.file_name,
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
    /// 若存在则直接创建 file_user 关联记录，无需上传文件数据。
    ///
    /// 返回 `Ok(Some(file_user_id))` 表示秒传成功；
    /// 返回 `Ok(None)` 表示文件不存在，需要走正常上传流程。
    /// - `user_id`: 文件属于的用户ID,0=系统
    /// - `add_user_id`: 文件添加(上传)用户ID
    pub async fn create_from_md5(
        &self,
        file_md5: &str,
        user_id: u64,
        add_user_id: u64,
        app_id: u64,
        tag_names: &[&str],
        env_data: Option<&RequestEnv>,
    ) -> FileResult<Option<u64>> {
        if file_md5.is_empty() {
            return Err(FileError::Param(fluent_message!("file-md5-empty")));
        }

        let existing = self
            .helper
            .find_existing_file(FileModel::STORAGE_TYPE_LOCAL_PUBLIC, file_md5)
            .await?;

        match existing {
            Some(file) => {
                let file_user_id = self
                    .create_file_user(&file, user_id, add_user_id, app_id, env_data)
                    .await?;
                self.tag_dao
                    .batch_add_tags(file.id, user_id, app_id, tag_names, None)
                    .await?;
                Ok(Some(file_user_id))
            }
            None => Ok(None),
        }
    }
}
