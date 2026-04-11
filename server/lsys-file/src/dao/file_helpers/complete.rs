use lsys_core::db::{QueryBuilderExt, TableMeta, Update};
use lsys_core::fluent_message;
use lsys_core::utils::now_time;

use super::super::{FileError, FileResult};
use super::FileHelper;
use crate::common::get_content_type;
use crate::model::*;

impl FileHelper {
    /// 通过指定文件完成 FILE 跟 FILE_LOCAL 的记录
    /// 返回 Some(other_file) 表示找到了已有的相同MD5文件, None 表示用新文件完成
    pub(crate) async fn complete_file_and_local(
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

        // 路径验证逻辑说明（支持四种调用场景）：
        // 1. 下载单文件：new_local_path=rel_path(有值), local_path=空
        // 2. 下载多分片：new_local_path=merge_rel(有值), local_path=空, file_chunk_total>1
        // 3. 上传单文件：new_local_path=local_path(同一值，有值), file_chunk_total=0
        // 4. 上传多分片：new_local_path=merge_rel(有值), local_path=空, file_chunk_total>1

        // 检查至少有一个路径提供
        if new_local_path.is_empty() && file_local.local_path.is_empty() {
            return Err(FileError::Param(fluent_message!("file-local-path-empty")));
        }

        // 分片场景（下载或上传）必须通过 new_local_path 参数传入最终路径
        // 此时 local_path 应始终为空
        if file_local.file_chunk_total > 1 && !file_local.local_path.is_empty() {
            return Err(FileError::Param(fluent_message!(
                "file-local-path-not-empty"
            )));
        }
        if file.id != file_local.file_id {
            return Err(FileError::Param(fluent_message!("file-id-mismatch")));
        }

        // 确定要使用的实际路径（优先使用 new_local_path）
        let actual_local_path = if !new_local_path.is_empty() {
            new_local_path.to_string()
        } else {
            file_local.local_path.clone()
        };

        let full_path = self.get_full_local_path(&file.storage_type, &actual_local_path).await?;
        // 计算新文件的 MD5
        let file_md5 = self.compute_file_md5(&full_path).await?;
        let now = now_time()?;

        // 查询是否已存在相同 MD5 的文件 (排除自身)
        let other_file = sqlx::query_as::<_, FileModel>(&format!(
            "SELECT * FROM {} WHERE storage_type=? AND file_md5=? AND status=? AND id!=? LIMIT 1",
            FileModel::table_name()
        ))
        .bind(&file.storage_type)
        .bind(&file_md5)
        .bind(FileStatus::Normal as i8)
        .bind(file.id)
        .fetch_optional(&self.db)
        .await?;

        if let Some(other) = other_file {
            // 查询 other_file 对应的 file_local
            let other_local = sqlx::query_as::<_, FileLocalModel>(&format!(
                "SELECT * FROM {} WHERE file_id=? LIMIT 1",
                FileLocalModel::table_name()
            ))
            .bind(other.id)
            .fetch_optional(&self.db)
            .await?;

            if let Some(other_local_rec) = other_local {
                // 检查 other 对应的本地文件是否存在
                let other_full_path =
                    self.get_full_local_path(&other.storage_type, &other_local_rec.local_path).await?;
                if !other_local_rec.local_path.is_empty()
                    && tokio::fs::metadata(&other_full_path).await.is_ok()
                {
                    // 先克隆一份用于返回
                    let result = other.clone();
                    
                    // 使用已有文件的信息更新当前记录（移动所有权，避免额外克隆）
                    file.file_size = other.file_size;
                    file.file_md5 = other.file_md5;
                    file.content_type = other.content_type;
                    file.from_user_id = other.from_user_id;
                    file.modify_time = other.modify_time;
                    file.copy_file_id = other.id;
                    file.status = FileStatus::Normal as i8;
                    file.change_time = now;
                    file_local.local_path = other_local_rec.local_path;

                    // 在事务中更新数据库
                    let mut tx = self.db.begin().await?;
                    
                    Update::<_, FileModel>::new()
                        .set(FileModel::FILE_SIZE, file.file_size)
                        .set(FileModel::FILE_MD5, &file.file_md5)
                        .set(FileModel::CONTENT_TYPE, &file.content_type)
                        .set(FileModel::FROM_USER_ID, file.from_user_id)
                        .set(FileModel::MODIFY_TIME, file.modify_time)
                        .set(FileModel::COPY_FILE_ID, file.copy_file_id)
                        .set(FileModel::STATUS, file.status)
                        .set(FileModel::CHANGE_TIME, file.change_time)
                        .execute(&mut *tx, |qb| {
                            qb.push_where().field_eq("id", file.id);
                        })
                        .await?;

                    Update::<_, FileLocalModel>::new()
                        .set(FileLocalModel::LOCAL_PATH, &file_local.local_path)
                        .execute(&mut *tx, |qb| {
                            qb.push_where().field_eq("id", file_local.id);
                        })
                        .await?;
                    
                    tx.commit().await?;
                    return Ok(Some(result));
                }
            }
        }

        // 不存在已有文件, 用新文件完成
        let metadata = tokio::fs::metadata(&full_path).await?;
        let content_type = get_content_type(&full_path).await?;
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
        file.status = FileStatus::Normal as i8;
        file.change_time = now;
        file_local.local_path = actual_local_path;

        // 如果存储类型为加密类型，在更新数据库前执行加密并替换明文文件
        if file.storage_type == FileModel::STORAGE_TYPE_LOCAL_CRYPTO {
            let plaintext_full =
                self.get_full_local_path(&file.storage_type, &file_local.local_path).await?;
            let (relative_path, _full_path) = self.encrypt_file(&plaintext_full).await.map_err(|e| {
                FileError::Io(std::io::Error::other(format!(
                    "encrypt completed file failed: {}",
                    e
                )))
            })?;

            // 删除明文文件
            if let Err(e) = tokio::fs::remove_file(&plaintext_full).await {
                tracing::warn!(
                    "complete_file_and_local: remove plaintext file failed: {}",
                    e
                );
            }

            // 更新 local_path 为加密文件路径
            file_local.local_path = relative_path;
        }

        // 在事务中更新数据库
        let mut tx = self.db.begin().await?;
        
        Update::<_, FileModel>::new()
            .set(FileModel::FILE_SIZE, file.file_size)
            .set(FileModel::FILE_MD5, &file.file_md5)
            .set(FileModel::CONTENT_TYPE, &file.content_type)
            .set(FileModel::MODIFY_TIME, file.modify_time)
            .set(FileModel::COPY_FILE_ID, file.copy_file_id)
            .set(FileModel::STATUS, file.status)
            .set(FileModel::CHANGE_TIME, file.change_time)
            .execute(&mut *tx, |qb| {
                qb.push_where().field_eq("id", file.id);
            })
            .await?;

        Update::<_, FileLocalModel>::new()
            .set(FileLocalModel::LOCAL_PATH, &file_local.local_path)
            .execute(&mut *tx, |qb| {
                qb.push_where().field_eq("id", file_local.id);
            })
            .await?;
        
        tx.commit().await?;
        Ok(None)
    }
}
