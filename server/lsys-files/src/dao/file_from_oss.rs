use lsys_core::db::{Insert, SqlQuote, SqlSuffix, TableMeta, Update};
use lsys_core::fluent_message;
use lsys_core::sql_format;
use lsys_core::utils::{now_time, RequestEnv};

use super::file_helpers::FileHelper;
use super::logger::*;
use super::*;
use crate::model::*;

impl FileDao {
    // ==================== 创建方法 1: OSS 远程文件 ====================
    pub async fn create_from_oss(
        &self,
        oss_result: OssResult,
        user_id: u64,
        app_id: u64,
        storage_type: &str,
        tag_names: &[&str],
        env_data: Option<&RequestEnv>,
    ) -> FileResult<FileModel> {
        use tracing::info;

        info!(
            "create_from_oss: starting, user_id={}, storage_type={}, file_md5={}",
            user_id, storage_type, &oss_result.file_md5
        );
        let now = now_time()?;

        // 辅助函数.1: 检测是否存在
        if let Some(existing) = self
            .helper
            .find_existing_file(storage_type, &oss_result.file_md5)
            .await?
        {
            info!("create_from_oss: existing file found, id={}", existing.id);
            // 检查 file_user 是否已存在
            if let Some(_fu) = self
                .helper
                .find_file_user(user_id, app_id, existing.id, FileUserStatus::Normal)
                .await?
            {
                info!("create_from_oss: user already linked to existing file");
                self.log_dao
                    .add(
                        existing.id,
                        0,
                        user_id,
                        "create_from_oss: existing file, user already linked",
                        None,
                    )
                    .await;
                for tag_name in tag_names {
                    self.tag_dao
                        .add_tag(existing.id, user_id, app_id, tag_name, None)
                        .await?;
                }
                return Ok(existing);
            }

            info!("create_from_oss: creating file_user link to existing file");
            // 创建 file_user
            let mut tx = self.helper.db.begin().await?;
            let tx_result: FileResult<()> = async {
                Insert::<_, FileUserModel>::new()
                    .set(FileUserModel::USER_ID, user_id)
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
                        "create_from_oss: existing file, created file_user",
                        Some(&mut tx),
                    )
                    .await;
                for tag_name in tag_names {
                    self.tag_dao
                        .add_tag(existing.id, user_id, app_id, tag_name, Some(&mut tx))
                        .await?;
                }
                Ok(())
            }
            .await;
            match tx_result {
                Ok(_) => {
                    tx.commit().await?;
                    return Ok(existing);
                }
                Err(e) => {
                    let _ = tx.rollback().await;
                    return Err(e);
                }
            }
        }

