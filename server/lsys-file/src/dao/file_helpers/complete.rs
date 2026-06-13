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

        let plaintext_full = self
            .get_full_local_path(&file.storage_type, &actual_local_path)
            .await?;
        // CRYPTO 类型：先加密，再计算 MD5，确保存储的 MD5 对应加密后内容
        let (file_full_path, encrypted_rel) =
            if file.storage_type == FileModel::STORAGE_TYPE_LOCAL_CRYPTO {
                let (rel, full) = self.encrypt_new_file(&plaintext_full).await.map_err(|e| {
                    FileError::Io(std::io::Error::other(format!(
                        "encrypt completed file failed: {}",
                        e
                    )))
                })?;
                (full, Some(rel))
            } else {
                (plaintext_full.clone(), None)
            };
        // 计算新文件的 MD5（CRYPTO 时基于加密文件，保证 MD5 与实际存储内容一致）
        let file_md5 = self.compute_file_md5(&file_full_path).await?;
        let now = now_time()?;

        // 分页 + 本地文件物理校验，排除自身
        let other_file = self
            .find_existing_local_file_exclude(&file.storage_type, &file_md5, file.id)
            .await?;

        if let Some(other) = other_file {
            // 查询 other_file 对应的 file_local（只需 local_path；
            // find_existing_file_exclude 已完成物理校验，不再重复 metadata 检查）
            let other_local = sqlx::query_as::<_, FileLocalModel>(&format!(
                "SELECT * FROM {} WHERE file_id=? LIMIT 1",
                FileLocalModel::table_name()
            ))
            .bind(other.id)
            .fetch_optional(&self.db)
            .await?;

            // find_existing_file_exclude 保证 other 物理文件存在；
            // 若此处记录缺失或路径为空，说明发生了竞态不一致，直接报错。
            let other_local_rec = match other_local {
                Some(rec) if !rec.local_path.is_empty() => rec,
                _ => {
                    return Err(FileError::System(fluent_message!("file-inconsistent-state")));
                }
            };

            {
                // 先克隆一份用于返回
                    let result = other.clone();

                    // 去重时清理本次产生的新文件
                    if encrypted_rel.is_some() {
                        // CRYPTO 场景：删除加密中间文件和明文原始文件
                        if let Err(e) = tokio::fs::remove_file(&file_full_path).await {
                            tracing::warn!(
                                "complete_file_and_local: remove encrypted file on dedup failed: {}",
                                e
                            );
                        }
                        if let Err(e) = tokio::fs::remove_file(&plaintext_full).await {
                            tracing::warn!(
                                "complete_file_and_local: remove plaintext file on dedup failed: {}",
                                e
                            );
                        }
                    } else {
                        // 非 CRYPTO 场景：删除新上传/下载的文件（file_full_path == plaintext_full）
                        if let Err(e) = tokio::fs::remove_file(&file_full_path).await {
                            tracing::warn!(
                                "complete_file_and_local: remove new file on dedup failed: {}",
                                e
                            );
                        }
                    }

                    // 使用已有文件的信息更新当前记录（移动所有权，避免额外克隆）
                    file.file_size = other.file_size;
                    file.file_md5 = other.file_md5;
                    file.content_type = other.content_type;
                    file.from_user_id = other.from_user_id;
                    file.modify_time = other.modify_time;
                    // find_existing_file 已限制 local_path_owner_id=0，去重后固定指向 owner
                    file.local_path_owner_id = other.id;
                    file.status = FileStatus::Normal as i8;
                    file.change_time = now;
                    file_local.local_path = other_local_rec.local_path;

                    // 在事务中更新数据库（CAS: AND status=Unfinished，防止并发重复完成）
                    let mut tx = self.db.begin().await?;

                    let cas_result = Update::<sqlx::MySql, FileModel>::new()
                        .set(FileModel::FILE_SIZE, file.file_size)
                        .set(FileModel::FILE_MD5, &file.file_md5)
                        .set(FileModel::CONTENT_TYPE, &file.content_type)
                        .set(FileModel::FROM_USER_ID, file.from_user_id)
                        .set(FileModel::MODIFY_TIME, file.modify_time)
                        .set(FileModel::LOCAL_PATH_OWNER_ID, file.local_path_owner_id)
                        .set(FileModel::STATUS, file.status)
                        .set(FileModel::CHANGE_TIME, file.change_time)
                        .execute(&mut *tx, |qb| {
                            qb.push_where().field_eq("id", file.id);
                            qb.push_and()
                                .field_eq("status", FileStatus::Unfinished as i8);
                        })
                        .await?;

                    if cas_result.rows_affected() == 0 {
                        // 另一并发调用已完成该文件，直接视为成功
                        if let Err(rb) = tx.rollback().await {
                            tracing::warn!(
                                "complete_file_and_local: dedup CAS rollback failed: {}",
                                rb
                            );
                        }
                        return Ok(None);
                    }

                    Update::<sqlx::MySql, FileLocalModel>::new()
                        .set(FileLocalModel::LOCAL_PATH, &file_local.local_path)
                        .execute(&mut *tx, |qb| {
                            qb.push_where().field_eq("id", file_local.id);
                        })
                        .await?;

                    tx.commit().await?;
                    return Ok(Some(result));
            }
        }

        // 不存在已有文件, 用新文件完成
        // CRYPTO 时：file_full_path 已是加密文件，删除明文
        if encrypted_rel.is_some()
            && let Err(e) = tokio::fs::remove_file(&plaintext_full).await {
                tracing::warn!(
                    "complete_file_and_local: remove plaintext file failed: {}",
                    e
                );
            }

        let final_local_path = encrypted_rel.unwrap_or(actual_local_path);

        let metadata = tokio::fs::metadata(&file_full_path).await?;
        let content_type = get_content_type(&file_full_path).await?;
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
        file.local_path_owner_id = 0;
        file.status = FileStatus::Normal as i8;
        file.change_time = now;
        file_local.local_path = final_local_path;

        // 在事务中更新数据库（CAS: AND status=Unfinished，防止并发重复完成）
        let mut tx = self.db.begin().await?;

        let cas_result = Update::<sqlx::MySql, FileModel>::new()
            .set(FileModel::FILE_SIZE, file.file_size)
            .set(FileModel::FILE_MD5, &file.file_md5)
            .set(FileModel::CONTENT_TYPE, &file.content_type)
            .set(FileModel::MODIFY_TIME, file.modify_time)
            .set(FileModel::LOCAL_PATH_OWNER_ID, file.local_path_owner_id)
            .set(FileModel::STATUS, file.status)
            .set(FileModel::CHANGE_TIME, file.change_time)
            .execute(&mut *tx, |qb| {
                qb.push_where().field_eq("id", file.id);
                qb.push_and().field_eq("status", FileStatus::Unfinished as i8);
            })
            .await?;

        if cas_result.rows_affected() == 0 {
            // 另一并发调用已完成该文件，直接视为成功
            if let Err(rb) = tx.rollback().await {
                tracing::warn!("complete_file_and_local: CAS rollback failed: {}", rb);
            }
            return Ok(None);
        }

        Update::<sqlx::MySql, FileLocalModel>::new()
            .set(FileLocalModel::LOCAL_PATH, &file_local.local_path)
            .execute(&mut *tx, |qb| {
                qb.push_where().field_eq("id", file_local.id);
            })
            .await?;

        tx.commit().await?;
        Ok(None)
    }
}
