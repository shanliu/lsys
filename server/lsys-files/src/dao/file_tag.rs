use lsys_core::db::{BatchInsert, Insert, OptionTxExecutor, QueryBuilderExt, TableMeta, Update};
use lsys_core::utils::now_time;
use sqlx::{MySql, Pool, QueryBuilder, Transaction};

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
        let existing = sqlx::query_as::<_, FileTagModel>(
            &format!(
                "SELECT * FROM {} WHERE file_id=? AND user_id=? AND app_id=? AND tag_name=? AND status=? LIMIT 1",
                FileTagModel::table_name(),
            )
        )
        .bind(file_id)
        .bind(user_id)
        .bind(app_id)
        .bind(&tag_name)
        .bind(FileTagStatus::Normal as i8)
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

    /// 批量添加标签
    ///
    /// 一次性添加多个标签，内部通过单次 SELECT 查找已存在的标签（幂等），
    /// 再通过 BatchInsert 批量插入不存在的标签，减少数据库往返次数。
    pub async fn batch_add_tags(
        &self,
        file_id: u64,
        user_id: u64,
        app_id: u64,
        tag_names: &[&str],
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
    ) -> FileResult<Vec<u64>> {
        // 去重、trim、转小写，过滤空串
        let tag_names: Vec<String> = tag_names
            .iter()
            .map(|t| t.trim().to_lowercase())
            .filter(|t| !t.is_empty())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        if tag_names.is_empty() {
            return Ok(vec![]);
        }

        // 单次 SELECT 查出所有已存在的 Normal 标签
        let mut qb: QueryBuilder<MySql> = QueryBuilder::new(format!(
            "SELECT * FROM {}",
            FileTagModel::table_name(),
        ));
        qb.push_where()
            .field_eq("file_id", file_id)
            .push_and()
            .field_eq("user_id", user_id)
            .push_and()
            .field_eq("app_id", app_id)
            .push_and()
            .field_in_string("tag_name", &tag_names)
            .push_and()
            .field_eq("status", FileTagStatus::Normal as i8);

        let existing_tags: Vec<FileTagModel> = qb.build_query_as()
            .fetch_all(&self.db)
            .await?;

        let existing_names: std::collections::HashSet<&str> = existing_tags
            .iter()
            .map(|t| t.tag_name.as_str())
            .collect();

        let mut result_ids: Vec<u64> = existing_tags.iter().map(|t| t.id).collect();

        // 筛选出需要新增的标签
        let new_tags: Vec<&String> = tag_names
            .iter()
            .filter(|t| !existing_names.contains(t.as_str()))
            .collect();

        if !new_tags.is_empty() {
            let now = now_time()?;

            let mut batch_insert =
                BatchInsert::<_, FileTagModel>::with_capacity(new_tags.len());
            for tag_name in &new_tags {
                batch_insert = batch_insert.push(
                    Insert::<_, FileTagModel>::new()
                        .set(FileTagModel::FILE_ID, file_id)
                        .set(FileTagModel::USER_ID, user_id)
                        .set(FileTagModel::APP_ID, app_id)
                        .set(FileTagModel::TAG_NAME, tag_name.as_str())
                        .set(FileTagModel::STATUS, FileTagStatus::Normal as i8)
                        .set(FileTagModel::ADD_TIME, now)
                        .set(FileTagModel::CHANGE_TIME, 0u64),
                );
            }

            let res = batch_insert
                .execute(OptionTxExecutor::new(transaction, &self.db))
                .await?;

            // 批量插入的自增 ID 是连续的
            let first_id = res.last_insert_id();
            for i in 0..new_tags.len() as u64 {
                result_ids.push(first_id + i);
            }
        }

        Ok(result_ids)
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

        let res = Update::<_, FileTagModel>::new()
            .set(FileTagModel::STATUS, FileTagStatus::Deleted as i8)
            .set(FileTagModel::CHANGE_TIME, now)
            .execute(OptionTxExecutor::new(transaction, &self.db), |qb| {
                qb.push_where().field_eq("file_id", file_id)
                    .push_and().field_eq("user_id", user_id)
                    .push_and().field_eq("app_id", app_id)
                    .push_and().field_eq("tag_name", tag_name.clone())
                    .push_and().field_eq("status", FileTagStatus::Normal as i8);
            })
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

        let res = Update::<_, FileTagModel>::new()
            .set(FileTagModel::STATUS, FileTagStatus::Deleted as i8)
            .set(FileTagModel::CHANGE_TIME, now)
            .execute(OptionTxExecutor::new(transaction, &self.db), |qb| {
                qb.push_where().field_eq("file_id", file_id)
                    .push_and().field_eq("user_id", user_id)
                    .push_and().field_eq("app_id", app_id)
                    .push_and().field_eq("status", FileTagStatus::Normal as i8);
            })
            .await?;

        Ok(res.rows_affected())
    }

}
