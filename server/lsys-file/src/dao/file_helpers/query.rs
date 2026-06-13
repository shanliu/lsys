use lsys_core::db::TableMeta;

use super::super::FileResult;
use super::FileHelper;
use crate::common::OssProvider;
use crate::model::*;

impl FileHelper {
    /// 查询 file_local by file_id
    pub async fn find_file_local_by_file_id(
        &self,
        file_id: u64,
    ) -> FileResult<Option<FileLocalModel>> {
        let row = sqlx::query_as::<_, FileLocalModel>(&format!(
            "SELECT * FROM {} WHERE file_id=? LIMIT 1",
            FileLocalModel::table_name()
        ))
        .bind(file_id)
        .fetch_optional(&self.db)
        .await?;
        Ok(row)
    }

    /// 查询 file by id
    pub async fn find_file_by_id(&self, file_id: u64) -> FileResult<Option<FileModel>> {
        let row = sqlx::query_as::<_, FileModel>(&format!(
            "SELECT * FROM {} WHERE id=? LIMIT 1",
            FileModel::table_name()
        ))
        .bind(file_id)
        .fetch_optional(&self.db)
        .await?;
        Ok(row)
    }

    /// 查询 file user by id
    pub async fn find_file_ref_by_id(&self, id: u64) -> FileResult<Option<FileRefModel>> {
        let row = sqlx::query_as::<_, FileRefModel>(&format!(
            "SELECT * FROM {} WHERE id=? LIMIT 1",
            FileRefModel::table_name()
        ))
        .bind(id)
        .fetch_optional(&self.db)
        .await?;
        Ok(row)
    }

    /// 查询 file_oss by file_id
    pub async fn find_file_oss_by_file_id(&self, file_id: u64) -> FileResult<Option<FileOssModel>> {
        let row = sqlx::query_as::<_, FileOssModel>(&format!(
            "SELECT * FROM {} WHERE file_id=? LIMIT 1",
            FileOssModel::table_name()
        ))
        .bind(file_id)
        .fetch_optional(&self.db)
        .await?;
        Ok(row)
    }

    /// 查询 file_local_chunk by file_id and chunk_index
    pub async fn find_chunk_by_file_and_index(
        &self,
        file_id: u64,
        chunk_index: u32,
    ) -> FileResult<Option<FileLocalChunkModel>> {
        let row = sqlx::query_as::<_, FileLocalChunkModel>(&format!(
            "SELECT * FROM {} WHERE file_id=? AND chunk_index=? LIMIT 1",
            FileLocalChunkModel::table_name()
        ))
        .bind(file_id)
        .bind(chunk_index)
        .fetch_optional(&self.db)
        .await?;
        Ok(row)
    }

    /// 查询 file_id 的所有 file_local_chunk
    pub async fn find_chunks_by_file_id(
        &self,
        file_id: u64,
    ) -> FileResult<Vec<FileLocalChunkModel>> {
        let rows = sqlx::query_as::<_, FileLocalChunkModel>(&format!(
            "SELECT * FROM {} WHERE file_id=? ORDER BY chunk_index ASC",
            FileLocalChunkModel::table_name()
        ))
        .bind(file_id)
        .fetch_all(&self.db)
        .await?;
        Ok(rows)
    }

    /// 查询 user file  by user_id + app_id + file_id + status
    pub async fn find_file_ref(
        &self,
        user_id: u64,
        app_id: u64,
        file_id: u64,
        status: FileUserStatus,
    ) -> FileResult<Option<FileRefModel>> {
        let row = sqlx::query_as::<_, FileRefModel>(&format!(
            "SELECT * FROM {} WHERE user_id=? AND app_id=? AND file_id=? AND status=? LIMIT 1",
            FileRefModel::table_name()
        ))
        .bind(user_id)
        .bind(app_id)
        .bind(file_id)
        .bind(status as i8)
        .fetch_optional(&self.db)
        .await?;
        Ok(row)
    }

    /// 查询 user file by user_id + app_id + source_md5 + status
    pub async fn find_file_ref_by_source_md5(
        &self,
        user_id: u64,
        app_id: u64,
        source_md5: &str,
        status: FileUserStatus,
    ) -> FileResult<Option<FileRefModel>> {
        let row = sqlx::query_as::<_, FileRefModel>(&format!(
            "SELECT * FROM {} WHERE user_id=? AND app_id=? AND source_md5=? AND status=? LIMIT 1",
            FileRefModel::table_name()
        ))
        .bind(user_id)
        .bind(app_id)
        .bind(source_md5)
        .bind(status as i8)
        .fetch_optional(&self.db)
        .await?;
        Ok(row)
    }

    /// 在本地存储中查找已存在的相同 MD5 文件（物理校验：size + MD5 重算）
    pub async fn find_existing_local_file(
        &self,
        storage_type: &str,
        file_md5: &str,
    ) -> FileResult<Option<FileModel>> {
        self.find_existing_local_file_inner(storage_type, file_md5, None)
            .await
    }

    /// 在本地存储中查找已存在的相同 MD5 文件（排除指定 file_id，用于 complete 场景）
    pub async fn find_existing_local_file_exclude(
        &self,
        storage_type: &str,
        file_md5: &str,
        exclude_file_id: u64,
    ) -> FileResult<Option<FileModel>> {
        self.find_existing_local_file_inner(storage_type, file_md5, Some(exclude_file_id))
            .await
    }

