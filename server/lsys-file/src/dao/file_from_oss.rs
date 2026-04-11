use lsys_core::db::{Insert, QueryBuilderExt, TableMeta, Update};
use lsys_core::fluent_message;
use lsys_core::utils::{RequestEnv, now_time};

use super::file_helpers::FileHelper;
use super::logger::*;
use super::*;
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
        env_data: Option<&RequestEnv>,
    ) -> FileResult<(FileModel, FileUserModel)> {
        use tracing::info;
        info!(
            "create_from_local_upload_oss: starting, local_path={}, user_id={}, storage_type={}",
            local_path, user_id, storage_type
        );

        let path = std::path::PathBuf::from(local_path);
        if !path.exists() {
            return Err(FileError::Param(fluent_message!("file-not-found")));
        }

        // 计算文件 MD5 和大小，以及修改时间
        use std::time::UNIX_EPOCH;

        let file_md5 = self.helper.compute_file_md5(&path).await?;
        let metadata = tokio::fs::metadata(&path).await;
        let file_size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
        let modify_time = metadata
            .as_ref()
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

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
            .find_existing_file(storage_type, &file_md5)
            .await?
        {
            info!(
                "create_from_local_upload_oss: file already exists in storage, id={}",
                existing.id
            );
            // 文件已存在，构建临时的 OssResult 并调用存在处理函数
            let temp_oss_result = OssResult {
                object_key: String::new(),
                bucket: String::new(),
                object_url: String::new(),
                file_md5: file_md5.clone(),
                file_size: Some(file_size),
                file_name: Some(actual_file_name.clone()),
                content_type: Some(actual_content_type.clone()),
                source_url: None,
                modify_time: Some(modify_time),
                region: None,
                local_file_id: None,
            };
            return self
                .handle_existing_file(
                    existing,
                    &temp_oss_result,
                    user_id,
                    add_user_id,
                    app_id,
                    tag_names,
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
            None,
            tag_names,
            env_data,
        )
        .await
    }

    // ==================== 创建方法 1: OSS 远程文件 ====================
    /// 处理文件已存在的情况
    /// 当检测到文件已存在时，创建或复用 file_user 记录
    #[allow(clippy::too_many_arguments)]
    async fn handle_existing_file(
        &self,
        existing: FileModel,
        oss_result: &OssResult,
        user_id: u64,
        add_user_id: u64,
        app_id: u64,
        tag_names: &[&str],
    ) -> FileResult<(FileModel, FileUserModel)> {
        use tracing::info;

        info!(
            "handle_existing_file: existing file found, id={}",
            existing.id
        );

        // 检查 file_user 是否已存在
        if let Some(fu) = self
            .helper
            .find_file_user(user_id, app_id, existing.id, FileUserStatus::Normal)
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
            return Ok((existing, fu));
        }

        info!("handle_existing_file: creating file_user link to existing file");

        let now = now_time()?;
        let mut tx = self.helper.db.begin().await?;
        let tx_result: FileResult<()> = async {
            Insert::<_, FileUserModel>::new()
                .set(FileUserModel::USER_ID, user_id)
                .set(FileUserModel::ADD_USER_ID, add_user_id)
                .set(FileUserModel::APP_ID, app_id)
                .set(FileUserModel::FILE_ID, existing.id)
                .set(FileUserModel::STATUS, FileUserStatus::Normal as i8)
                .set(
                    FileUserModel::SOURCE_URL,
                    oss_result.source_url.as_deref().unwrap_or(""),
                )
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
                    "handle_existing_file: existing file, created file_user",
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
                let file_user = self
                    .helper
                    .find_file_user(user_id, app_id, existing.id, FileUserStatus::Normal)
                    .await?
                    .ok_or_else(|| FileError::System(fluent_message!("file-user-create-error")))?;
                Ok((existing, file_user))
            }
            Err(e) => {
                let _ = tx.rollback().await;
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
        copy_file_id: Option<u64>,
        tag_names: &[&str],
        env_data: Option<&RequestEnv>,
    ) -> FileResult<(FileModel, FileUserModel)> {
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
                    FileModel::FILE_NAME,
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
                .set(FileModel::COPY_FILE_ID, copy_file_id.unwrap_or(0))
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
                .set(
                    FileOssModel::LOCAL_FILE_ID,
                    oss_result.local_file_id.unwrap_or(0),
                )
                .set(FileOssModel::LAST_ERROR, "")
                .execute(&mut *tx)
                .await?;

            Insert::<_, FileUserModel>::new()
                .set(FileUserModel::USER_ID, user_id)
                .set(FileUserModel::ADD_USER_ID, add_user_id)
                .set(FileUserModel::APP_ID, app_id)
                .set(FileUserModel::FILE_ID, file_id)
                .set(FileUserModel::STATUS, FileUserStatus::Normal as i8)
                .set(
                    FileUserModel::SOURCE_URL,
                    oss_result.source_url.as_deref().unwrap_or(""),
                )
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
                let _ = tx.rollback().await;
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

        Ok((file, file_user))
    }

    /// 从 OSS 结果创建文件记录
    ///
    /// 检测文件是否已存在：
    /// - 如果存在，复用文件并创建 file_user 关联
    /// - 如果不存在，创建新的文件记录
    #[allow(clippy::too_many_arguments)]
    pub async fn create_from_oss(
        &self,
        oss_result: OssResult,
        user_id: u64,
        add_user_id: u64,
        app_id: u64,
        storage_type: &str,
        copy_file_id: Option<u64>,
        tag_names: &[&str],
        env_data: Option<&RequestEnv>,
    ) -> FileResult<(FileModel, FileUserModel)> {
        use tracing::info;

        info!(
            "create_from_oss: starting, user_id={}, storage_type={}, file_md5={}",
            user_id, storage_type, &oss_result.file_md5
        );

        // 检测是否存在
        if let Some(existing) = self
            .helper
            .find_existing_file(storage_type, &oss_result.file_md5)
            .await?
        {
            info!("create_from_oss: existing file found, id={}", existing.id);
            // 文件已存在，调用处理存在文件的函数
            self.handle_existing_file(
                existing,
                &oss_result,
                user_id,
                add_user_id,
                app_id,
                tag_names,
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
                copy_file_id,
                tag_names,
                env_data,
            )
            .await
        }
    }

    // ==================== 操作方法 1: OSS 同步到本地 ====================
    /// 将用户的 OSS 文件同步到本地存储
    ///
    /// 通过 `FileOpContext` 传入参数：
    /// - 必须: `file_user`
    /// - 可选: `file` / `oss_provider`（未提供时自动从 DB + 注册表解析）
    pub async fn sync_oss_to_local(
        &self,
        ctx: FileOpContext<'_>,
        env_data: Option<&RequestEnv>,
    ) -> FileResult<(FileModel, FileUserModel)> {
        let file_user = ctx.file_user;
        let file = ctx.file().await?;
        let oss_provider = ctx.oss_provider().await?;

        if file.is_local() {
            return Err(FileError::Param(fluent_message!("file-must-be-oss-type")));
        }

        // 获取 OSS 文件记录
        let file_oss = self
            .helper
            .find_file_oss_by_file_id(file.id)
            .await?
            .ok_or_else(|| FileError::Param(fluent_message!("file-oss-not-found")))?;

        // 检查是否已有本地副本：通过 lst_file_oss.local_file_id 判断
        if file_oss.local_file_id > 0 {
            // 找到了对应的本地文件 ID，查询本地文件记录
            if let Some(local_file) = self.helper.find_file_by_id(file_oss.local_file_id).await? {
                let now = now_time()?;
                let source_tags = self.data_dao.get_file_tag_names_for_user(file.id, file_user.user_id, file_user.app_id).await?;
                let source_tag_refs: Vec<&str> = source_tags.iter().map(String::as_str).collect();

                // 检查当前用户是否已有该本地文件的 file_user 关联
                if let Some(fu) = self
                    .helper
                    .find_file_user(
                        file_user.user_id,
                        file_user.app_id,
                        local_file.id,
                        FileUserStatus::Normal,
                    )
                    .await?
                {
                    // 当前用户已关联，直接补充 tag 后返回
                    self.tag_dao
                        .batch_add_tags(
                            local_file.id,
                            file_user.user_id,
                            file_user.app_id,
                            &source_tag_refs,
                            None,
                        )
                        .await?;
                    return Ok((local_file, fu));
                }

                // 当前用户未关联：创建 file_user 记录，使当前用户可见该本地文件
                let mut tx = self.helper.db.begin().await?;
                let tx_result: FileResult<()> = async {
                    Insert::<_, FileUserModel>::new()
                        .set(FileUserModel::USER_ID, file_user.user_id)
                        .set(FileUserModel::ADD_USER_ID, file_user.add_user_id)
                        .set(FileUserModel::APP_ID, file_user.app_id)
                        .set(FileUserModel::FILE_ID, local_file.id)
                        .set(FileUserModel::STATUS, FileUserStatus::Normal as i8)
                        .set(FileUserModel::SOURCE_URL, "")
                        .set(FileUserModel::SOURCE_MD5, "")
                        .set(FileUserModel::ADD_TIME, now)
                        .set(FileUserModel::DELETE_TIME, 0u64)
                        .execute(&mut *tx)
                        .await?;
                    self.log_dao
                        .add(
                            local_file.id,
                            0,
                            file_user.user_id,
                            "sync_oss_to_local: existing local file, created file_user",
                            Some(&mut tx),
                        )
                        .await;
                    self.tag_dao
                        .batch_add_tags(
                            local_file.id,
                            file_user.user_id,
                            file_user.app_id,
                            &source_tag_refs,
                            Some(&mut tx),
                        )
                        .await?;
                    Ok(())
                }
                .await;
                match tx_result {
                    Ok(_) => {
                        tx.commit().await?;
                        // 查询刚创建的 file_user
                        let new_file_user = self
                            .helper
                            .find_file_user(
                                file_user.user_id,
                                file_user.app_id,
                                local_file.id,
                                FileUserStatus::Normal,
                            )
                            .await?
                            .ok_or_else(|| {
                                FileError::System(fluent_message!("file-user-create-error"))
                            })?;
                        return Ok((local_file, new_file_user));
                    }
                    Err(e) => {
                        let _ = tx.rollback().await;
                        return Err(e);
                    }
                }
            }
        }

        // 下载 OSS 文件到本地

        let oss_ext = crate::common::extract_extension(Some(&file.file_name));
        let oss_prefix = format!("{}_{}_oss", file_user.app_id, file_user.user_id);
        let storage_type = FileModel::STORAGE_TYPE_LOCAL_PUBLIC;
        let (_rel_path, full_path) = self
            .helper
            .create_new_file(storage_type, &oss_prefix, oss_ext)
            .await?;
        oss_provider
            .download_to_local(&file_oss, full_path.to_str().unwrap_or(""))
            .await?;

        // 用创建方法4创建本地文件记录
        let source_tags = self.data_dao.get_file_tag_names_for_user(file.id, file_user.user_id, file_user.app_id).await?;
        let source_tag_refs: Vec<&str> = source_tags.iter().map(String::as_str).collect();
        let (local_file, local_file_user) = self
            .create_from_local_file(
                full_path.to_str().unwrap_or(""),
                file_user.user_id,
                file_user.add_user_id,
                file_user.app_id,
                storage_type,
                Some(&file.file_name),
                LocalFileMode::Move,
                Some(file.id),
                Some(file.id),
                &source_tag_refs,
                env_data,
            )
            .await?;

        Update::<_, FileOssModel>::new()
            .set(FileOssModel::LOCAL_FILE_ID, local_file.id)
            .execute(&self.helper.db, |qb| {
                qb.push_where().field_eq("id", file_oss.id);
            })
        .await?;

        self.logger
            .add(
                &LogFileSync {
                    action: "sync_oss_to_local",
                    user_id: file_user.user_id,
                    file_id: file.id,
                    storage_type: &file.storage_type,
                },
                Some(file.id),
                Some(file_user.add_user_id),
                None,
                env_data,
            )
            .await;

        Ok((local_file, local_file_user))
    }

    // ==================== 操作方法 2: 本地文件上传到 OSS ====================
    /// 将用户的本地文件上传到 OSS
    ///
    /// 通过 `FileOpContext` 传入参数：
    /// - 必须: `file_user`
    /// - 可选: `file` / `oss_provider`（未提供时自动从 DB + 注册表解析）
    ///
    /// **注意**: 本地文件类型必须为 `STORAGE_TYPE_LOCAL_PUBLIC` 或 `STORAGE_TYPE_LOCAL_PRIVATE`，
    /// 加密文件（`STORAGE_TYPE_LOCAL_CRYPTO`）不允许直接上传到 OSS，请使用
    /// [`upload_local_to_oss_auto_convert`] 自动解密后上传。
    pub async fn upload_local_to_oss(
        &self,
        ctx: FileOpContext<'_>,
        storage_type: &str,
        env_data: Option<&RequestEnv>,
    ) -> FileResult<(FileModel, FileUserModel)> {
        let file_user = ctx.file_user;
        let file = ctx.file().await?;
        let oss_provider = ctx.oss_provider().await?;

        if !file.is_local() {
            return Err(FileError::Param(fluent_message!("file-must-be-local-type")));
        }

        // 加密文件不能直接上传到 OSS，需要先转换为 PUBLIC 或 PRIVATE
        if file.storage_type == FileModel::STORAGE_TYPE_LOCAL_CRYPTO {
            return Err(FileError::Param(fluent_message!(
                "file-crypto-cannot-upload-oss"
            )));
        }

        // 检查是否已有 OSS 副本
        let existing_oss = sqlx::query_as::<_, FileOssModel>(
            &format!(
                "SELECT fo.* FROM {} fo INNER JOIN {} f ON fo.file_id=f.id WHERE fo.local_file_id=? AND f.storage_type=? LIMIT 1",
                FileOssModel::table_name(),
                FileModel::table_name(),
            )
        )
        .bind(file.id)
        .bind(storage_type)
        .fetch_optional(&self.helper.db)
        .await?;

        if let Some(oss) = existing_oss
            && let Some(oss_file) = self.helper.find_file_by_id(oss.file_id).await?
        {
            // 检查当前用户是否已有该 OSS 文件的 file_user 关联
            if let Some(fu) = self
                .helper
                .find_file_user(
                    file_user.user_id,
                    file_user.app_id,
                    oss_file.id,
                    FileUserStatus::Normal,
                )
                .await?
            {
                return Ok((oss_file, fu));
            }
            // 当前用户未关联：创建 file_user 记录
            let now = now_time()?;
            let mut tx = self.helper.db.begin().await?;
            let tx_result: FileResult<()> = async {
                Insert::<_, FileUserModel>::new()
                    .set(FileUserModel::USER_ID, file_user.user_id)
                    .set(FileUserModel::ADD_USER_ID, file_user.add_user_id)
                    .set(FileUserModel::APP_ID, file_user.app_id)
                    .set(FileUserModel::FILE_ID, oss_file.id)
                    .set(FileUserModel::STATUS, FileUserStatus::Normal as i8)
                    .set(FileUserModel::SOURCE_URL, "")
                    .set(FileUserModel::SOURCE_MD5, "")
                    .set(FileUserModel::ADD_TIME, now)
                    .set(FileUserModel::DELETE_TIME, 0u64)
                    .execute(&mut *tx)
                    .await?;
                self.log_dao
                    .add(
                        oss_file.id,
                        0,
                        file_user.user_id,
                        "upload_local_to_oss: existing oss file, created file_user",
                        Some(&mut tx),
                    )
                    .await;
                Ok(())
            }
            .await;
            match tx_result {
                Ok(_) => {
                    tx.commit().await?;
                    let new_fu = self
                        .helper
                        .find_file_user(
                            file_user.user_id,
                            file_user.app_id,
                            oss_file.id,
                            FileUserStatus::Normal,
                        )
                        .await?
                        .ok_or_else(|| {
                            FileError::System(fluent_message!("file-user-create-error"))
                        })?;
                    return Ok((oss_file, new_fu));
                }
                Err(e) => {
                    let _ = tx.rollback().await;
                    return Err(e);
                }
            }
        }

        // 获取本地文件路径
        let local = self
            .helper
            .find_file_local_by_file_id(file.id)
            .await?
            .ok_or_else(|| FileError::Param(fluent_message!("file-local-not-found")))?;

        let full_path = self
            .helper
            .get_full_local_path(&file.storage_type, &local.local_path).await?;

        // 上传到 OSS
        let upload_info = UploadFileInfo {
            file_name: &file.file_name,
            file_md5: &file.file_md5,
            file_size: file.file_size,
            content_type: &file.content_type,
        };
        let mut oss_result = oss_provider
            .upload_from_local(full_path.to_str().unwrap_or(""), &upload_info)
            .await?;
        oss_result.local_file_id = Some(file.id);

        // 通过创建方法1创建OSS记录
        let source_tags = self.data_dao.get_file_tag_names_for_user(file.id, file_user.user_id, file_user.app_id).await?;
        let source_tag_refs: Vec<&str> = source_tags.iter().map(String::as_str).collect();
        let (oss_file, oss_file_user) = self
            .create_from_oss(
                oss_result,
                file_user.user_id,
                file_user.add_user_id,
                file_user.app_id,
                storage_type,
                Some(file.id),
                &source_tag_refs,
                env_data,
            )
            .await?;

        self.logger
            .add(
                &LogFileSync {
                    action: "upload_local_to_oss",
                    user_id: file_user.user_id,
                    file_id: file.id,
                    storage_type,
                },
                Some(file.id),
                Some(file_user.add_user_id),
                None,
                env_data,
            )
            .await;

        Ok((oss_file, oss_file_user))
    }

    /// 将本地文件上传到 OSS，自动处理加密文件的转换
    ///
    /// 如果本地文件类型为 `STORAGE_TYPE_LOCAL_CRYPTO`，会先将其转换为
    /// `STORAGE_TYPE_LOCAL_PRIVATE`（解密），然后再上传转换后的文件到 OSS。
    ///
    /// 对于 `STORAGE_TYPE_LOCAL_PUBLIC` 和 `STORAGE_TYPE_LOCAL_PRIVATE`，直接上传。
    pub async fn upload_local_to_oss_auto(
        &self,
        ctx: FileOpContext<'_>,
        storage_type: &str,
        env_data: Option<&RequestEnv>,
    ) -> FileResult<(FileModel, FileUserModel)> {
        let file_user = ctx.file_user;
        let file = ctx.file().await?;

        if !file.is_local() {
            return Err(FileError::Param(fluent_message!("file-must-be-local-type")));
        }

        // 如果是加密文件，先转换为 PRIVATE 类型
        if file.storage_type == FileModel::STORAGE_TYPE_LOCAL_CRYPTO {
            let (converted_file, converted_file_user) = self
                .convert_storage_type(file_user, FileModel::STORAGE_TYPE_LOCAL_PRIVATE, env_data)
                .await?;

            let new_ctx = self
                .create_context(&converted_file_user)
                .with_file(&converted_file)?;

            return self
                .upload_local_to_oss(new_ctx, storage_type, env_data)
                .await;
        }

        // 非加密类型，直接上传
        let new_ctx = self.create_context(file_user).with_file(file)?;

        self.upload_local_to_oss(new_ctx, storage_type, env_data)
            .await
    }
}
