use std::path::PathBuf;

use lsys_core::db::{Insert, TableMeta};
use lsys_core::fluent_message;
use lsys_core::utils::{RequestEnv, now_time};
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
    /// - `storage_type`: 存储类型（STORAGE_TYPE_LOCAL_PUBLIC/PRIVATE/CRYPTO）
    #[allow(clippy::too_many_arguments)]
    pub async fn create_from_local_file(
        &self,
        local_file_path: &str,
        user_id: u64,
        add_user_id: u64,
        app_id: u64,
        storage_type: &str,
        file_name: Option<&str>,
        mode: LocalFileMode,
        copy_file_id: Option<u64>,
        from_oss_file_id: Option<u64>,
        tag_names: &[&str],
        env_data: Option<&RequestEnv>,
    ) -> FileResult<(FileModel, FileUserModel)> {
        use tracing::info;

        // 验证存储类型
        if !FileModel::is_local_key(storage_type) {
            return Err(FileError::Param(fluent_message!(
                "file-invalid-storage-type",
                {"storage_type": storage_type}
            )));
        }

        info!(
            "create_from_local_file: starting, user_id={}, path={}, storage_type={}",
            user_id, local_file_path, storage_type
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
                .find_existing_file(storage_type, &file_md5)
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
                        let full = self.helper.get_full_local_path(&existing.storage_type, &local_rec.local_path).await?;
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
                        && let Err(e) = fs::remove_file(local_file_path).await
                    {
                        warn!("create_from_local: remove source file failed: {}", e);
                    }
                    // 查询刚创建的 file_user
                    let file_user = self
                        .helper
                        .find_file_user(user_id, app_id, existing.id, FileUserStatus::Normal)
                        .await?
                        .ok_or_else(|| {
                            FileError::System(fluent_message!("file-user-create-error"))
                        })?;
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
                    .move_file_to_storage(storage_type, local_file_path, &prefix, Some(actual_name))
                    .await?
            }
            LocalFileMode::Copy => {
                self.helper
                    .copy_file_to_storage(storage_type, local_file_path, &prefix, Some(actual_name))
                    .await?
            }
        };

        let full_path = self.helper.get_full_local_path(storage_type, &relative_path).await?;
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
                .set(FileModel::STORAGE_TYPE, storage_type)
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

            // 如果是从另一个文件拷贝，需要继承关联关系
            let (public_id, private_id, crypto_id) = if let Some(source_id) = copy_file_id {
                if let Some(source_local) = self.helper.find_file_local_by_file_id(source_id).await? {
                    (source_local.public_file_id, source_local.private_file_id, source_local.crypto_file_id)
                } else {
                    (0, 0, 0)
                }
            } else {
                (0, 0, 0)
            };

            Insert::<_, FileLocalModel>::new()
                .set(FileLocalModel::FILE_ID, file_id)
                .set(FileLocalModel::SOURCE_TYPE, FileSourceType::LocalPath as i8)
                .set(FileLocalModel::SOURCE_NAME, local_file_path)
                .set(FileLocalModel::FROM_OSS_FILE_ID, from_oss_file_id.unwrap_or(0))
                .set(FileLocalModel::LOCAL_PATH, &relative_path)
                .set(FileLocalModel::FILE_CHUNK_TOTAL, 0u32)
                .set(FileLocalModel::FILE_CHUNK_SUCC, 0u32)
                .set(FileLocalModel::FILE_CHUNK_SIZE, 0u64)
                .set(FileLocalModel::PUBLIC_FILE_ID, public_id)
                .set(FileLocalModel::PRIVATE_FILE_ID, private_id)
                .set(FileLocalModel::CRYPTO_FILE_ID, crypto_id)
                .set(FileLocalModel::LAST_ERROR, "")
                .execute(&mut *tx)
                .await?;

            // 如果是拷贝操作，需要更新相关文件的关联字段
            if let Some(source_id) = copy_file_id {
                // 更新源文件指向新文件
                if self.helper.find_file_by_id(source_id).await?.is_some() {
                    let update_field = match storage_type {
                        FileModel::STORAGE_TYPE_LOCAL_PUBLIC => Some(FileLocalModel::PUBLIC_FILE_ID),
                        FileModel::STORAGE_TYPE_LOCAL_PRIVATE => Some(FileLocalModel::PRIVATE_FILE_ID),
                        FileModel::STORAGE_TYPE_LOCAL_CRYPTO => Some(FileLocalModel::CRYPTO_FILE_ID),
                        _ => None,
                    };
                    if let Some(field) = update_field {
                        sqlx::query(&format!(
                            "UPDATE {} SET {} = ? WHERE file_id = ?",
                            FileLocalModel::table_name(),
                            field
                        ))
                        .bind(file_id)
                        .bind(source_id)
                        .execute(&mut *tx)
                        .await?;
                    }

                    // 更新所有相关文件指向新文件
                    if public_id > 0 {
                        let update_field_for_public = match storage_type {
                            FileModel::STORAGE_TYPE_LOCAL_PRIVATE => Some(FileLocalModel::PRIVATE_FILE_ID),
                            FileModel::STORAGE_TYPE_LOCAL_CRYPTO => Some(FileLocalModel::CRYPTO_FILE_ID),
                            _ => None,
                        };
                        if let Some(field) = update_field_for_public {
                            sqlx::query(&format!(
                                "UPDATE {} SET {} = ? WHERE file_id = ?",
                                FileLocalModel::table_name(),
                                field
                            ))
                            .bind(file_id)
                            .bind(public_id)
                            .execute(&mut *tx)
                            .await?;
                        }
                    }

                    if private_id > 0 {
                        let update_field_for_private = match storage_type {
                            FileModel::STORAGE_TYPE_LOCAL_PUBLIC => Some(FileLocalModel::PUBLIC_FILE_ID),
                            FileModel::STORAGE_TYPE_LOCAL_CRYPTO => Some(FileLocalModel::CRYPTO_FILE_ID),
                            _ => None,
                        };
                        if let Some(field) = update_field_for_private {
                            sqlx::query(&format!(
                                "UPDATE {} SET {} = ? WHERE file_id = ?",
                                FileLocalModel::table_name(),
                                field
                            ))
                            .bind(file_id)
                            .bind(private_id)
                            .execute(&mut *tx)
                            .await?;
                        }
                    }

                    if crypto_id > 0 {
                        let update_field_for_crypto = match storage_type {
                            FileModel::STORAGE_TYPE_LOCAL_PUBLIC => Some(FileLocalModel::PUBLIC_FILE_ID),
                            FileModel::STORAGE_TYPE_LOCAL_PRIVATE => Some(FileLocalModel::PRIVATE_FILE_ID),
                            _ => None,
                        };
                        if let Some(field) = update_field_for_crypto {
                            sqlx::query(&format!(
                                "UPDATE {} SET {} = ? WHERE file_id = ?",
                                FileLocalModel::table_name(),
                                field
                            ))
                            .bind(file_id)
                            .bind(crypto_id)
                            .execute(&mut *tx)
                            .await?;
                        }
                    }
                }
            }

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
                let st = storage_type.to_string();
                if let Ok(bp) = self.helper.config.get_base_path(&st).await {
                    let full = bp.join(&rp);
                    if let Err(e) = tokio::fs::remove_file(&full).await {
                        tracing::warn!("create_from_local: rollback remove file failed: {}", e);
                    }
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
                    storage_type,
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

            let src_full = self.helper.get_full_local_path(&file.storage_type, &local.local_path).await?;
            let copy_prefix = format!("{}_{}_copy", app_id, user_id);
            let copy_ext = crate::common::extract_extension(Some(&file.file_name));
            let (_rel, dst_full) = self.helper.create_new_file(&file.storage_type, &copy_prefix, copy_ext).await?;
            fs::copy(&src_full, &dst_full).await?;

            // 拷贝源文件的标签
            let source_tags = self.data_dao.get_file_tag_names_for_user(file.id, user_id, app_id).await?;
            let source_tag_refs: Vec<&str> = source_tags.iter().map(String::as_str).collect();

            let (new_file, new_file_user) = self
                .create_from_local_file(
                    dst_full.to_str().unwrap_or(""),
                    user_id,
                    add_user_id,
                    app_id,
                    &file.storage_type,
                    Some(&file.file_name),
                    LocalFileMode::Move,
                    Some(file.id),
                    None,
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
            let sync_ctx =
                FileOpContext::new(file_user, &self.helper, &self.oss_config).with_file(file)?;
            let (local_file, local_file_user) = self.sync_oss_to_local(sync_ctx, env_data).await?;

            // 对本地文件进行物理拷贝，创建独立副本
            let copy_ctx = FileOpContext::new(&local_file_user, &self.helper, &self.oss_config)
                .with_file(&local_file)?;
            Box::pin(self.copy_file(copy_ctx, env_data)).await
        }
    }

    // ==================== 操作方法 6: 存储类型转换 ====================
    /// 转换文件存储类型（加密/解密/拷贝）
    ///
    /// # Arguments
    /// * `file_user` - 文件用户记录
    /// * `target_storage_type` - 目标存储类型（STORAGE_TYPE_LOCAL_PUBLIC/PRIVATE/CRYPTO）
    /// * `env_data` - 请求环境数据
    ///
    /// # Returns
    /// * `Ok((FileModel, FileUserModel))` - 转换后的文件和用户记录
    /// * `Err(FileError)` - 转换失败
    ///
    /// # 说明
    /// - 如果已存在对应类型的关联文件，直接返回
    /// - PUBLIC <-> PRIVATE: 直接拷贝
    /// - PUBLIC/PRIVATE -> CRYPTO: 加密
    /// - CRYPTO -> PUBLIC/PRIVATE: 解密
    pub async fn convert_storage_type(
        &self,
        file_user: &FileUserModel,
        target_storage_type: &str,
        env_data: Option<&RequestEnv>,
    ) -> FileResult<(FileModel, FileUserModel)> {
        use tracing::info;

        // 验证目标存储类型
        if !FileModel::is_local_key(target_storage_type) {
            return Err(FileError::Param(fluent_message!(
                "file-invalid-storage-type",
                {"storage_type": target_storage_type}
            )));
        }

        // 获取源文件信息
        let source_file = self
            .helper
            .find_file_by_id(file_user.file_id)
            .await?
            .ok_or_else(|| FileError::System(fluent_message!("file-not-found")))?;

        // 验证源文件是本地存储
        if !FileModel::is_local_key(&source_file.storage_type) {
            return Err(FileError::Param(fluent_message!(
                "file-source-must-be-local"
            )));
        }

        // 如果源类型和目标类型相同，直接返回
        if source_file.storage_type == target_storage_type {
            return Ok((source_file, file_user.clone()));
        }

        // 获取源文件的 local 记录
        let source_local = self
            .helper
            .find_file_local_by_file_id(source_file.id)
            .await?
            .ok_or_else(|| FileError::System(fluent_message!("file-local-not-found")))?;

        // 检查是否已存在关联的目标类型文件
        let existing_file_id = match target_storage_type {
            FileModel::STORAGE_TYPE_LOCAL_PUBLIC => source_local.public_file_id,
            FileModel::STORAGE_TYPE_LOCAL_PRIVATE => source_local.private_file_id,
            FileModel::STORAGE_TYPE_LOCAL_CRYPTO => source_local.crypto_file_id,
            _ => 0,
        };

        if existing_file_id > 0 {
            info!(
                "convert_storage_type: found existing related file_id={}",
                existing_file_id
            );
            // 检查文件是否存在
            if let Some(existing_file) = self.helper.find_file_by_id(existing_file_id).await? {
                // 检查用户是否已关联
                if let Some(existing_file_user) = self
                    .helper
                    .find_file_user(
                        file_user.user_id,
                        file_user.app_id,
                        existing_file_id,
                        FileUserStatus::Normal,
                    )
                    .await?
                {
                    return Ok((existing_file, existing_file_user));
                }

                // 创建用户关联
                let now = now_time()?;
                let mut tx = self.helper.db.begin().await?;
                let tx_result: FileResult<()> = async {
                    Insert::<_, FileUserModel>::new()
                        .set(FileUserModel::USER_ID, file_user.user_id)
                        .set(FileUserModel::ADD_USER_ID, file_user.add_user_id)
                        .set(FileUserModel::APP_ID, file_user.app_id)
                        .set(FileUserModel::FILE_ID, existing_file_id)
                        .set(FileUserModel::STATUS, FileUserStatus::Normal as i8)
                        .set(FileUserModel::SOURCE_URL, "")
                        .set(FileUserModel::SOURCE_MD5, "")
                        .set(FileUserModel::ADD_TIME, now)
                        .set(FileUserModel::DELETE_TIME, 0u64)
                        .execute(&mut *tx)
                        .await?;
                    Ok(())
                }
                .await;

                match tx_result {
                    Ok(_) => tx.commit().await?,
                    Err(e) => {
                        let _ = tx.rollback().await;
                        return Err(e);
                    }
                }

                let new_file_user = self
                    .helper
                    .find_file_user(
                        file_user.user_id,
                        file_user.app_id,
                        existing_file_id,
                        FileUserStatus::Normal,
                    )
                    .await?
                    .ok_or_else(|| FileError::System(fluent_message!("file-user-create-error")))?;

                return Ok((existing_file, new_file_user));
            }
        }

        // 需要执行转换操作
        info!(
            "convert_storage_type: converting from {} to {}",
            source_file.storage_type, target_storage_type
        );

        let source_full_path = self
            .helper
            .get_full_local_path(&source_file.storage_type, &source_local.local_path).await?;

        // 根据源类型和目标类型决定操作
        let (new_relative_path, new_file_size) = match (
            source_file.storage_type.as_str(),
            target_storage_type,
        ) {
            // 加密操作: PUBLIC/PRIVATE -> CRYPTO
            (FileModel::STORAGE_TYPE_LOCAL_PUBLIC, FileModel::STORAGE_TYPE_LOCAL_CRYPTO)
            | (FileModel::STORAGE_TYPE_LOCAL_PRIVATE, FileModel::STORAGE_TYPE_LOCAL_CRYPTO) => {
                info!("convert_storage_type: encrypting file");
                let (relative_path, _full_path) = self
                    .helper
                    .encrypt_file(&source_full_path)
                    .await
                    .map_err(|e| {
                        FileError::System(fluent_message!("file-encrypt-error", {"error": e.to_string()}))
                    })?;
                (relative_path, source_file.file_size)
            }

            // 解密操作: CRYPTO -> PUBLIC/PRIVATE
            (FileModel::STORAGE_TYPE_LOCAL_CRYPTO, FileModel::STORAGE_TYPE_LOCAL_PUBLIC)
            | (FileModel::STORAGE_TYPE_LOCAL_CRYPTO, FileModel::STORAGE_TYPE_LOCAL_PRIVATE) => {
                info!("convert_storage_type: decrypting file");
                let (relative_path, _full_path) = self
                    .helper
                    .decrypt_file_to_storage(
                        &source_local.local_path,
                        target_storage_type,
                    )
                    .await
                    .map_err(|e| {
                        FileError::System(fluent_message!("file-decrypt-error", {"error": e.to_string()}))
                    })?;
                (relative_path, source_file.file_size)
            }

            // 拷贝操作: PUBLIC <-> PRIVATE
            _ => {
                info!("convert_storage_type: copying file");
                let prefix = format!("{}_{}_convert", file_user.app_id, file_user.user_id);
                let relative_path = self
                    .helper
                    .copy_file_to_storage(
                        target_storage_type,
                        source_full_path.to_str().unwrap(),
                        &prefix,
                        Some(&source_file.file_name),
                    )
                    .await?;
                (relative_path, source_file.file_size)
            }
        };

        // 创建新文件记录
        let now = now_time()?;
        let mut tx = self.helper.db.begin().await?;

        let tx_result: FileResult<u64> = async {
            // 插入 lst_file
            let file_res = Insert::<_, FileModel>::new()
                .set(FileModel::STORAGE_TYPE, target_storage_type)
                .set(FileModel::STATUS, FileStatus::Normal as i8)
                .set(FileModel::FILE_MD5, &source_file.file_md5)
                .set(FileModel::FILE_SIZE, new_file_size)
                .set(FileModel::FILE_NAME, &source_file.file_name)
                .set(FileModel::CONTENT_TYPE, &source_file.content_type)
                .set(FileModel::MODIFY_TIME, source_file.modify_time)
                .set(FileModel::FROM_USER_ID, file_user.add_user_id)
                .set(FileModel::ADD_TIME, now)
                .set(FileModel::CHANGE_TIME, 0u64)
                .set(FileModel::COPY_FILE_ID, source_file.id)
                .execute(&mut *tx)
                .await?;

            let new_file_id = file_res.last_insert_id();

            // 插入 lst_file_local
            let mut local_insert = Insert::<_, FileLocalModel>::new()
                .set(FileLocalModel::FILE_ID, new_file_id)
                .set(FileLocalModel::SOURCE_TYPE, FileSourceType::LocalPath as i8)
                .set(FileLocalModel::SOURCE_NAME, "converted")
                .set(FileLocalModel::FROM_OSS_FILE_ID, 0u64)
                .set(FileLocalModel::LOCAL_PATH, &new_relative_path)
                .set(FileLocalModel::FILE_CHUNK_TOTAL, 0u32)
                .set(FileLocalModel::FILE_CHUNK_SUCC, 0u32)
                .set(FileLocalModel::FILE_CHUNK_SIZE, 0u64)
                .set(FileLocalModel::LAST_ERROR, "");

            // 继承源文件的所有关联字段，并设置新文件指向源文件的关联
            // 1. 继承源文件已有的关联
            local_insert = local_insert
                .set(FileLocalModel::PUBLIC_FILE_ID, source_local.public_file_id)
                .set(FileLocalModel::PRIVATE_FILE_ID, source_local.private_file_id)
                .set(FileLocalModel::CRYPTO_FILE_ID, source_local.crypto_file_id);

            // 2. 设置新文件指向源文件的关联
            match source_file.storage_type.as_str() {
                FileModel::STORAGE_TYPE_LOCAL_PUBLIC => {
                    local_insert = local_insert.set(FileLocalModel::PUBLIC_FILE_ID, source_file.id);
                }
                FileModel::STORAGE_TYPE_LOCAL_PRIVATE => {
                    local_insert = local_insert.set(FileLocalModel::PRIVATE_FILE_ID, source_file.id);
                }
                FileModel::STORAGE_TYPE_LOCAL_CRYPTO => {
                    local_insert = local_insert.set(FileLocalModel::CRYPTO_FILE_ID, source_file.id);
                }
                _ => {}
            }

            local_insert.execute(&mut *tx).await?;

            // 插入 lst_file_user
            Insert::<_, FileUserModel>::new()
                .set(FileUserModel::USER_ID, file_user.user_id)
                .set(FileUserModel::ADD_USER_ID, file_user.add_user_id)
                .set(FileUserModel::APP_ID, file_user.app_id)
                .set(FileUserModel::FILE_ID, new_file_id)
                .set(FileUserModel::STATUS, FileUserStatus::Normal as i8)
                .set(FileUserModel::SOURCE_URL, "")
                .set(FileUserModel::SOURCE_MD5, "")
                .set(FileUserModel::ADD_TIME, now)
                .set(FileUserModel::DELETE_TIME, 0u64)
                .execute(&mut *tx)
                .await?;

            // 更新源文件的关联字段（源文件指向新文件）
            let update_field = match target_storage_type {
                FileModel::STORAGE_TYPE_LOCAL_PUBLIC => FileLocalModel::PUBLIC_FILE_ID,
                FileModel::STORAGE_TYPE_LOCAL_PRIVATE => FileLocalModel::PRIVATE_FILE_ID,
                FileModel::STORAGE_TYPE_LOCAL_CRYPTO => FileLocalModel::CRYPTO_FILE_ID,
                _ => return Err(FileError::System(fluent_message!("file-invalid-storage-type"))),
            };

            sqlx::query(&format!(
                "UPDATE {} SET {} = ? WHERE file_id = ?",
                FileLocalModel::table_name(),
                update_field
            ))
            .bind(new_file_id)
            .bind(source_file.id)
            .execute(&mut *tx)
            .await?;

            // 更新所有相关文件的关联字段
            // 如果源文件有 public_file_id，更新该 public 文件指向新文件
            if source_local.public_file_id > 0 {
                let update_field_for_public = match target_storage_type {
                    FileModel::STORAGE_TYPE_LOCAL_PRIVATE => Some(FileLocalModel::PRIVATE_FILE_ID),
                    FileModel::STORAGE_TYPE_LOCAL_CRYPTO => Some(FileLocalModel::CRYPTO_FILE_ID),
                    _ => None,
                };
                if let Some(field) = update_field_for_public {
                    sqlx::query(&format!(
                        "UPDATE {} SET {} = ? WHERE file_id = ?",
                        FileLocalModel::table_name(),
                        field
                    ))
                    .bind(new_file_id)
                    .bind(source_local.public_file_id)
                    .execute(&mut *tx)
                    .await?;
                }
            }

            // 如果源文件有 private_file_id，更新该 private 文件指向新文件
            if source_local.private_file_id > 0 {
                let update_field_for_private = match target_storage_type {
                    FileModel::STORAGE_TYPE_LOCAL_PUBLIC => Some(FileLocalModel::PUBLIC_FILE_ID),
                    FileModel::STORAGE_TYPE_LOCAL_CRYPTO => Some(FileLocalModel::CRYPTO_FILE_ID),
                    _ => None,
                };
                if let Some(field) = update_field_for_private {
                    sqlx::query(&format!(
                        "UPDATE {} SET {} = ? WHERE file_id = ?",
                        FileLocalModel::table_name(),
                        field
                    ))
                    .bind(new_file_id)
                    .bind(source_local.private_file_id)
                    .execute(&mut *tx)
                    .await?;
                }
            }

            // 如果源文件有 crypto_file_id，更新该 crypto 文件指向新文件
            if source_local.crypto_file_id > 0 {
                let update_field_for_crypto = match target_storage_type {
                    FileModel::STORAGE_TYPE_LOCAL_PUBLIC => Some(FileLocalModel::PUBLIC_FILE_ID),
                    FileModel::STORAGE_TYPE_LOCAL_PRIVATE => Some(FileLocalModel::PRIVATE_FILE_ID),
                    _ => None,
                };
                if let Some(field) = update_field_for_crypto {
                    sqlx::query(&format!(
                        "UPDATE {} SET {} = ? WHERE file_id = ?",
                        FileLocalModel::table_name(),
                        field
                    ))
                    .bind(new_file_id)
                    .bind(source_local.crypto_file_id)
                    .execute(&mut *tx)
                    .await?;
                }
            }

            self.log_dao
                .add(
                    new_file_id,
                    0,
                    file_user.user_id,
                    &format!(
                        "convert_storage_type: {} -> {}",
                        source_file.storage_type, target_storage_type
                    ),
                    Some(&mut tx),
                )
                .await;

            Ok(new_file_id)
        }
        .await;

        let new_file_id = match tx_result {
            Ok(id) => {
                tx.commit().await?;
                id
            }
            Err(e) => {
                let _ = tx.rollback().await;
                // 清理已创建的文件
                if let Ok(base_path) = self.helper.config.get_base_path(target_storage_type).await {
                    let full_path = base_path.join(&new_relative_path);
                    if let Err(e) = tokio::fs::remove_file(&full_path).await {
                        tracing::warn!("convert_storage_type: cleanup file failed: {}", e);
                    }
                }
                return Err(e);
            }
        };

        // 查询新创建的文件和用户记录
        let new_file = self
            .helper
            .find_file_by_id(new_file_id)
            .await?
            .ok_or_else(|| FileError::System(fluent_message!("file-create-error")))?;

        let new_file_user = self
            .helper
            .find_file_user(
                file_user.user_id,
                file_user.app_id,
                new_file_id,
                FileUserStatus::Normal,
            )
            .await?
            .ok_or_else(|| FileError::System(fluent_message!("file-user-create-error")))?;

        self.logger
            .add(
                &LogFileCreate {
                    action: "convert_storage_type",
                    storage_type: target_storage_type,
                    user_id: file_user.user_id,
                    file_id: new_file.id,
                    file_md5: &new_file.file_md5,
                },
                Some(new_file.id),
                Some(file_user.user_id),
                None,
                env_data,
            )
            .await;

        Ok((new_file, new_file_user))
    }
}