    async fn find_existing_local_file_inner(
        &self,
        storage_type: &str,
        file_md5: &str,
        exclude_file_id: Option<u64>,
    ) -> FileResult<Option<FileModel>> {
        const BATCH_SIZE: u64 = 128;
        let mut last_id: u64 = 0;

        loop {
            let sql = if exclude_file_id.is_some() {
                format!(
                    "SELECT * FROM {} WHERE id>? AND storage_type=? AND file_md5=? AND status=? \
                     AND local_path_owner_id=0 AND id!=? ORDER BY id ASC LIMIT {}",
                    FileModel::table_name(),
                    BATCH_SIZE
                )
            } else {
                format!(
                    "SELECT * FROM {} WHERE id>? AND storage_type=? AND file_md5=? AND status=? \
                     AND local_path_owner_id=0 ORDER BY id ASC LIMIT {}",
                    FileModel::table_name(),
                    BATCH_SIZE
                )
            };

            let mut query = sqlx::query_as::<_, FileModel>(&sql)
                .bind(last_id)
                .bind(storage_type)
                .bind(file_md5)
                .bind(FileStatus::Normal as i8);
            if let Some(id) = exclude_file_id {
                query = query.bind(id);
            }

            let rows = query.fetch_all(&self.db).await?;
            if rows.is_empty() {
                return Ok(None);
            }

            for row in &rows {
                let Some(file_local) = self.find_file_local_by_file_id(row.id).await? else {
                    continue;
                };
                if file_local.local_path.is_empty() {
                    continue;
                }
                let full_path = match self
                    .get_full_local_path(&row.storage_type, &file_local.local_path)
                    .await
                {
                    Ok(p) => p,
                    Err(_) => continue,
                };
                let metadata = match tokio::fs::metadata(&full_path).await {
                    Ok(m) => m,
                    Err(_) => continue,
                };
                if metadata.len() != row.file_size {
                    continue;
                }
                match self.compute_file_md5(&full_path).await {
                    Ok(md5) if md5 == row.file_md5 => return Ok(Some(row.clone())),
                    _ => continue,
                }
            }

            last_id = rows.last().map(|r| r.id).unwrap_or(last_id);
        }
    }

    /// 在 OSS 存储中查找已存在的相同 MD5 文件
    ///
    /// 调用 `object_meta` 做活性检查；若能获取到 `content_md5`（简单上传），
    /// 同时校验 MD5 一致性；分片上传 ETag 含 '-' 时无 MD5，仅凭存在性通过。
    /// 网络/鉴权异常时降级，直接信任 DB 记录。
    pub async fn find_existing_oss_file(
        &self,
        storage_type: &str,
        file_md5: &str,
        oss_provider: &dyn OssProvider,
    ) -> FileResult<Option<FileModel>> {
        const BATCH_SIZE: u64 = 128;
        let mut last_id: u64 = 0;

        loop {
            let sql = format!(
                "SELECT * FROM {} WHERE id>? AND storage_type=? AND file_md5=? AND status=? \
                 AND local_path_owner_id=0 ORDER BY id ASC LIMIT {}",
                FileModel::table_name(),
                BATCH_SIZE
            );

            let rows = sqlx::query_as::<_, FileModel>(&sql)
                .bind(last_id)
                .bind(storage_type)
                .bind(file_md5)
                .bind(FileStatus::Normal as i8)
                .fetch_all(&self.db)
                .await?;

            if rows.is_empty() {
                return Ok(None);
            }

            for row in &rows {
                if let Ok(Some(oss_rec)) = self.find_file_oss_by_file_id(row.id).await {
                    let meta = oss_provider.object_meta(&oss_rec).await?;
                    if !meta.exists {
                        // 对象已被外部删除，跳过
                        continue;
                    }
                    // 简单上传时 ETag = MD5，可做额外校验；分片上传无 MD5 则跳过
                    if let Some(ref remote_md5) = meta.content_md5
                        && remote_md5 != &row.file_md5 {
                            continue; // MD5 不符，跳过
                        }
                }
                return Ok(Some(row.clone()));
            }

            last_id = rows.last().map(|r| r.id).unwrap_or(last_id);
        }
    }
    /// 查询特定用户+应用下文件的所有标签名（status=Normal，去重）
    ///
    /// 用于文件拷贝/同步时获取源文件在特定用户上下文中的标签。
    pub(crate) async fn get_file_tag_names_for_user(
        &self,
        file_id: u64,
        user_id: u64,
        app_id: u64,
    ) -> FileResult<Vec<String>> {
        let sql = format!(
            "SELECT DISTINCT tag_name FROM {} WHERE file_id=? AND user_id=? AND app_id=? AND status=? ORDER BY tag_name ASC",
            FileTagModel::table_name(),
        );
        let rows: Vec<(String,)> = sqlx::query_as(&sql)
            .bind(file_id)
            .bind(user_id)
            .bind(app_id)
            .bind(FileTagStatus::Normal as i8)
            .fetch_all(&self.db)
            .await?;
        Ok(rows.into_iter().map(|r| r.0).collect())
    }
}
