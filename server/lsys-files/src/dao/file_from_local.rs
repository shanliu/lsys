use std::path::PathBuf;

use lsys_core::db::Insert;
use lsys_core::fluent_message;
use lsys_core::utils::{now_time, RequestEnv};
use tokio::fs;
use tracing::warn;

use super::logger::*;
use super::*;
use crate::model::*;

/// 本地文件导入模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalFileMode {
    /// 移动源文件到存储目录（源文件将被删除）
    Move,
    /// 拷贝源文件到存储目录（保留源文件）
    Copy,
}

impl FileDao {
    // ==================== 创建方法 4: 已知本地文件生成 ====================
    ///
    /// - `user_id`: 文件属于的用户ID,0=系统
    /// - `add_user_id`: 文件添加(上传)用户ID
    #[allow(clippy::too_many_arguments)]
    pub async fn create_from_local_file(
        &self,
        local_file_path: &str,
        user_id: u64,
        add_user_id: u64,
        app_id: u64,
        file_name: Option<&str>,
        mode: LocalFileMode,
        copy_file_id: Option<u64>,
        tag_names: &[&str],
        env_data: Option<&RequestEnv>,
    ) -> FileResult<(FileModel, FileUserModel)> {
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
                .find_existing_file(FileModel::STORAGE_TYPE_LOCAL_PUBLIC, &file_md5)
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
                    if let Some(fu) = self
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
                        self.log_dao
                            .add(
                                existing.id,
                                0,
                                user_id,
                                "create_from_local: existing file+user, deleted source",
                                None,
                            )
                            .await;
                        self.tag_dao
                            .batch_add_tags(existing.id, user_id, app_id, tag_names, None)
                            .await?;
                        return Ok((existing, fu));
                    }

