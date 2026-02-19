use lsys_core::db::{Insert, OffsetPageParam, SqlQuote, TableMeta};
use lsys_core::db_option_executor;
use lsys_core::now_time;
use sqlx::{MySql, Pool, Transaction};
use tracing::{debug, warn};

use crate::model::FileLogModel;

/// 文件日志 DAO
pub struct FileLogDao {
    db: Pool<MySql>,
}

impl FileLogDao {
    pub fn new(db: Pool<MySql>) -> Self {
        Self { db }
    }

    /// 添加文件日志
    pub async fn add(
        &self,
        file_id: u64,
        file_chunk_id: u64,
        user_id: u64,
        message: &str,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
    ) {
        let time = now_time().unwrap_or_default();
        let msg: String = message.chars().take(1024).collect();

        let res = db_option_executor!(
            db,
            {
                Insert::<FileLogModel>::new()
                    .set(FileLogModel::FILE_ID, file_id)
                    .set(FileLogModel::FILE_CHUNK_ID, file_chunk_id)
                    .set(FileLogModel::MESSAGE, msg)
                    .set(FileLogModel::USER_ID, user_id)
                    .set(FileLogModel::ADD_TIME, time)
                    .execute(db.as_executor())
                    .await
            },
            transaction,
            &self.db
        );
        match res {
            Err(err) => warn!("add file log fail:{}", err),
            Ok(r) => debug!("add file log id:{}", r.last_insert_id()),
        };
    }

    /// 查询文件日志列表（支持分页）
    pub async fn list_by_file_id(
        &self,
        file_id: u64,
        page: &OffsetPageParam,
    ) -> crate::common::FileResult<Vec<FileLogModel>> {
        let sql = format!(
            "SELECT * FROM {} WHERE file_id={} ORDER BY id DESC {}",
            FileLogModel::table_name().sql_quote(),
            file_id,
            page.page_query().limit_sql().unwrap_or_default()
        );
        sqlx::query_as::<_, FileLogModel>(&sql)
            .fetch_all(&self.db)
            .await
            .map_err(Into::into)
    }

    /// 查询文件日志总数
    pub async fn count_by_file_id(&self, file_id: u64) -> crate::common::FileResult<i64> {
        let sql = format!(
            "SELECT COUNT(*) as count FROM {} WHERE file_id={}",
            FileLogModel::table_name().sql_quote(),
            file_id
        );
        sqlx::query_scalar(&sql)
            .fetch_one(&self.db)
            .await
            .map_err(Into::into)
    }
}
