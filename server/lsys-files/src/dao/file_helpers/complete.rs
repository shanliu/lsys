use lsys_core::db::{SqlQuote, SqlSuffix, TableMeta, Update};
use lsys_core::sql_format;
use lsys_core::{fluent_message, now_time};

use super::super::{FileError, FileResult};
use super::FileHelper;
use crate::common::get_content_type;
use crate::model::*;

impl FileHelper {
    /// 通过 storage_type + MD5 查询已存在的正常状态文件记录
    pub async fn find_existing_file(
        &self,
        storage_type: &str,
        file_md5: &str,
    ) -> FileResult<Option<FileModel>> {
        let row = sqlx::query_as::<_, FileModel>(&sql_format!(
            "SELECT * FROM {} WHERE storage_type={} AND file_md5={} AND status={} LIMIT 1",
            FileModel::table_name(),
            storage_type,
            file_md5,
            FileStatus::Normal as i8
        ))
        .fetch_optional(&self.db)
        .await?;
        Ok(row)
    }

    /// 通过指定文件完成 FILE 跟 FILE_LOCAL 的记录
    /// 返回 Some(other_file) 表示找到了已有的相同MD5文件, None 表示用新文件完成
    pub async fn complete_file_and_local(
        &self,
        file: &mut FileModel,
        file_local: &mut FileLocalModel,
        new_local_path: &str,
    ) -> FileResult<Option<FileModel>> {
        // 验证前置条件
        if !FileStatus::Unfinished.eq(file.status) {
            return Err(FileError::Param(fluent_message!(
                "file-status-error",
                { "status": &format!("{}", file.status) }
            )));
        }
        // 根据是否分片判断 local_path 的要求
        if file_local.file_chunk_total > 1 {
            // 分片上传：local_path 必须为空（来自下载或合并，新创建的路径通过参数传入）
            if !file_local.local_path.is_empty() {
                return Err(FileError::Param(fluent_message!(
                    "file-local-path-not-empty"
                )));
            }
        } else {
            // 单文件上传：local_path 必须不为空（已在 open_file_write 时创建和设置）
            if file_local.local_path.is_empty() {
                return Err(FileError::Param(fluent_message!(
                    "file-local-path-empty"
                )));
            }
        }
        if file.id != file_local.file_id {
            return Err(FileError::Param(fluent_message!("file-id-mismatch")));
        }

        let full_path = self.get_full_local_path(new_local_path);
        // 计算新文件的 MD5
        let file_md5 = self.compute_file_md5(&full_path).await?;
        let now = now_time()?;

        // 查询是否已存在相同 MD5 的文件 (排除自身)
        let other_file = sqlx::query_as::<_, FileModel>(&sql_format!(
            "SELECT * FROM {} WHERE storage_type={} AND file_md5={} AND status={} AND id!={} LIMIT 1",
            FileModel::table_name(),
            &file.storage_type,
            &file_md5,
            FileStatus::Normal as i8,
            file.id
        ))
        .fetch_optional(&self.db)
        .await?;

        if let Some(ref other) = other_file {
            // 查询 other_file 对应的 file_local
            let other_local = sqlx::query_as::<_, FileLocalModel>(&sql_format!(
                "SELECT * FROM {} WHERE file_id={} LIMIT 1",
                FileLocalModel::table_name(),
                other.id
            ))
            .fetch_optional(&self.db)
            .await?;

            if let Some(ref other_local_rec) = other_local {
                // 检查 other 对应的本地文件是否存在
                let other_full_path = self.get_full_local_path(&other_local_rec.local_path);
                if !other_local_rec.local_path.is_empty()
                    && tokio::fs::metadata(&other_full_path).await.is_ok()
                {
                    // 使用已有文件的信息更新当前记录
                    file.file_size = other.file_size;
                    file.file_md5 = other.file_md5.clone();
                    file.content_type = other.content_type.clone();
                    file.from_user_id = other.from_user_id;
                    file.modify_time = other.modify_time;
                    file.copy_file_id = other.id;
                    file.status = FileStatus::Normal.to();
                    file.change_time = now;
                    file_local.local_path = other_local_rec.local_path.clone();

                    // 更新数据库
                    Update::<FileModel>::new()
                        .set(FileModel::FILE_SIZE, file.file_size)
                        .set(FileModel::FILE_MD5, &file.file_md5)
                        .set(FileModel::CONTENT_TYPE, &file.content_type)
                        .set(FileModel::FROM_USER_ID, file.from_user_id)
                        .set(FileModel::MODIFY_TIME, file.modify_time)
                        .set(FileModel::COPY_FILE_ID, file.copy_file_id)
                        .set(FileModel::STATUS, file.status)
                        .set(FileModel::CHANGE_TIME, file.change_time)
                        .execute(SqlSuffix::Where(&sql_format!("id={}", file.id)), &self.db)
                        .await?;

                    Update::<FileLocalModel>::new()
                        .set(FileLocalModel::LOCAL_PATH, &file_local.local_path)
                        .execute(
                            SqlSuffix::Where(&sql_format!("id={}", file_local.id)),
                            &self.db,
                        )
                        .await?;

                    return Ok(Some(other.clone()));
                }
            }
        }

        // 不存在已有文件, 用新文件完成
        let metadata = tokio::fs::metadata(&full_path).await?;
        let content_type = get_content_type(new_local_path)?;
        let modify_time = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        file.file_size = metadata.len();
        file.file_md5 = file_md5;
        file.content_type = content_type;
        file.modify_time = modify_time;
        file.copy_file_id = 0;
        file.status = FileStatus::Normal.to();
        file.change_time = now;
        file_local.local_path = new_local_path.to_string();

        Update::<FileModel>::new()
            .set(FileModel::FILE_SIZE, file.file_size)
            .set(FileModel::FILE_MD5, &file.file_md5)
            .set(FileModel::CONTENT_TYPE, &file.content_type)
            .set(FileModel::MODIFY_TIME, file.modify_time)
            .set(FileModel::COPY_FILE_ID, file.copy_file_id)
            .set(FileModel::STATUS, file.status)
            .set(FileModel::CHANGE_TIME, file.change_time)
            .execute(SqlSuffix::Where(&sql_format!("id={}", file.id)), &self.db)
            .await?;

        Update::<FileLocalModel>::new()
            .set(FileLocalModel::LOCAL_PATH, &file_local.local_path)
            .execute(
                SqlSuffix::Where(&sql_format!("id={}", file_local.id)),
                &self.db,
            )
            .await?;

        Ok(None)
    }
}
