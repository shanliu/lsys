use lsys_core::db::{Insert, TableMeta};
use lsys_core::dist_lock::{DistLockError, WatchdogConfig};
use lsys_core::fluent_message;
use lsys_core::utils::{RequestEnv, now_time};
use std::time::Duration;
use tracing::warn;

use super::super::file_helpers::FileHelper;
use super::super::logger::*;
use super::super::*;
use crate::common::get_content_type;
use crate::model::*;

impl FileDao {
    // ==================== 创建方法 0: 本地路径上传到 OSS 并创建文件记录 ====================
    /// 将本地文件上传到 OSS，然后创建对应的文件记录
    ///
    /// 封装了 `OssProvider::upload_from_local` + `create_from_oss` 的完整流程。
    /// 适用于本地已有文件、需要直接上传到 OSS 并入库的场景。
    ///
    /// - `local_path`: 本地文件的绝对路径
    /// - `user_id`: 文件属于的用户ID,0=系统
    /// - `add_user_id`: 文件添加(上传)用户ID
    /// - `app_id`: 应用ID
    /// - `storage_type`: 存储类型标识（如 "aliyun-oss"）
    /// - `oss_provider`: OSS 服务提供者
    /// - `tag_names`: 标签名列表
    /// - `env_data`: 请求环境信息
    ///
    /// 注意：文件名和 MIME 类型会自动从本地文件获取
    #[allow(clippy::too_many_arguments)]
    pub async fn create_from_local_upload_oss(
        &self,
        local_path: &str,
        user_id: u64,
        add_user_id: u64,
        app_id: u64,
        storage_type: &str,
        oss_provider: &dyn OssProvider,
        tag_names: &[&str],
        expire_time: Option<u64>,
        env_data: Option<&RequestEnv>,
    ) -> FileResult<(FileModel, FileRefModel)> {
        use tracing::info;
        info!(
            "create_from_local_upload_oss: starting, local_path={}, user_id={}, storage_type={}",
            local_path, user_id, storage_type
        );

        let path = std::path::PathBuf::from(local_path);
        if !path.exists() {
            return Err(FileError::Param(fluent_message!("file-not-found")));
        }

        let file_md5 = self.helper.compute_file_md5(&path).await?;
        let metadata = tokio::fs::metadata(&path).await;
        let file_size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);

        // 自动获取文件名和 content_type
        let actual_file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let actual_content_type = get_content_type(&path)
            .await
            .unwrap_or_else(|_| "application/octet-stream".to_string());

        // 先检查文件是否已存在
        if let Some(existing) = self
            .helper
            .find_existing_oss_file(storage_type, &file_md5, oss_provider)
            .await?
        {
            info!(
                "create_from_local_upload_oss: file already exists in storage, id={}",
                existing.id
            );
            // 文件已存在，直接传入文件名调用存在处理函数
            return self
                .handle_existing_file(
                    existing,
                    Some(&actual_file_name),
                    user_id,
                    add_user_id,
                    app_id,
                    tag_names,
                    expire_time,
                    None,
                )
                .await;
        }

        // 文件不存在，上传到 OSS
        info!("create_from_local_upload_oss: file not found in storage, uploading to OSS");

        // 构建上传所需的文件元数据
        let upload_info = UploadFileInfo {
            file_name: &actual_file_name,
            file_md5: &file_md5,
            file_size,
            content_type: &actual_content_type,
        };

        // 上传到 OSS
        info!("create_from_local_upload_oss: uploading to OSS");
        let oss_result = oss_provider
            .upload_from_local(local_path, &upload_info)
            .await?;

        info!(
            "create_from_local_upload_oss: upload done, object_url={}",
            &oss_result.object_url
        );

