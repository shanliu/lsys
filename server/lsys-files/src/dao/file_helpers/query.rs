use lsys_core::db::{SqlQuote, TableMeta};
use lsys_core::sql_format;

use super::super::FileResult;
use super::FileHelper;
use crate::model::*;

impl FileHelper {
    /// 查询 file_local by file_id
    pub async fn find_file_local_by_file_id(
        &self,
        file_id: u64,
    ) -> FileResult<Option<FileLocalModel>> {
        let row = sqlx::query_as::<_, FileLocalModel>(&sql_format!(
            "SELECT * FROM {} WHERE file_id={} LIMIT 1",
            FileLocalModel::table_name(),
            file_id
        ))
        .fetch_optional(&self.db)
        .await?;
        Ok(row)
    }

    /// 查询 file_local by oss_file_id (查找从某个OSS文件同步而来的本地文件记录)
    pub async fn find_file_local_by_oss_file_id(
        &self,
        oss_file_id: u64,
    ) -> FileResult<Option<FileLocalModel>> {
        let row = sqlx::query_as::<_, FileLocalModel>(&sql_format!(
            "SELECT * FROM {} WHERE oss_file_id={} LIMIT 1",
            FileLocalModel::table_name(),
            oss_file_id
        ))
        .fetch_optional(&self.db)
        .await?;
        Ok(row)
    }

    /// 查询 file by id
    pub async fn find_file_by_id(&self, file_id: u64) -> FileResult<Option<FileModel>> {
        let row = sqlx::query_as::<_, FileModel>(&sql_format!(
            "SELECT * FROM {} WHERE id={} LIMIT 1",
            FileModel::table_name(),
            file_id
        ))
        .fetch_optional(&self.db)
        .await?;
        Ok(row)
    }

    /// 查询 file_user by id
    pub async fn find_file_user_by_id(&self, id: u64) -> FileResult<Option<FileUserModel>> {
        let row = sqlx::query_as::<_, FileUserModel>(&sql_format!(
            "SELECT * FROM {} WHERE id={} LIMIT 1",
            FileUserModel::table_name(),
            id
        ))
        .fetch_optional(&self.db)
        .await?;
        Ok(row)
    }

    /// 查询 file_oss by file_id
    pub async fn find_file_oss_by_file_id(&self, file_id: u64) -> FileResult<Option<FileOssModel>> {
        let row = sqlx::query_as::<_, FileOssModel>(&sql_format!(
            "SELECT * FROM {} WHERE file_id={} LIMIT 1",
            FileOssModel::table_name(),
            file_id
        ))
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
        let row = sqlx::query_as::<_, FileLocalChunkModel>(&sql_format!(
            "SELECT * FROM {} WHERE file_id={} AND chunk_index={} LIMIT 1",
            FileLocalChunkModel::table_name(),
            file_id,
            chunk_index
        ))
        .fetch_optional(&self.db)
        .await?;
        Ok(row)
    }

    /// 查询 file_id 的所有 file_local_chunk
    pub async fn find_chunks_by_file_id(
        &self,
        file_id: u64,
    ) -> FileResult<Vec<FileLocalChunkModel>> {
        let rows = sqlx::query_as::<_, FileLocalChunkModel>(&sql_format!(
            "SELECT * FROM {} WHERE file_id={} ORDER BY chunk_index ASC",
            FileLocalChunkModel::table_name(),
            file_id
        ))
        .fetch_all(&self.db)
        .await?;
        Ok(rows)
    }

    /// 查询 file_user by user_id + app_id + file_id + status
    pub async fn find_file_user(
        &self,
        user_id: u64,
        app_id: u64,
        file_id: u64,
        status: FileUserStatus,
    ) -> FileResult<Option<FileUserModel>> {
        let row = sqlx::query_as::<_, FileUserModel>(&sql_format!(
            "SELECT * FROM {} WHERE user_id={} AND app_id={} AND file_id={} AND status={} LIMIT 1",
            FileUserModel::table_name(),
            user_id,
            app_id,
            file_id,
            status as i8
        ))
        .fetch_optional(&self.db)
        .await?;
        Ok(row)
    }

    /// 查询 file_user by user_id + app_id + source_md5 + status
    pub async fn find_file_user_by_source_md5(
        &self,
        user_id: u64,
        app_id: u64,
        source_md5: &str,
        status: FileUserStatus,
    ) -> FileResult<Option<FileUserModel>> {
        let row = sqlx::query_as::<_, FileUserModel>(&sql_format!(
            "SELECT * FROM {} WHERE user_id={} AND app_id={} AND source_md5={} AND status={} LIMIT 1",
            FileUserModel::table_name(),
            user_id,
            app_id,
            source_md5,
            status as i8
        ))
        .fetch_optional(&self.db)
        .await?;
        Ok(row)
    }

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
}
