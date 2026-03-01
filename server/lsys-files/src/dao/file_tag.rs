use lsys_core::db::{Insert, OptionTxExecutor, SqlQuote, SqlSuffix, TableMeta, Update};
use lsys_core::sql_format;
use lsys_core::utils::now_time;
use sqlx::{MySql, Pool, Transaction};

use super::*;
use crate::model::*;

/// 文件标签 DAO
pub struct FileTagDao {
    db: Pool<MySql>,
}

impl FileTagDao {
    pub fn new(db: Pool<MySql>) -> Self {
        Self { db }
    }

    /// 添加标签
    ///
    /// 防重复逻辑：先查是否已有 status=Normal 的同名标签 → 有则直接返回（幂等），否则 INSERT 新行。
    /// 不复用已删除记录，每次添加都是新行，保证历史记录可追溯。
    pub async fn add_tag(
        &self,
        file_id: u64,
        user_id: u64,
        app_id: u64,
        tag_name: &str,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
    ) -> FileResult<u64> {
        let tag_name = tag_name.trim().to_lowercase();
        if tag_name.is_empty() {
            return Err(FileError::Param(lsys_core::fluent_message!(
                "file-tag-name-empty"
            )));
        }

        let now = now_time()?;

        // 查是否已有 Normal 状态的同名标签（幂等）
        let existing = sqlx::query_as::<_, FileTagModel>(&sql_format!(
            "SELECT * FROM {} WHERE file_id={} AND user_id={} AND app_id={} AND tag_name={} AND status={} LIMIT 1",
            FileTagModel::table_name(),
            file_id,
            user_id,
            app_id,
            &tag_name,
            FileTagStatus::Normal as i8
        ))
        .fetch_optional(&self.db)
        .await?;

        if let Some(existing) = existing {
            return Ok(existing.id);
        }

        // INSERT 新行
        let res = Insert::<_, FileTagModel>::new()
            .set(FileTagModel::FILE_ID, file_id)
            .set(FileTagModel::USER_ID, user_id)
            .set(FileTagModel::APP_ID, app_id)
            .set(FileTagModel::TAG_NAME, &tag_name)
            .set(FileTagModel::STATUS, FileTagStatus::Normal as i8)
            .set(FileTagModel::ADD_TIME, now)
            .set(FileTagModel::CHANGE_TIME, 0u64)
            .execute(OptionTxExecutor::new(transaction, &self.db))
            .await?;

        Ok(res.last_insert_id())
    }

    /// 移除标签（软删除）
    pub async fn remove_tag(
        &self,
        file_id: u64,
        user_id: u64,
        app_id: u64,
        tag_name: &str,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
    ) -> FileResult<u64> {
        let tag_name = tag_name.trim().to_lowercase();
        if tag_name.is_empty() {
            return Err(FileError::Param(lsys_core::fluent_message!(
                "file-tag-name-empty"
            )));
        }

        let now = now_time()?;

        let where_sql = sql_format!(
            "file_id={} AND user_id={} AND app_id={} AND tag_name={} AND status={}",
            file_id,
            user_id,
            app_id,
            &tag_name,
            FileTagStatus::Normal as i8
        );

        let res = Update::<_, FileTagModel>::new()
            .set(FileTagModel::STATUS, FileTagStatus::Deleted as i8)
            .set(FileTagModel::CHANGE_TIME, now)
            .execute(
                SqlSuffix::Where(&where_sql),
                OptionTxExecutor::new(transaction, &self.db),
            )
            .await?;

        Ok(res.rows_affected())
    }

    /// 删除文件关联的所有标签（软删除）
    ///
    /// 用于删除文件时同步清理该用户在该应用下该文件的所有标签。
    pub(crate) async fn remove_all_tags(
        &self,
        file_id: u64,
        user_id: u64,
        app_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
    ) -> FileResult<u64> {
        let now = now_time()?;

        let where_sql = sql_format!(
            "file_id={} AND user_id={} AND app_id={} AND status={}",
            file_id,
            user_id,
            app_id,
            FileTagStatus::Normal as i8
        );

        let res = Update::<_, FileTagModel>::new()
            .set(FileTagModel::STATUS, FileTagStatus::Deleted as i8)
            .set(FileTagModel::CHANGE_TIME, now)
            .execute(
                SqlSuffix::Where(&where_sql),
                OptionTxExecutor::new(transaction, &self.db),
            )
            .await?;

        Ok(res.rows_affected())
    }

    /// 查询文件的所有标签名（status=Normal，去重）
    ///
    /// 用于文件拷贝/同步时获取源文件的标签，以便复制到新文件。
    pub(crate) async fn get_file_tag_names(&self, file_id: u64) -> FileResult<Vec<String>> {
        let sql = format!(
            "SELECT DISTINCT tag_name FROM {} WHERE file_id={} AND status={} ORDER BY tag_name ASC",
            FileTagModel::table_name().sql_quote(),
            file_id,
            FileTagStatus::Normal as i8
        );
        let rows: Vec<(String,)> = sqlx::query_as(&sql).fetch_all(&self.db).await?;
        Ok(rows.into_iter().map(|r| r.0).collect())
    }
}