        // 不存在: 创建新记录
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
                .set(FileModel::FROM_USER_ID, user_id)
                .set(FileModel::ADD_TIME, now)
                .set(FileModel::CHANGE_TIME, 0u64)
                .set(FileModel::COPY_FILE_ID, 0u64)
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
                    user_id,
                    "create_from_oss: new file created",
                    Some(&mut tx),
                )
                .await;

            for tag_name in tag_names {
                self.tag_dao
                    .add_tag(file_id, user_id, app_id, tag_name, Some(&mut tx))
                    .await?;
            }

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

        Ok(file)
    }

    // ==================== 操作方法 1: OSS 同步到本地 ====================
    pub async fn sync_oss_to_local(
        &self,
        file: &FileModel,
        oss_provider: &dyn OssProvider,
        env_data: Option<&RequestEnv>,
    ) -> FileResult<FileModel> {
        if file.storage_type == FileModel::STORAGE_TYPE_LOCAL {
            return Err(FileError::Param(fluent_message!("file-must-be-oss-type")));
        }

        // 检查是否已有本地副本: 查找 file_local 中 oss_file_id 等于当前 OSS 文件 ID 的记录
        if let Some(local) = self.helper.find_file_local_by_oss_file_id(file.id).await? {
            // 找到了从该 OSS 文件同步而来的本地文件记录，返回对应的本地 file
            if let Some(local_file) = self.helper.find_file_by_id(local.file_id).await? {
                return Ok(local_file);
            }
        }

        // 下载 OSS 文件到本地
        let file_oss = self
            .helper
            .find_file_oss_by_file_id(file.id)
            .await?
            .ok_or_else(|| FileError::Param(fluent_message!("file-oss-not-found")))?;

        // 反向查询最早的 file_user 记录, 获取真实 app_id
        let min_file_user = self
            .helper
            .find_min_file_user_by_file_id(file.id)
            .await?
            .ok_or_else(|| FileError::Param(fluent_message!("file-user-not-found")))?;
        let sync_app_id = min_file_user.app_id;

        let oss_ext = FileHelper::extract_extension(&file.file_name);
        let oss_prefix = format!("{}_{}_oss", sync_app_id, file.from_user_id);
        let (_rel_path, full_path) = self
            .helper
            .create_new_file(&oss_prefix, oss_ext)
            .await?;
        oss_provider
            .download_to_local(&file_oss, full_path.to_str().unwrap_or(""))
            .await?;

        // 用创建方法4创建本地文件记录
        let source_tags = self.tag_dao.get_file_tag_names(file.id).await?;
        let source_tag_refs: Vec<&str> = source_tags.iter().map(String::as_str).collect();
        let local_file = self
            .create_from_local_file(
                full_path.to_str().unwrap_or(""),
                file.from_user_id,
                sync_app_id,
                Some(&file.file_name),
                LocalFileMode::Move,
                None,
                &source_tag_refs,
                env_data,
            )
            .await?;

        // 更新 file_local.oss_file_id 指向源 OSS 文件，以便下次查找已有副本
        if let Some(local_rec) = self
            .helper
            .find_file_local_by_file_id(local_file.id)
            .await?
        {
            Update::<_, FileLocalModel>::new()
                .set(FileLocalModel::OSS_FILE_ID, file.id)
                .execute(
                    SqlSuffix::Where(&sql_format!("id={}", local_rec.id)),
                    &self.helper.db,
                )
                .await?;
        }

        self.logger
            .add(
                &LogFileSync {
                    action: "sync_oss_to_local",
                    user_id: file.from_user_id,
                    file_id: file.id,
                    storage_type: &file.storage_type,
                },
                Some(file.id),
                Some(file.from_user_id),
                None,
                env_data,
            )
            .await;

        Ok(local_file)
    }

    // ==================== 操作方法 2: 本地文件上传到 OSS ====================
    pub async fn upload_local_to_oss(
        &self,
        file: &FileModel,
        storage_type: &str,
        oss_provider: &dyn OssProvider,
        env_data: Option<&RequestEnv>,
    ) -> FileResult<FileModel> {
        if file.storage_type != FileModel::STORAGE_TYPE_LOCAL {
            return Err(FileError::Param(fluent_message!("file-must-be-local-type")));
        }

        // 检查是否已有 OSS 副本
        let existing_oss = sqlx::query_as::<_, FileOssModel>(&sql_format!(
            "SELECT fo.* FROM {} fo INNER JOIN {} f ON fo.file_id=f.id WHERE fo.local_file_id={} AND f.storage_type={} LIMIT 1",
            FileOssModel::table_name(),
            FileModel::table_name(),
            file.id,
            storage_type
        ))
        .fetch_optional(&self.helper.db)
        .await?;

        if let Some(oss) = existing_oss {
            if let Some(oss_file) = self.helper.find_file_by_id(oss.file_id).await? {
                return Ok(oss_file);
            }
        }

        // 获取本地文件路径
        let local = self
            .helper
            .find_file_local_by_file_id(file.id)
            .await?
            .ok_or_else(|| FileError::Param(fluent_message!("file-local-not-found")))?;

        let full_path = self.helper.get_full_local_path(&local.local_path);

        // 上传到 OSS
        let mut oss_result = oss_provider
            .upload_from_local(full_path.to_str().unwrap_or(""), file)
            .await?;
        oss_result.local_file_id = Some(file.id);

        // 通过创建方法1创建OSS记录
        let source_tags = self.tag_dao.get_file_tag_names(file.id).await?;
        let source_tag_refs: Vec<&str> = source_tags.iter().map(String::as_str).collect();
        let oss_file = self
            .create_from_oss(
                oss_result,
                file.from_user_id,
                0,
                storage_type,
                &source_tag_refs,
                env_data,
            )
            .await?;

        self.logger
            .add(
                &LogFileSync {
                    action: "upload_local_to_oss",
                    user_id: file.from_user_id,
                    file_id: file.id,
                    storage_type,
                },
                Some(file.id),
                Some(file.from_user_id),
                None,
                env_data,
            )
            .await;

        Ok(oss_file)
    }
}
