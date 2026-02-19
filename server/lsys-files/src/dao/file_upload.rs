use fs2::FileExt;
use lsys_core::db::{Insert, SqlExpr, SqlQuote, SqlSuffix, Update};
use lsys_core::sql_format;
use lsys_core::{fluent_message, now_time, RequestEnv};
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
}

impl FileWriteHandle {
    pub fn into_handle(self) -> tokio::fs::File {
        self.handle
    }
}

impl FileDao {
    /// 创建上传函数
    pub async fn create_upload(
        &self,
        user_id: u64,
        app_id: u64,
        chunks: &[ChunkInfo],
        file_name: &str,
        env_data: Option<&RequestEnv>,
    ) -> FileResult<FileModel> {
        if chunks.is_empty() {
            return Err(FileError::Param(fluent_message!("file-chunks-empty")));
        }

        let now = now_time()?;
        let mut tx = self.db().begin().await?;

        let tx_result: FileResult<u64> = async {
            let file_res;
            if chunks.len() == 1 {
                let chunk = &chunks[0];
                let chunk_md5 = chunk.md5.as_deref().unwrap_or("");

                file_res = Insert::<FileModel>::new()
                    .set(FileModel::STORAGE_TYPE, FileModel::STORAGE_TYPE_LOCAL)
                    .set(FileModel::STATUS, FileStatus::Unfinished as i8)
                    .set(FileModel::FILE_MD5, chunk_md5)
                    .set(FileModel::FILE_SIZE, chunk.len)
                    .set(FileModel::FILE_NAME, file_name)
                    .set(FileModel::CONTENT_TYPE, "")
                    .set(FileModel::MODIFY_TIME, 0u64)
                    .set(FileModel::FROM_USER_ID, user_id)
                    .set(FileModel::ADD_TIME, now)
                    .set(FileModel::CHANGE_TIME, 0u64)
                    .set(FileModel::COPY_FILE_ID, 0u64)
                    .execute(&mut *tx)
                    .await?;

                let file_id = file_res.last_insert_id();

                Insert::<FileLocalModel>::new()
                    .set(FileLocalModel::FILE_ID, file_id)
                    .set(FileLocalModel::SOURCE_TYPE, FileSourceType::Upload as i8)
                    .set(FileLocalModel::SOURCE_NAME, "")
                    .set(FileLocalModel::OSS_FILE_ID, 0u64)
                    .set(FileLocalModel::LOCAL_PATH, "")
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
            } else {
                let total_size: u64 = chunks.iter().map(|c| c.len).sum();

                file_res = Insert::<FileModel>::new()
                    .set(FileModel::STORAGE_TYPE, FileModel::STORAGE_TYPE_LOCAL)
                    .set(FileModel::STATUS, FileStatus::Unfinished as i8)
                    .set(FileModel::FILE_MD5, "")
                    .set(FileModel::FILE_SIZE, total_size)
                    .set(FileModel::FILE_NAME, file_name)
                    .set(FileModel::CONTENT_TYPE, "")
                    .set(FileModel::MODIFY_TIME, 0u64)
                    .set(FileModel::FROM_USER_ID, user_id)
                    .set(FileModel::ADD_TIME, now)
                    .set(FileModel::CHANGE_TIME, 0u64)
                    .set(FileModel::COPY_FILE_ID, 0u64)
                    .execute(&mut *tx)
                    .await?;

                let file_id = file_res.last_insert_id();

                Insert::<FileLocalModel>::new()
                    .set(FileLocalModel::FILE_ID, file_id)
                    .set(FileLocalModel::SOURCE_TYPE, FileSourceType::Upload as i8)
                    .set(FileLocalModel::SOURCE_NAME, "")
                    .set(FileLocalModel::OSS_FILE_ID, 0u64)
                    .set(FileLocalModel::LOCAL_PATH, "")
                    .set(FileLocalModel::FILE_CHUNK_TOTAL, chunks.len() as u32)
                    .set(FileLocalModel::FILE_CHUNK_SUCC, 0u32)
                    .set(FileLocalModel::FILE_CHUNK_SIZE, 0u64)
                    .set(FileLocalModel::LAST_ERROR, "")
                    .execute(&mut *tx)
                    .await?;

                for (idx, chunk) in chunks.iter().enumerate() {
                    Insert::<FileLocalChunkModel>::new()
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
            }

            self.log_dao()
                .add(
                    file_res.last_insert_id(),
                    0,
                    user_id,
                    "create_upload: file created",
                    Some(&mut tx),
                )
                .await;

            Ok(file_res.last_insert_id())
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
            .helper()
            .find_file_by_id(file_id)
            .await?
            .ok_or_else(|| FileError::System(fluent_message!("file-create-error")))?;

        self.logger()
            .add(
                &LogFileUpload {
                    action: "create_upload",
                    user_id,
                    file_id: file.id,
                    file_name: &file.file_name,
                    chunk_count: chunks.len(),
                },
                Some(file.id),
                Some(user_id),
                None,
                env_data,
            )
            .await;

        Ok(file)
    }

    /// 获取上传文件写句柄
    pub async fn get_upload_handle(
        &self,
        file: &FileModel,
        chunk_index: u32,
    ) -> FileResult<FileWriteHandle> {
        let mut file_local = self
            .helper()
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
                .helper()
                .find_chunk_by_file_and_index(file.id, chunk_index)
                .await?
                .ok_or_else(|| FileError::Param(fluent_message!("file-chunk-not-found")))?;

            let (handle, _rel_path) = if !chunk.chunk_path.is_empty() {
                // 已存在路径, 获取写锁
                let full = self.helper().get_full_local_path(&chunk.chunk_path);
                let f = tokio::fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(false)
                    .open(&full)
                    .await?;
                (f, chunk.chunk_path.clone())
            } else {
                // 新增文件
                let chunk_ext =
                    crate::dao::file_helpers::FileHelper::extract_extension(&file.file_name);
                let (rel, full) = self
                    .helper()
                    .create_new_file(&format!("chunk.{}", chunk_ext))
                    .await?;
                let f = tokio::fs::OpenOptions::new()
                    .write(true)
                    .open(&full)
                    .await?;

                // 保存路径
                chunk.chunk_path = rel.clone();
                Update::<FileLocalChunkModel>::new()
                    .set(FileLocalChunkModel::CHUNK_PATH, &rel)
                    .execute(SqlSuffix::Where(&sql_format!("id={}", chunk.id)), self.db())
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
            })
        } else {
            // 单文件
            let (handle, _rel_path) = if !file_local.local_path.is_empty() {
                let full = self.helper().get_full_local_path(&file_local.local_path);
                let f = tokio::fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(false)
                    .open(&full)
                    .await?;
                (f, file_local.local_path.clone())
            } else {
                let upload_ext =
                    crate::dao::file_helpers::FileHelper::extract_extension(&file.file_name);
                let (rel, full) = self
                    .helper()
                    .create_new_file(&format!("upload.{}", upload_ext))
                    .await?;
                let f = tokio::fs::OpenOptions::new()
                    .write(true)
                    .open(&full)
                    .await?;

                file_local.local_path = rel.clone();
                Update::<FileLocalModel>::new()
                    .set(FileLocalModel::LOCAL_PATH, &rel)
                    .execute(
                        SqlSuffix::Where(&sql_format!("id={}", file_local.id)),
                        self.db(),
                    )
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
            })
        }
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
            Update::<FileLocalChunkModel>::new()
                .set(FileLocalChunkModel::COMPLETE_SIZE, chunk.complete_size)
                .execute(SqlSuffix::Where(&sql_format!("id={}", chunk.id)), self.db())
                .await?;
        } else {
            if write_handle.file_local_chunk.is_some() {
                return Err(FileError::Param(fluent_message!("file-chunk-unexpected")));
            }
            write_handle.file_local.file_chunk_size += data.len() as u64;

            // 立即同步已写入数据量到数据库
            Update::<FileLocalModel>::new()
                .set(
                    FileLocalModel::FILE_CHUNK_SIZE,
                    write_handle.file_local.file_chunk_size,
                )
                .execute(
                    SqlSuffix::Where(&sql_format!("id={}", write_handle.file_local.id)),
                    self.db(),
                )
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

            let chunk_full = self.helper().get_full_local_path(&chunk.chunk_path);
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

            Update::<FileLocalChunkModel>::new()
                .set(FileLocalChunkModel::FILE_SIZE, actual_size)
                .set(FileLocalChunkModel::CHUNK_MD5, &chunk_md5)
                .set(FileLocalChunkModel::STATUS, status)
                .set(FileLocalChunkModel::COMPLETE_SIZE, actual_size)
                .set(FileLocalChunkModel::CHANGE_TIME, now)
                .execute(SqlSuffix::Where(&sql_format!("id={}", chunk.id)), self.db())
                .await?;

            // file_local: file_chunk_succ+1, file_chunk_size+=actual_size (原子操作)
            Update::<FileLocalModel>::new()
                .set(
                    FileLocalModel::FILE_CHUNK_SUCC,
                    SqlExpr("file_chunk_succ + 1"),
                )
                .set(
                    FileLocalModel::FILE_CHUNK_SIZE,
                    SqlExpr(format!("file_chunk_size + {}", actual_size)),
                )
                .execute(
                    SqlSuffix::Where(&sql_format!("id={}", write_handle.file_local.id)),
                    self.db(),
                )
                .await?;

            self.log_dao()
                .add(
                    write_handle.file.id,
                    chunk.id,
                    write_handle.file.from_user_id,
                    &format!("complete_upload: chunk {} done", chunk.chunk_index),
                    None,
                )
                .await;

            // 检查所有 chunk 是否都=1(正常)
            let helper = self.helper();
            let all_chunks = helper.find_chunks_by_file_id(write_handle.file.id).await?;
            let all_normal = all_chunks
                .iter()
                .all(|c| FileChunkStatus::Normal.eq(c.status));

            if all_normal {
                // 合并文件
                let merge_ext = crate::dao::file_helpers::FileHelper::extract_extension(
                    &write_handle.file.file_name,
                );
                let (merge_rel, merge_full) = helper
                    .create_new_file(&format!("merged.{}", merge_ext))
                    .await?;
                match helper.merge_chunk_files(&all_chunks, &merge_full).await {
                    Err(e) => {
                        // 合并失败
                        write_handle.file_local.local_path = merge_rel.clone();
                        write_handle.file.status = FileStatus::Failed.to();
                        write_handle.file.change_time = now;

                        Update::<FileLocalModel>::new()
                            .set(FileLocalModel::LOCAL_PATH, &merge_rel)
                            .execute(
                                SqlSuffix::Where(&sql_format!("id={}", write_handle.file_local.id)),
                                self.db(),
                            )
                            .await?;

                        Update::<FileModel>::new()
                            .set(FileModel::STATUS, FileStatus::Failed as i8)
                            .set(FileModel::CHANGE_TIME, now)
                            .execute(
                                SqlSuffix::Where(&sql_format!("id={}", write_handle.file.id)),
                                self.db(),
                            )
                            .await?;

                        self.log_dao()
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
                        Update::<FileLocalChunkModel>::new()
                            .set(FileLocalChunkModel::STATUS, FileChunkStatus::Merged as i8)
                            .set(FileLocalChunkModel::CHANGE_TIME, now)
                            .execute(
                                SqlSuffix::Where(&sql_format!("file_id={}", write_handle.file.id)),
                                self.db(),
                            )
                            .await?;

                        // 清理已合并的chunk文件
                        self.helper().cleanup_merged_chunks(chunk_ids);

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
                            self.log_dao()
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
            self.helper()
                .complete_file_and_local(
                    &mut write_handle.file,
                    &mut write_handle.file_local,
                    &local_path,
                )
                .await?;
        }

        // 返回最新的文件记录
        let file = self
            .helper()
            .find_file_by_id(write_handle.file.id)
            .await?
            .ok_or_else(|| FileError::System(fluent_message!("file-not-found")))?;

        self.logger()
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

            Update::<FileLocalChunkModel>::new()
                .set(FileLocalChunkModel::STATUS, FileChunkStatus::Failed as i8)
                .set(FileLocalChunkModel::CHANGE_TIME, now)
                .execute(SqlSuffix::Where(&sql_format!("id={}", chunk.id)), self.db())
                .await?;

            self.log_dao()
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

            Update::<FileModel>::new()
                .set(FileModel::STATUS, FileStatus::Failed as i8)
                .set(FileModel::CHANGE_TIME, now)
                .execute(
                    SqlSuffix::Where(&sql_format!("id={}", write_handle.file.id)),
                    self.db(),
                )
                .await?;
        }

        self.logger()
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
}