        // 调用不存在处理函数完成入库
        self.create_new_file_record(
            oss_result,
            user_id,
            add_user_id,
            app_id,
            storage_type,
            tag_names,
            expire_time,
            env_data,
            None,
        )
        .await
    }

    // ==================== 创建方法 1: OSS 远程文件 ====================
    /// 处理文件已存在的情况
    /// 当检测到文件已存在时，创建或复用 file_ref 记录
    #[allow(clippy::too_many_arguments)]
    async fn handle_existing_file(
        &self,
        existing: FileModel,
        file_name: Option<&str>,
        user_id: u64,
        add_user_id: u64,
        app_id: u64,
        tag_names: &[&str],
        expire_time: Option<u64>,
        source_url: Option<&str>,
    ) -> FileResult<(FileModel, FileRefModel)> {
        use tracing::info;

        info!(
            "handle_existing_file: existing file found, id={}",
            existing.id
        );

        // 检查 file_ref 是否已存在
        if let Some(fr) = self
            .helper
            .find_file_ref(user_id, app_id, existing.id, FileUserStatus::Normal)
            .await?
        {
            info!("handle_existing_file: user already linked to existing file");
            self.log_dao
                .add(
                    existing.id,
                    0,
                    user_id,
                    "handle_existing_file: existing file, user already linked",
                    None,
                )
                .await;
            self.tag_dao
                .batch_add_tags(existing.id, user_id, app_id, tag_names, None)
                .await?;
            return Ok((existing, fr));
        }

        info!("handle_existing_file: creating file_ref link to existing file");

        let now = now_time()?;
        let mut tx = self.helper.db.begin().await?;
        let tx_result: FileResult<()> = async {
            Insert::<_, FileRefModel>::new()
                .set(FileRefModel::USER_ID, user_id)
                .set(FileRefModel::ADD_USER_ID, add_user_id)
                .set(FileRefModel::APP_ID, app_id)
                .set(FileRefModel::FILE_ID, existing.id)
                .set(FileRefModel::STATUS, FileUserStatus::Normal as i8)
                .set(FileRefModel::SOURCE_URL, source_url.unwrap_or(""))
                .set(FileRefModel::SOURCE_MD5, "")
                .set(FileRefModel::FILE_NAME, file_name.unwrap_or(""))
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
                    "handle_existing_file: existing file, created file_ref",
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
                // 关联完成后清除可能缓存的 None 值
                self.file_url_cache.clear(&existing.id).await;
                let file_ref = self
                    .helper
                    .find_file_ref(user_id, app_id, existing.id, FileUserStatus::Normal)
                    .await?
                    .ok_or_else(|| FileError::System(fluent_message!("file-ref-create-error")))?;
                Ok((existing, file_ref))
            }
            Err(e) => {
                if let Err(rb_err) = tx.rollback().await {
                    warn!("handle_existing_file: rollback failed: {}", rb_err);
                }
                Err(e)
            }
        }
    }

    /// 处理文件不存在的情况，创建新的文件记录
    #[allow(clippy::too_many_arguments)]
    async fn create_new_file_record(
        &self,
        oss_result: OssResult,
        user_id: u64,
        add_user_id: u64,
        app_id: u64,
        storage_type: &str,
        tag_names: &[&str],
        expire_time: Option<u64>,
        env_data: Option<&RequestEnv>,
        source_url: Option<&str>,
    ) -> FileResult<(FileModel, FileRefModel)> {
        use tracing::info;

        info!(
            "create_new_file_record: creating new file, user_id={}, storage_type={}, file_md5={}",
            user_id, storage_type, &oss_result.file_md5
        );

        let now = now_time()?;
        let mut tx = self.helper.db.begin().await?;

        let tx_result: FileResult<u64> = async {
            let file_res = Insert::<_, FileModel>::new()
                .set(FileModel::STORAGE_TYPE, storage_type)
                .set(FileModel::STATUS, FileStatus::Normal as i8)
                .set(FileModel::FILE_MD5, &oss_result.file_md5)
                .set(FileModel::FILE_SIZE, oss_result.file_size.unwrap_or(0))
                .set(
                    FileModel::ORIGIN_NAME,
                    oss_result.file_name.as_deref().unwrap_or(""),
                )
                .set(
                    FileModel::CONTENT_TYPE,
                    oss_result.content_type.as_deref().unwrap_or(""),
                )
                .set(FileModel::MODIFY_TIME, oss_result.modify_time.unwrap_or(0))
                .set(FileModel::FROM_USER_ID, add_user_id)
                .set(FileModel::ADD_TIME, now)
                .set(FileModel::CHANGE_TIME, 0u64)
                .set(FileModel::LOCAL_PATH_OWNER_ID, 0u64)
                .execute(&mut *tx)
                .await?;

            let file_id = file_res.last_insert_id();

            let object_url_md5 = FileHelper::compute_str_md5(&oss_result.object_url);

            Insert::<_, FileOssModel>::new()
                .set(FileOssModel::FILE_ID, file_id)
                .set(FileOssModel::OBJECT_KEY, &oss_result.object_key)
                .set(FileOssModel::BUCKET, &oss_result.bucket)
                .set(FileOssModel::OBJECT_URL, &oss_result.object_url)
                .set(FileOssModel::OBJECT_URL_MD5, &object_url_md5)
                .set(
                    FileOssModel::REGION,
                    oss_result.region.as_deref().unwrap_or(""),
                )
                .set(FileOssModel::SIZE, oss_result.file_size.unwrap_or(0))
                .set(FileOssModel::LAST_ERROR, "")
                .execute(&mut *tx)
                .await?;

            Insert::<_, FileRefModel>::new()
                .set(FileRefModel::USER_ID, user_id)
                .set(FileRefModel::ADD_USER_ID, add_user_id)
                .set(FileRefModel::APP_ID, app_id)
                .set(FileRefModel::FILE_ID, file_id)
                .set(FileRefModel::STATUS, FileUserStatus::Normal as i8)
                .set(FileRefModel::SOURCE_URL, source_url.unwrap_or(""))
                .set(FileRefModel::SOURCE_MD5, "")
                .set(
                    FileRefModel::FILE_NAME,
                    oss_result.file_name.as_deref().unwrap_or(""),
                )
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
                    "create_new_file_record: new file created",
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
                if let Err(rb_err) = tx.rollback().await {
                    warn!("create_new_file_record: rollback failed: {}", rb_err);
                }
                return Err(e);
            }
        };

        let file = self
            .helper
            .find_file_by_id(file_id)
            .await?
            .ok_or_else(|| FileError::System(fluent_message!("file-create-error")))?;

        // 新文件入库后清除可能缓存的 None 值
        self.file_url_cache.clear(&file_id).await;

        let file_ref = self
            .helper
            .find_file_ref(user_id, app_id, file_id, FileUserStatus::Normal)
            .await?
            .ok_or_else(|| FileError::System(fluent_message!("file-ref-create-error")))?;
        self.logger
            .add(
                &LogFileCreate {
                    action: "create_from_oss",
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

    /// 从 OSS 结果创建文件记录
    ///
    /// 检测文件是否已存在：
    /// - 如果存在，复用文件并创建 file_ref 关联
    /// - 如果不存在，创建新的文件记录
    #[allow(clippy::too_many_arguments)]
    pub async fn create_from_oss(
        &self,
        oss_result: OssResult,
        user_id: u64,
        add_user_id: u64,
        app_id: u64,
        storage_type: &str,
        oss_provider: &dyn OssProvider,
        tag_names: &[&str],
        expire_time: Option<u64>,
        env_data: Option<&RequestEnv>,
        source_url: Option<&str>,
    ) -> FileResult<(FileModel, FileRefModel)> {
        use tracing::info;

        info!(
            "create_from_oss: starting, user_id={}, storage_type={}, file_md5={}",
            user_id, storage_type, &oss_result.file_md5
        );

        // 检测是否存在
        if let Some(existing) = self
            .helper
            .find_existing_oss_file(storage_type, &oss_result.file_md5, oss_provider)
            .await?
        {
            info!("create_from_oss: existing file found, id={}", existing.id);
            // 文件已存在，调用处理存在文件的函数
            self.handle_existing_file(
                existing,
                oss_result.file_name.as_deref(),
                user_id,
                add_user_id,
                app_id,
                tag_names,
                expire_time,
                source_url,
            )
            .await
        } else {
            // 文件不存在，调用创建新文件记录的函数
            self.create_new_file_record(
                oss_result,
                user_id,
                add_user_id,
                app_id,
                storage_type,
                tag_names,
                expire_time,
                env_data,
                source_url,
            )
            .await
        }
    }

    // ==================== 同步辅助: lineage 查找 + 用户关联 ====================

    /// 通过 lineage 查找当前用户已同步的目标文件，若找到则确保用户已关联。
    ///
    /// - `transfer_tags`: true 时将源文件的 tag 复制到目标文件
    async fn find_synced_and_link_user(
        &self,
        src_file: &FileModel,
        target_storage_type: &str,
        file_ref: &FileRefModel,
        transfer_tags: bool,
    ) -> FileResult<Option<(FileModel, FileRefModel)>> {
        // 按用户维度查找 lineage：只看当前用户自己的关系记录
        let existing_file_id: Option<u64> = sqlx::query_scalar(&format!(
            "SELECT fl.dst_file_id FROM {} fl \
             INNER JOIN {} f ON fl.dst_file_id=f.id \
             WHERE fl.src_file_id=? AND fl.rel_type=? AND fl.status=? AND f.storage_type=? \
             AND fl.user_id=? AND fl.app_id=? LIMIT 1",
            FileLineageModel::table_name(),
            FileModel::table_name(),
        ))
        .bind(src_file.id)
        .bind(FileLineageRelType::OssSync as i8)
        .bind(FileLineageStatus::Normal as i8)
        .bind(target_storage_type)
        .bind(file_ref.user_id)
        .bind(file_ref.app_id)
        .fetch_optional(&self.helper.db)
        .await?;

        let Some(target_file_id) = existing_file_id else {
            return Ok(None);
        };
        let Some(target_file) = self.helper.find_file_by_id(target_file_id).await? else {
            return Ok(None);
        };

        let source_tags = if transfer_tags {
            self.data_dao
                .helper
                .get_file_tag_names_for_user(src_file.id, file_ref.user_id, file_ref.app_id)
                .await?
        } else {
            vec![]
        };
        let source_tag_refs: Vec<&str> = source_tags.iter().map(String::as_str).collect();

        // 用户已关联
        if let Some(fu) = self
            .helper
            .find_file_ref(
                file_ref.user_id,
                file_ref.app_id,
                target_file.id,
                FileUserStatus::Normal,
            )
            .await?
        {
            if !source_tag_refs.is_empty() {
                self.tag_dao
                    .batch_add_tags(
                        target_file.id,
                        file_ref.user_id,
                        file_ref.app_id,
                        &source_tag_refs,
                        None,
                    )
                    .await?;
            }
            return Ok(Some((target_file, fu)));
        }

        // 创建 file_ref
        let now = now_time()?;
        let mut tx = self.helper.db.begin().await?;
        let tx_result: FileResult<()> = async {
            Insert::<_, FileRefModel>::new()
                .set(FileRefModel::USER_ID, file_ref.user_id)
                .set(FileRefModel::ADD_USER_ID, file_ref.add_user_id)
                .set(FileRefModel::APP_ID, file_ref.app_id)
                .set(FileRefModel::FILE_ID, target_file.id)
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
                    target_file.id,
                    0,
                    file_ref.user_id,
                    "find_synced_and_link_user: created file_ref",
                    Some(&mut tx),
                )
                .await;
            if !source_tag_refs.is_empty() {
                self.tag_dao
                    .batch_add_tags(
                        target_file.id,
                        file_ref.user_id,
                        file_ref.app_id,
                        &source_tag_refs,
                        Some(&mut tx),
                    )
                    .await?;
            }
            Ok(())
        }
        .await;

        match tx_result {
            Ok(_) => {
                tx.commit().await?;
                let new_fu = self
                    .helper
                    .find_file_ref(
                        file_ref.user_id,
                        file_ref.app_id,
                        target_file.id,
                        FileUserStatus::Normal,
                    )
                    .await?
                    .ok_or_else(|| FileError::System(fluent_message!("file-user-create-error")))?;
                Ok(Some((target_file, new_fu)))
            }
            Err(e) => {
                if let Err(rb_err) = tx.rollback().await {
                    warn!("find_synced_and_link_user: rollback failed: {}", rb_err);
                }
                Err(e)
            }
        }
    }

    // ==================== 操作方法 1: OSS 同步到本地 ====================
    /// 将用户的 OSS 文件同步到本地存储
    ///
    /// 通过 `FileOpContext` 传入参数：
    /// - 必须: `file_ref`
    /// - 可选: `file` / `oss_provider`（未提供时自动从 DB + 注册表解析）
    ///
    /// # 参数
    /// - `storage_type`: 本地存储类型，如 `STORAGE_TYPE_LOCAL_PUBLIC` 或 `STORAGE_TYPE_LOCAL_PRIVATE`
    pub async fn sync_oss_to_local(
        &self,
        ctx: FileOpContext<'_>,
        storage_type: &str,
        env_data: Option<&RequestEnv>,
    ) -> FileResult<(FileModel, FileRefModel)> {
        let file_ref = ctx.file_ref;
        let file = ctx.file().await?;
        let oss_provider = ctx.oss_provider().await?;

        if file.is_local() {
            return Err(FileError::Param(fluent_message!("file-must-be-oss-type")));
        }

        // 获取 OSS 文件记录（用于后续下载）
        let file_oss = self
            .helper
            .find_file_oss_by_file_id(file.id)
            .await?
            .ok_or_else(|| FileError::Param(fluent_message!("file-oss-not-found")))?;

        // 通过 lineage 检查是否已有对应 storage_type 的本地副本
        if let Some((local_file, fu)) = self
            .find_synced_and_link_user(file, storage_type, file_ref, true)
            .await?
        {
            return Ok((local_file, fu));
        }

        let sync_lock_key = format!("{}:{}", file.id, storage_type);
        let sync_lock_guard = {
            let ttl = Duration::from_secs(self.helper.config.sync_lock_timeout);
            match self
                .helper
                .sync_locker
                .try_lock_with_watchdog(
                    &sync_lock_key,
                    ttl,
                    WatchdogConfig {
                        max_duration: Some(ttl),
                        ..Default::default()
                    },
                )
                .await
            {
                Ok(guard) => guard,
                Err(DistLockError::AcquireFailed { .. }) => {
                    return Err(FileError::System(fluent_message!("file-sync-in-progress")));
                }
                Err(e) => return Err(FileError::Lock(e)),
            }
        };
        let result: FileResult<(FileModel, FileRefModel)> = async {
            let oss_ext = crate::common::extract_extension(Some(&file.origin_name));
            let oss_prefix = format!("{}_{}_oss", file_ref.app_id, file_ref.user_id);
            let (_rel_path, full_path) = self
                .helper
                .create_new_file(storage_type, &oss_prefix, oss_ext)
                .await?;

            // 流式下载保存到本地
            {
                use futures_util::StreamExt;
                use tokio::fs::File;
                use tokio::io::AsyncWriteExt;

                let dl_result = oss_provider.download_stream(&file_oss, None, None).await?;

                let mut stream = match dl_result {
                    crate::common::OssDownloadResult::RangeSupported(s) => s,
                    crate::common::OssDownloadResult::FullStreamOnly(s) => s,
                };

                let mut local_out = File::create(&full_path).await?;
                while let Some(chunk_result) = stream.next().await {
                    let chunk = chunk_result?;
                    local_out.write_all(&chunk.data).await?;
                }
                local_out.flush().await?;
            }

            // 创建本地文件记录
            let source_tags = self
                .data_dao
                .helper
                .get_file_tag_names_for_user(file.id, file_ref.user_id, file_ref.app_id)
                .await?;
            let source_tag_refs: Vec<&str> = source_tags.iter().map(String::as_str).collect();
            let full_path_str = full_path
                .to_str()
                .ok_or_else(|| FileError::System(fluent_message!("file-path-invalid-utf8")))?;
            let (local_file, local_file_ref) = self
                .create_from_local_file(
                    full_path_str,
                    file_ref.user_id,
                    file_ref.add_user_id,
                    file_ref.app_id,
                    storage_type,
                    Some(&file_ref.file_name),
                    LocalFileMode::Move,
                    LocalFileSource::Plaintext, // OSS 下载的文件是明文，若目标为 CRYPTO 需加密
                    true,
                    &source_tag_refs,
                    Some(file_ref.expire_time),
                    env_data,
                )
                .await?;

            // 记录 OSS→本地 派生关系
            if let Err(e) = self
                .file_ops
                .add_lineage(
                    file_ref.user_id,
                    file_ref.app_id,
                    file.id,
                    local_file.id,
                    FileLineageRelType::OssSync as i8,
                )
                .await
            {
                warn!("sync_oss_to_local: add_lineage failed: {}", e);
            }

            Ok((local_file, local_file_ref))
        }
        .await;
        sync_lock_guard.release().await?;
        if result.is_ok() {
            self.logger
                .add(
                    &LogFileSync {
                        action: "sync_oss_to_local",
                        user_id: file_ref.user_id,
                        file_id: file.id,
                        storage_type: &file.storage_type,
                    },
                    Some(file.id),
                    Some(file_ref.add_user_id),
                    None,
                    env_data,
                )
                .await;
        }

        result
    }

    // ==================== 操作方法 2: 本地文件同步到 OSS ====================
    /// 将用户的本地文件同步到 OSS
    ///
    /// 通过 `FileOpContext` 传入参数：
    /// - 必须: `file_ref`
    /// - 可选: `file` / `oss_provider`（未提供时自动从 DB + 注册表解析）
    ///
    /// **注意**: 本地文件类型必须为 `STORAGE_TYPE_LOCAL_PUBLIC` 或 `STORAGE_TYPE_LOCAL_PRIVATE`，
    /// 加密文件（`STORAGE_TYPE_LOCAL_CRYPTO`）不允许直接上传到 OSS，请使用
    /// [`sync_local_to_oss_auto`] 自动解密后上传。
    pub async fn sync_local_to_oss(
        &self,
        ctx: FileOpContext<'_>,
        storage_type: &str,
        env_data: Option<&RequestEnv>,
    ) -> FileResult<(FileModel, FileRefModel)> {
        let file_ref = ctx.file_ref;
        let file = ctx.file().await?;

        if !file.is_local() {
            return Err(FileError::Param(fluent_message!("file-must-be-local-type")));
        }

        // 加密文件：解密到临时文件后直接上传 OSS，不产生中间记录
        if file.storage_type == FileModel::STORAGE_TYPE_LOCAL_CRYPTO {
            let local = self
                .helper
                .find_file_local_by_file_id(file.id)
                .await?
                .ok_or_else(|| FileError::Param(fluent_message!("file-local-not-found")))?;
            let ext = crate::common::extract_extension(Some(&file.origin_name));
            let temp_path = self
                .helper
                .decrypt_to_temp_file(&local.local_path, ext)
                .await
                .map_err(|e| {
                    FileError::System(lsys_core::fluent_message!("file-decrypt-error", e))
                })?;
            // 根据目标 storage_type 获取对应的 OSS provider
            let oss_provider = self.oss_config.resolve_provider(storage_type).await?;
            let upload_info = UploadFileInfo {
                file_name: &file_ref.file_name,
                file_md5: &file.file_md5,
                file_size: file.file_size,
                content_type: &file.content_type,
            };
            let temp_path_str = temp_path
                .to_str()
                .ok_or_else(|| FileError::System(fluent_message!("file-path-invalid-utf8")))?;
            let oss_result = oss_provider
                .upload_from_local(temp_path_str, &upload_info)
                .await;
            if let Err(e) = tokio::fs::remove_file(&temp_path).await {
                warn!(
                    "sync_local_to_oss(crypto): failed to remove temp file {:?}: {}",
                    temp_path, e
                );
            }
            let oss_result = oss_result?;
            let source_tags = self
                .data_dao
                .helper
                .get_file_tag_names_for_user(file.id, file_ref.user_id, file_ref.app_id)
                .await?;
            let source_tag_refs: Vec<&str> = source_tags.iter().map(String::as_str).collect();
            let (oss_file, oss_file_ref) = self
                .create_from_oss(
                    oss_result,
                    file_ref.user_id,
                    file_ref.add_user_id,
                    file_ref.app_id,
                    storage_type,
                    oss_provider.as_ref(),
                    &source_tag_refs,
                    Some(file_ref.expire_time),
                    env_data,
                    None,
                )
                .await?;
            if let Err(e) = self
                .file_ops
                .add_lineage(
                    file_ref.user_id,
                    file_ref.app_id,
                    file.id,
                    oss_file.id,
                    FileLineageRelType::OssSync as i8,
                )
                .await
            {
                warn!("sync_local_to_oss(crypto): add_lineage failed: {}", e);
            }
            return Ok((oss_file, oss_file_ref));
        }

        // 通过 lineage 检查是否已有对应 storage_type 的 OSS 副本
        if let Some((oss_file, fu)) = self
            .find_synced_and_link_user(file, storage_type, file_ref, false)
            .await?
        {
            return Ok((oss_file, fu));
        }

        let sync_lock_key = format!("{}:{}", file.id, storage_type);
        let sync_lock_guard = {
            let ttl = Duration::from_secs(self.helper.config.sync_lock_timeout);
            match self
                .helper
                .sync_locker
                .try_lock_with_watchdog(
                    &sync_lock_key,
                    ttl,
                    WatchdogConfig {
                        max_duration: Some(ttl),
                        ..Default::default()
                    },
                )
                .await
            {
                Ok(guard) => guard,
                Err(DistLockError::AcquireFailed { .. }) => {
                    return Err(FileError::System(fluent_message!("file-sync-in-progress")));
                }
                Err(e) => return Err(FileError::Lock(e)),
            }
        };

        // 根据目标 storage_type 获取对应的 OSS provider
        let oss_provider = self.oss_config.resolve_provider(storage_type).await?;

        // 上传本地文件到 OSS
        let result: FileResult<(FileModel, FileRefModel)> = async {
            let local = self
                .helper
                .find_file_local_by_file_id(file.id)
                .await?
                .ok_or_else(|| FileError::Param(fluent_message!("file-local-not-found")))?;

            let full_path = self
                .helper
                .get_full_local_path(&file.storage_type, &local.local_path)
                .await?;

            let upload_info = UploadFileInfo {
                file_name: &file_ref.file_name,
                file_md5: &file.file_md5,
                file_size: file.file_size,
                content_type: &file.content_type,
            };
            let oss_result = oss_provider
                .upload_from_local(full_path.to_str().unwrap_or(""), &upload_info)
                .await?;

            let source_tags = self
                .data_dao
                .helper
                .get_file_tag_names_for_user(file.id, file_ref.user_id, file_ref.app_id)
                .await?;
            let source_tag_refs: Vec<&str> = source_tags.iter().map(String::as_str).collect();
            let (oss_file, oss_file_ref) = self
                .create_from_oss(
                    oss_result,
                    file_ref.user_id,
                    file_ref.add_user_id,
                    file_ref.app_id,
                    storage_type,
                    oss_provider.as_ref(),
                    &source_tag_refs,
                    Some(file_ref.expire_time),
                    env_data,
                    None,
                )
                .await?;

            // 记录本地→OSS 派生关系
            if let Err(e) = self
                .file_ops
                .add_lineage(
                    file_ref.user_id,
                    file_ref.app_id,
                    file.id,
                    oss_file.id,
                    FileLineageRelType::OssSync as i8,
                )
                .await
            {
                warn!("sync_local_to_oss: add_lineage failed: {}", e);
            }

            Ok((oss_file, oss_file_ref))
        }
        .await;
        sync_lock_guard.release().await?;
        if result.is_ok() {
            self.logger
                .add(
                    &LogFileSync {
                        action: "sync_local_to_oss",
                        user_id: file_ref.user_id,
                        file_id: file.id,
                        storage_type,
                    },
                    Some(file.id),
                    Some(file_ref.add_user_id),
                    None,
                    env_data,
                )
                .await;
        }

        result
    }
}
