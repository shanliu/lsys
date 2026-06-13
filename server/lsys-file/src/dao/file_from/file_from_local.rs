use std::path::PathBuf;

use super::super::logger::*;
use super::super::*;
use crate::model::*;
use lsys_core::db::Insert;
use lsys_core::fluent_message;
use lsys_core::utils::{RequestEnv, STRING_CLEAR_FORMAT, StringClear, now_time, string_clear};
use tokio::fs;
use tracing::warn;

/// 本地文件操作模式
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LocalFileMode {
    /// 移动：操作完成后删除源文件
    Move,
    /// 拷贝：保留源文件
    Copy,
}

/// 本地文件拷贝模式
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LocalFileCopyMode {
    /// 引用：不拷贝文件，创建新的 file_ref 引用已有文件
    Ref,
    /// 拷贝：创建独立的文件副本
    Copy,
}

/// 传入 `create_from_local_file` 的源文件类型
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LocalFileSource {
    /// local_file_path 是明文：若 storage_type 为 CRYPTO，函数内部自动加密后存入
    Plaintext,
    /// local_file_path 是密文（CRYPTO 格式）：若 storage_type 非 CRYPTO，函数内部自动解密后存入
    Encrypted,
}

impl FileDao {
    // ==================== 创建方法 4: 已知本地文件生成 ====================
    ///
    /// - `user_id`: 文件属于的用户ID,0=系统
    /// - `add_user_id`: 文件添加(上传)用户ID
    /// - `storage_type`: 存储类型（STORAGE_TYPE_LOCAL_PUBLIC/PRIVATE/CRYPTO）
    /// - `source`: 源文件类型（Plaintext=明文，Encrypted=密文/已加密）
    /// - `force_new`: true 时跳过去重检查，强制创建独立的新文件记录
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
        source: LocalFileSource,
        force_new: bool,
        tag_names: &[&str],
        expire_time: Option<u64>,
        env_data: Option<&RequestEnv>,
    ) -> FileResult<(FileModel, FileRefModel)> {
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

        // 验证文件路径是否存在且是正常文件
        if !path.exists() {
            return Err(FileError::Param(fluent_message!(
                "file-local-path-not-exist",
                {"path": local_file_path}
            )));
        }
        if !path.is_file() {
            return Err(FileError::Param(fluent_message!(
                "file-local-path-not-file",
                {"path": local_file_path}
            )));
        }

        let now = now_time()?;

        // 确定用户可见文件名（使用原始 local_file_path 的文件名，转换前）
        let actual_name = {
            let raw = file_name.unwrap_or_else(|| {
                path.file_name()
                    .map(|n| n.to_str().unwrap_or("unknown"))
                    .unwrap_or("unknown")
            });
            string_clear(raw, StringClear::Option(STRING_CLEAR_FORMAT), Some(254))
        };

        // 如需加密/解密，在计算 MD5 之前完成转换，确保 MD5 对应最终存储内容
        let (actual_path_buf, pre_converted_relative): (PathBuf, Option<String>) =
            match (source, storage_type == FileModel::STORAGE_TYPE_LOCAL_CRYPTO) {
                (LocalFileSource::Plaintext, true) => {
                    // 明文 → CRYPTO：加密后放入 CRYPTO 目录
                    info!("create_from_local_file: encrypting plaintext file for CRYPTO storage");
                    let (rel, full) = self.helper.encrypt_new_file(local_file_path).await?;
                    // 源文件删除延迟到事务提交后，避免事务回滚导致数据丢失
                    (full, Some(rel))
                }
                (LocalFileSource::Encrypted, false) => {
                    // CRYPTO 密文 → 明文存储：解密后放入目标目录
                    info!(
                        "create_from_local_file: decrypting encrypted file for non-CRYPTO storage"
                    );
                    let enc_path = PathBuf::from(local_file_path);
                    let crypto_base = self
                        .helper
                        .config
                        .get_base_path(FileModel::STORAGE_TYPE_LOCAL_CRYPTO)
                        .await?;
                    let enc_rel = enc_path
                        .strip_prefix(&crypto_base)
                        .map_err(|_| {
                            FileError::Param(fluent_message!("file-not-in-crypto-storage"))
                        })?
                        .to_str()
                        .ok_or_else(|| {
                            FileError::System(fluent_message!("file-path-invalid-utf8"))
                        })?
                        .to_string();
                    // 提取原始文件扩展名
                    let original_ext = crate::common::extract_extension(Some(&actual_name));
                    let (rel, full) = self
                        .helper
                        .decrypt_file_to_storage(&enc_rel, storage_type, Some(original_ext))
                        .await?;
                    // 源文件删除延迟到事务提交后，避免事务回滚导致数据丢失
                    (full, Some(rel))
                }
                _ => {
                    // Encrypted+CRYPTO 或 Plaintext+非CRYPTO：无需转换
                    (PathBuf::from(local_file_path), None)
                }
            };

        // 计算 MD5（转换后的最终存储文件）
        let file_md5 = self.helper.compute_file_md5(&actual_path_buf).await?;
        info!("create_from_local_file: computed md5={}", file_md5);

        // 是否已完成预转换（加密或解密）
        let converted = pre_converted_relative.is_some();

        if !force_new {
            // 辅助函数.1: 检查是否存在
            // find_existing_local_file 已在内部完成物理校验（size + MD5 重算），直接信任结果。
            if let Some(existing) = self
                .helper
                .find_existing_local_file(storage_type, &file_md5)
                .await?
            {
                info!(
                    "create_from_local_file: existing file found, id={}",
                    existing.id
                );

                if let Some(fu) = self
                        .helper
                        .find_file_ref(user_id, app_id, existing.id, FileUserStatus::Normal)
                        .await?
                    {
                        // 已存在，清理当前文件
                        info!("create_from_local_file: user already linked");
                        if converted {
                            // 预转换产生的文件已无用，清理
                            if let Err(e) = fs::remove_file(&actual_path_buf).await {
                                warn!(
                                    "create_from_local: cleanup converted file on dedup failed: {}",
                                    e
                                );
                            }
                        }
                        if mode == LocalFileMode::Move
                            && let Err(e) = fs::remove_file(local_file_path).await {
                                warn!(
                                    "create_from_local: remove source file (Move mode) failed: {}",
                                    e
                                );
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

                    // 创建 file_ref
                    info!("create_from_local_file: creating file_ref link to existing file");
                    let mut tx = self.helper.db.begin().await?;
                    let tx_result: FileResult<()> = async {
                        Insert::<_, FileRefModel>::new()
                            .set(FileRefModel::USER_ID, user_id)
                            .set(FileRefModel::ADD_USER_ID, add_user_id)
                            .set(FileRefModel::APP_ID, app_id)
                            .set(FileRefModel::FILE_ID, existing.id)
                            .set(FileRefModel::STATUS, FileUserStatus::Normal as i8)
                            .set(FileRefModel::SOURCE_URL, "")
                            .set(FileRefModel::SOURCE_MD5, "")
                            .set(FileRefModel::FILE_NAME, actual_name.as_str())
                            .set(FileRefModel::ADD_TIME, now)
                            .set(FileRefModel::DELETE_TIME, 0u64)
                            .set(FileRefModel::EXPIRE_TIME, expire_time.unwrap_or(0))
                            .execute(&mut *tx)
                            .await?;
                        self.log_dao
                            .add(
                                existing.id,
                                0,
                                user_id,
                                "create_from_local: existing file, new file_ref",
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
                            if let Err(rb_err) = tx.rollback().await {
                                warn!("create_from_local: rollback failed: {}", rb_err);
                            }
                            return Err(e);
                        }
                    }

                    if converted
                        && let Err(e) = fs::remove_file(&actual_path_buf).await {
                            warn!(
                                "create_from_local: cleanup converted file on dedup failed: {}",
                                e
                            );
                        }
                    if mode == LocalFileMode::Move
                        && let Err(e) = fs::remove_file(local_file_path).await {
                            warn!(
                                "create_from_local: remove source file (Move mode) failed: {}",
                                e
                            );
                        }
                    // 查询刚创建的 file_ref
                    let file_ref = self
                        .helper
                        .find_file_ref(user_id, app_id, existing.id, FileUserStatus::Normal)
                        .await?
                        .ok_or_else(|| {
                            FileError::System(fluent_message!("file-ref-create-error"))
                        })?;
                    return Ok((existing, file_ref));
            }
        } // end if !force_new

        // 不存在或物理文件丢失: 存入存储路径
        info!("create_from_local_file: no existing file, creating new record");

        let relative_path = if let Some(rel) = pre_converted_relative {
            // 已完成加密/解密，文件已在目标存储目录
            rel
        } else {
            let prefix = format!("{}_{}_local", app_id, user_id);
            match mode {
                LocalFileMode::Move => {
                    self.helper
                        .move_file_to_storage(
                            storage_type,
                            local_file_path,
                            &prefix,
                            Some(&actual_name),
                        )
                        .await?
                }
                LocalFileMode::Copy => {
                    self.helper
                        .copy_file_to_storage(
                            storage_type,
                            local_file_path,
                            &prefix,
                            Some(&actual_name),
                        )
                        .await?
                }
            }
        };

        let full_path = self
            .helper
            .get_full_local_path(storage_type, &relative_path)
            .await?;
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
                .set(FileModel::ORIGIN_NAME, &actual_name)
                .set(FileModel::CONTENT_TYPE, &content_type)
                .set(FileModel::MODIFY_TIME, modify_time)
                .set(FileModel::FROM_USER_ID, add_user_id)
                .set(FileModel::ADD_TIME, now)
                .set(FileModel::CHANGE_TIME, 0u64)
                .set(FileModel::LOCAL_PATH_OWNER_ID, 0u64)
                .execute(&mut *tx)
                .await?;

            let file_id = file_res.last_insert_id();

            Insert::<_, FileLocalModel>::new()
                .set(FileLocalModel::FILE_ID, file_id)
                .set(FileLocalModel::SOURCE_TYPE, FileSourceType::LocalPath as i8)
                .set(FileLocalModel::SOURCE_NAME, local_file_path)
                .set(FileLocalModel::LOCAL_PATH, &relative_path)
                .set(FileLocalModel::FILE_CHUNK_TOTAL, 0u32)
                .set(FileLocalModel::FILE_CHUNK_SUCC, 0u32)
                .set(FileLocalModel::FILE_CHUNK_SIZE, 0u64)
                .set(FileLocalModel::LAST_ERROR, "")
                .execute(&mut *tx)
                .await?;

            Insert::<_, FileRefModel>::new()
                .set(FileRefModel::USER_ID, user_id)
                .set(FileRefModel::ADD_USER_ID, add_user_id)
                .set(FileRefModel::APP_ID, app_id)
                .set(FileRefModel::FILE_ID, file_id)
                .set(FileRefModel::STATUS, FileUserStatus::Normal as i8)
                .set(FileRefModel::SOURCE_URL, "")
                .set(FileRefModel::SOURCE_MD5, "")
                .set(FileRefModel::FILE_NAME, &actual_name)
                .set(FileRefModel::ADD_TIME, now)
                .set(FileRefModel::DELETE_TIME, 0u64)
                .set(FileRefModel::EXPIRE_TIME, expire_time.unwrap_or(0))
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
                // 转换模式下（加密/解密），源文件在事务提交成功后才删除
                if converted && mode == LocalFileMode::Move
                    && let Err(e) = fs::remove_file(local_file_path).await {
                        warn!(
                            "create_from_local: remove source after commit failed: {}",
                            e
                        );
                    }
                id
            }
            Err(e) => {
                if let Err(rb_err) = tx.rollback().await {
                    warn!("create_from_local: rollback failed: {}", rb_err);
                }
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

        let file_ref = self
            .helper
            .find_file_ref(user_id, app_id, file_id, FileUserStatus::Normal)
            .await?
            .ok_or_else(|| FileError::System(fluent_message!("file-ref-create-error")))?;
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

        Ok((file, file_ref))
    }
    // ==================== 本地文件类型转换 ====================

    /// 本地文件类型转换（public ↔ private ↔ crypto）
    ///
    /// 必须在不同的本地存储类型之间转换，否则返回错误。
    /// 始终创建独立的新文件记录（force_new=true），不会返回原有记录。
    ///
    /// 通过 `FileOpContext` 传入参数：
    /// - 必须: `file_ref`
    /// - 可选: `file`（未提供时自动查询）
    ///
    /// # Arguments
    /// * `target_storage_type` - 目标存储类型（必须是本地类型，且必须与源类型不同）
    ///
    /// # Returns
    /// 返回新创建的文件和文件用户关联记录
    pub async fn local_file_convert(
        &self,
        ctx: FileOpContext<'_>,
        target_storage_type: &str,
        env_data: Option<&RequestEnv>,
    ) -> FileResult<(FileModel, FileRefModel)> {
        use tracing::info;

        // 验证目标存储类型
        if !FileModel::is_local_key(target_storage_type) {
            return Err(FileError::Param(fluent_message!(
                "file-invalid-storage-type",
                {"storage_type": target_storage_type}
            )));
        }

        let file_ref = ctx.file_ref;
        let file = ctx.file().await?;

        // 验证源文件是本地类型
        if !file.is_local() {
            return Err(FileError::Param(fluent_message!("file-not-local-type")));
        }

        // 验证源类型和目标类型必须不同
        if file.storage_type == target_storage_type {
            return Err(FileError::Param(fluent_message!(
                "file-convert-requires-different-types"
            )));
        }

        info!(
            "local_file_convert: converting file_id={} from {} to {}",
            file.id, file.storage_type, target_storage_type
        );

        // 获取源文件的本地路径
        let source_local = self
            .helper
            .find_file_local_by_file_id(file.id)
            .await?
            .ok_or_else(|| FileError::System(fluent_message!("file-local-not-found")))?;

        let source_full_path = self
            .helper
            .get_full_local_path(&file.storage_type, &source_local.local_path)
            .await?;

        // 根据目标类型选择转换方式（加密/解密由 create_from_local_file 内部处理）
        let source_full_path_str = source_full_path
            .to_str()
            .ok_or_else(|| FileError::System(fluent_message!("file-path-invalid-utf8")))?;
        let (new_file, new_file_ref) = match (file.storage_type.as_str(), target_storage_type) {
            // 转换为加密文件：源是明文，函数内部加密
            (_, FileModel::STORAGE_TYPE_LOCAL_CRYPTO) => {
                info!("local_file_convert: encrypting via create_from_local_file");
                self.create_from_local_file(
                    source_full_path_str,
                    file_ref.user_id,
                    file_ref.add_user_id,
                    file_ref.app_id,
                    target_storage_type,
                    Some(&file_ref.file_name),
                    LocalFileMode::Copy,        // 保留源文件
                    LocalFileSource::Plaintext, // 源是明文，内部自动加密
                    true,
                    &[],
                    Some(file_ref.expire_time),
                    env_data,
                )
                .await?
            }

            // 从加密文件转换：源是密文，函数内部解密
            (FileModel::STORAGE_TYPE_LOCAL_CRYPTO, _) => {
                info!("local_file_convert: decrypting via create_from_local_file");
                self.create_from_local_file(
                    source_full_path_str,
                    file_ref.user_id,
                    file_ref.add_user_id,
                    file_ref.app_id,
                    target_storage_type,
                    Some(&file_ref.file_name),
                    LocalFileMode::Copy,        // 保留源加密文件
                    LocalFileSource::Encrypted, // 源是密文，内部自动解密
                    true,
                    &[],
                    Some(file_ref.expire_time),
                    env_data,
                )
                .await?
            }

            // public ↔ private 之间转换（均为明文，直接拷贝）
            _ => {
                info!("local_file_convert: copying between public/private");
                self.create_from_local_file(
                    source_full_path_str,
                    file_ref.user_id,
                    file_ref.add_user_id,
                    file_ref.app_id,
                    target_storage_type,
                    Some(&file_ref.file_name),
                    LocalFileMode::Copy,
                    LocalFileSource::Plaintext, // public/private 均为明文
                    true,
                    &[],
                    Some(file_ref.expire_time),
                    env_data,
                )
                .await?
            }
        };

        // 记录转换派生关系
        if let Err(e) = self
            .file_ops
            .add_lineage(
                file_ref.user_id,
                file_ref.app_id,
                file.id,
                new_file.id,
                FileLineageRelType::Copy as i8,
            )
            .await
        {
            tracing::warn!("local_file_convert: add_lineage failed: {}", e);
        }

        // 复制标签
        let source_tags = self
            .data_dao
            .helper
            .get_file_tag_names_for_user(file.id, file_ref.user_id, file_ref.app_id)
            .await?;
        let tag_refs: Vec<&str> = source_tags.iter().map(String::as_str).collect();
        self.tag_dao
            .batch_add_tags(
                new_file.id,
                file_ref.user_id,
                file_ref.app_id,
                &tag_refs,
                None,
            )
            .await?;

        self.logger
            .add(
                &LogFileSync {
                    action: "local_file_convert",
                    user_id: file_ref.user_id,
                    file_id: new_file.id,
                    storage_type: target_storage_type,
                },
                Some(new_file.id),
                Some(file_ref.user_id),
                None,
                env_data,
            )
            .await;

        Ok((new_file, new_file_ref))
    }

    // ==================== 相同类型本地文件拷贝 ====================

    /// 相同类型本地文件拷贝或引用
    ///
    /// 必须在相同的本地存储类型之间操作，否则返回错误。
    ///
    /// 通过 `FileOpContext` 传入参数：
    /// - 必须: `file_ref`
    /// - 可选: `file`（未提供时自动查询）
    ///
    /// # Arguments
    /// * `target_storage_type` - 目标存储类型（必须是本地类型，且必须与源类型相同）
    /// * `copy_mode` - 拷贝模式（Ref=引用，Copy=拷贝）
    ///
    /// # Returns
    /// 返回文件和文件用户关联记录
    pub async fn local_file_copy(
        &self,
        ctx: FileOpContext<'_>,
        target_storage_type: &str,
        copy_mode: LocalFileCopyMode,
        env_data: Option<&RequestEnv>,
    ) -> FileResult<(FileModel, FileRefModel)> {
        use tracing::info;

        // 验证目标存储类型
        if !FileModel::is_local_key(target_storage_type) {
            return Err(FileError::Param(fluent_message!(
                "file-invalid-storage-type",
                {"storage_type": target_storage_type}
            )));
        }

        let file_ref = ctx.file_ref;
        let file = ctx.file().await?;

        // 验证源文件是本地类型
        if !file.is_local() {
            return Err(FileError::Param(fluent_message!("file-not-local-type")));
        }

        // 验证源类型和目标类型必须相同
        if file.storage_type != target_storage_type {
            return Err(FileError::Param(fluent_message!(
                "file-same-type-copy-requires-same-types"
            )));
        }

        match copy_mode {
            LocalFileCopyMode::Ref => {
                // 引用模式：不拷贝文件，创建新的 file_ref
                info!(
                    "local_file_copy(Ref): creating reference for file_id={} in storage type {}",
                    file.id, target_storage_type
                );

                let now = now_time()?;

                // 检查是否已存在该用户的引用
                if let Some(existing_ref) = self
                    .helper
                    .find_file_ref(file_ref.user_id, file_ref.app_id, file.id, FileUserStatus::Normal)
                    .await?
                {
                    info!("local_file_copy(Ref): reference already exists");
                    return Ok((file.clone(), existing_ref));
                }

                // 创建新的 file_ref 引用
                let mut tx = self.helper.db.begin().await?;
                let tx_result: FileResult<()> = async {
                    Insert::<_, FileRefModel>::new()
                        .set(FileRefModel::USER_ID, file_ref.user_id)
                        .set(FileRefModel::ADD_USER_ID, file_ref.add_user_id)
                        .set(FileRefModel::APP_ID, file_ref.app_id)
                        .set(FileRefModel::FILE_ID, file.id)
                        .set(FileRefModel::STATUS, FileUserStatus::Normal as i8)
                        .set(FileRefModel::SOURCE_URL, "")
                        .set(FileRefModel::SOURCE_MD5, "")
                        .set(FileRefModel::FILE_NAME, &file_ref.file_name)
                        .set(FileRefModel::ADD_TIME, now)
                        .set(FileRefModel::DELETE_TIME, 0u64)
                        .set(FileRefModel::EXPIRE_TIME, file_ref.expire_time)
                        .execute(&mut *tx)
                        .await?;
                    
                    self.log_dao
                        .add(
                            file.id,
                            0,
                            file_ref.user_id,
                            "local_file_copy(Ref): new reference created",
                            Some(&mut tx),
                        )
                        .await;
                    Ok(())
                }
                .await;

                match tx_result {
                    Ok(_) => {
                        tx.commit().await?;
                    }
                    Err(e) => {
                        if let Err(rb_err) = tx.rollback().await {
                            warn!("local_file_copy(Ref): rollback failed: {}", rb_err);
                        }
                        return Err(e);
                    }
                }

                // 查询刚创建的 file_ref
                let new_file_ref = self
                    .helper
                    .find_file_ref(file_ref.user_id, file_ref.app_id, file.id, FileUserStatus::Normal)
                    .await?
                    .ok_or_else(|| {
                        FileError::System(fluent_message!("file-ref-create-error"))
                    })?;

                self.logger
                    .add(
                        &LogFileSync {
                            action: "local_file_copy_ref",
                            user_id: file_ref.user_id,
                            file_id: file.id,
                            storage_type: target_storage_type,
                        },
                        Some(file.id),
                        Some(file_ref.user_id),
                        None,
                        env_data,
                    )
                    .await;

                Ok((file.clone(), new_file_ref))
            }
            LocalFileCopyMode::Copy => {
                // 拷贝模式：创建独立的文件副本
                info!(
                    "local_file_copy(Copy): copying file_id={} within storage type {}",
                    file.id, target_storage_type
                );

                // 获取源文件的本地路径
                let source_local = self
                    .helper
                    .find_file_local_by_file_id(file.id)
                    .await?
                    .ok_or_else(|| FileError::System(fluent_message!("file-local-not-found")))?;

                let source_full_path = self
                    .helper
                    .get_full_local_path(&file.storage_type, &source_local.local_path)
                    .await?;

                let source_full_path_str = source_full_path
                    .to_str()
                    .ok_or_else(|| FileError::System(fluent_message!("file-path-invalid-utf8")))?;

                // 根据存储类型确定源文件类型
                let source_type = if file.storage_type == FileModel::STORAGE_TYPE_LOCAL_CRYPTO {
                    LocalFileSource::Encrypted
                } else {
                    LocalFileSource::Plaintext
                };

                let (new_file, new_file_ref) = self
                    .create_from_local_file(
                        source_full_path_str,
                        file_ref.user_id,
                        file_ref.add_user_id,
                        file_ref.app_id,
                        target_storage_type,
                        Some(&file_ref.file_name),
                        LocalFileMode::Copy, // 保留源文件
                        source_type,
                        true,
                        &[],
                        Some(file_ref.expire_time),
                        env_data,
                    )
                    .await?;

                // 记录拷贝派生关系
                if let Err(e) = self
                    .file_ops
                    .add_lineage(
                        file_ref.user_id,
                        file_ref.app_id,
                        file.id,
                        new_file.id,
                        FileLineageRelType::Copy as i8,
                    )
                    .await
                {
                    tracing::warn!("local_file_copy(Copy): add_lineage failed: {}", e);
                }

                // 复制标签
                let source_tags = self
                    .data_dao
                    .helper
                    .get_file_tag_names_for_user(file.id, file_ref.user_id, file_ref.app_id)
                    .await?;
                let tag_refs: Vec<&str> = source_tags.iter().map(String::as_str).collect();
                self.tag_dao
                    .batch_add_tags(
                        new_file.id,
                        file_ref.user_id,
                        file_ref.app_id,
                        &tag_refs,
                        None,
                    )
                    .await?;

                self.logger
                    .add(
                        &LogFileSync {
                            action: "local_file_copy_copy",
                            user_id: file_ref.user_id,
                            file_id: new_file.id,
                            storage_type: target_storage_type,
                        },
                        Some(new_file.id),
                        Some(file_ref.user_id),
                        None,
                        env_data,
                    )
                    .await;

                Ok((new_file, new_file_ref))
            }
        }
    }

    }