                    // 创建 file_user
                    info!("create_from_local_file: creating file_user link to existing file");
                    let mut tx = self.helper.db.begin().await?;
                    let tx_result: FileResult<()> = async {
                        Insert::<_, FileUserModel>::new()
                            .set(FileUserModel::USER_ID, user_id)
                            .set(FileUserModel::ADD_USER_ID, add_user_id)
                            .set(FileUserModel::APP_ID, app_id)
                            .set(FileUserModel::FILE_ID, existing.id)
                            .set(FileUserModel::STATUS, FileUserStatus::Normal as i8)
                            .set(FileUserModel::SOURCE_URL, "")
                            .set(FileUserModel::SOURCE_MD5, "")
                            .set(FileUserModel::ADD_TIME, now)
                            .set(FileUserModel::DELETE_TIME, 0u64)
                            .execute(&mut *tx)
                            .await?;
                        self.log_dao
                            .add(
                                existing.id,
                                0,
                                user_id,
                                "create_from_local: existing file, new file_user",
                                Some(&mut tx),
                            )
                            .await;
                        self.tag_dao
                            .batch_add_tags(existing.id, user_id, app_id, tag_names, Some(&mut tx))
                            .await?;
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

                    if mode == LocalFileMode::Move
                        && let Err(e) = fs::remove_file(local_file_path).await {
                            warn!("create_from_local: remove source file failed: {}", e);
                        }
                    // 查询刚创建的 file_user
                    let file_user = self
                        .helper
                        .find_file_user(user_id, app_id, existing.id, FileUserStatus::Normal)
                        .await?
                        .ok_or_else(|| FileError::System(fluent_message!("file-user-create-error")))?;
                    return Ok((existing, file_user));
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

        let prefix = format!("{}_{}_local", app_id, user_id);
        let relative_path = match mode {
            LocalFileMode::Move => {
                self.helper
                    .move_file_to_storage(local_file_path, &prefix, Some(actual_name))
                    .await?
            }
            LocalFileMode::Copy => {
                self.helper
                    .copy_file_to_storage(local_file_path, &prefix, Some(actual_name))
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

        let mut tx = self.helper.db.begin().await?;

        let tx_result: FileResult<u64> = async {
            let file_res = Insert::<_, FileModel>::new()
                .set(FileModel::STORAGE_TYPE, FileModel::STORAGE_TYPE_LOCAL_PUBLIC)
                .set(FileModel::STATUS, FileStatus::Normal as i8)
                .set(FileModel::FILE_MD5, &file_md5)
                .set(FileModel::FILE_SIZE, metadata.len())
                .set(FileModel::FILE_NAME, actual_name)
                .set(FileModel::CONTENT_TYPE, &content_type)
                .set(FileModel::MODIFY_TIME, modify_time)
                .set(FileModel::FROM_USER_ID, add_user_id)
                .set(FileModel::ADD_TIME, now)
                .set(FileModel::CHANGE_TIME, 0u64)
                .set(FileModel::COPY_FILE_ID, copy_file_id.unwrap_or(0))
                .execute(&mut *tx)
                .await?;

            let file_id = file_res.last_insert_id();

            Insert::<_, FileLocalModel>::new()
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

            Insert::<_, FileUserModel>::new()
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

            self.log_dao
                .add(
                    file_id,
                    0,
                    add_user_id,
                    "create_from_local: new file created",
                    Some(&mut tx),
                )
                .await;

            self.tag_dao
                .batch_add_tags(file_id, user_id, app_id, tag_names, Some(&mut tx))
                .await?;

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
                let bp = self.helper.config.storage_base_path.clone();
                let full = std::path::Path::new(&bp).join(&rp);
                if let Err(e) = tokio::fs::remove_file(&full).await {
                    tracing::warn!("create_from_local: rollback remove file failed: {}", e);
                }
                return Err(e);
            }
        };

        let file = self
            .helper
            .find_file_by_id(file_id)
            .await?
            .ok_or_else(|| FileError::System(fluent_message!("file-create-error")))?;

        let file_user = self
            .helper
            .find_file_user(user_id, app_id, file_id, FileUserStatus::Normal)
            .await?
            .ok_or_else(|| FileError::System(fluent_message!("file-user-create-error")))?;

        self.logger
            .add(
                &LogFileCreate {
                    action: "create_from_local_file",
                    storage_type: FileModel::STORAGE_TYPE_LOCAL_PUBLIC,
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

        Ok((file, file_user))
    }

    // ==================== 操作方法 5: 拷贝函数 ====================
    /// 拷贝用户文件
    ///
    /// 通过 `FileOpContext` 传入参数：
    /// - 必须: `file_user`
    /// - 可选: `file`（未提供时自动查询）
    /// - 可选: `oss_provider`（OSS 文件需要先同步到本地时使用）
    pub async fn copy_file(
        &self,
        ctx: FileOpContext<'_>,
        env_data: Option<&RequestEnv>,
    ) -> FileResult<(FileModel, FileUserModel)> {
        let file_user = ctx.file_user;
        let file = ctx.file().await?;

        if !FileStatus::Normal.eq(file.status) {
            return Err(FileError::Param(fluent_message!(
                "file-status-must-be-normal"
            )));
        }

        let user_id = file_user.user_id;
        let add_user_id = file_user.add_user_id;
        let app_id = file_user.app_id;

        if file.is_local() {
            // 拷贝本地文件
            let local = self
                .helper
                .find_file_local_by_file_id(file.id)
                .await?
                .ok_or_else(|| FileError::Param(fluent_message!("file-local-not-found")))?;

            let src_full = self.helper.get_full_local_path(&local.local_path);
            let copy_prefix = format!("{}_{}_copy", app_id, user_id);
            let copy_ext = crate::common::extract_extension(Some(&file.file_name));
            let (_rel, dst_full) = self.helper.create_new_file(&copy_prefix, copy_ext).await?;
            fs::copy(&src_full, &dst_full).await?;

            // 拷贝源文件的标签
            let source_tags = self.data_dao.get_file_tag_names(file.id).await?;
            let source_tag_refs: Vec<&str> = source_tags.iter().map(String::as_str).collect();

            let (new_file, new_file_user) = self
                .create_from_local_file(
                    dst_full.to_str().unwrap_or(""),
                    user_id,
                    add_user_id,
                    app_id,
                    Some(&file.file_name),
                    LocalFileMode::Move,
                    Some(file.id),
                    &source_tag_refs,
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
                    Some(add_user_id),
                    None,
                    env_data,
                )
                .await;

            Ok((new_file, new_file_user))
        } else {
            // OSS类型: 先同步到本地，再对本地文件进行物理拷贝，确保副本独立
            let sync_ctx = FileOpContext::new(file_user, &self.helper, &self.oss_config).with_file(file)?;
            let (local_file, local_file_user) = self
                .sync_oss_to_local(sync_ctx, env_data)
                .await?;

            // 对本地文件进行物理拷贝，创建独立副本
            let copy_ctx = FileOpContext::new(&local_file_user, &self.helper, &self.oss_config).with_file(&local_file)?;
            Box::pin(self.copy_file(copy_ctx, env_data)).await
        }
    }
}
