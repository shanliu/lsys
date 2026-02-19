use lsys_core::db::{OffsetPageParam, SqlQuote, TableMeta};
use sqlx::{MySql, Pool};

use crate::model::FileLocalChunkModel;

/// 文件本地分片 DAO
pub struct FileLocalChunkDao {
    db: Pool<MySql>,
}

impl FileLocalChunkDao {
    pub fn new(db: Pool<MySql>) -> Self {
        Self { db }
    }

    /// 查询文件分片列表（支持分页）
    pub async fn list_by_file_id(
        &self,
        file_id: u64,
        page: &OffsetPageParam,
    ) -> crate::common::FileResult<Vec<FileLocalChunkModel>> {
        let sql = format!(
            "SELECT * FROM {} WHERE file_id={} ORDER BY chunk_index DESC {}",
            FileLocalChunkModel::table_name().sql_quote(),
            file_id,
            page.page_query().limit_sql().unwrap_or_default()
        );
        sqlx::query_as::<_, FileLocalChunkModel>(&sql)
            .fetch_all(&self.db)
            .await
            .map_err(Into::into)
    }

    /// 查询文件分片总数
    pub async fn count_by_file_id(&self, file_id: u64) -> crate::common::FileResult<i64> {
        let sql = format!(
            "SELECT COUNT(*) as count FROM {} WHERE file_id={}",
            FileLocalChunkModel::table_name().sql_quote(),
            file_id
        );
        sqlx::query_scalar(&sql)
            .fetch_one(&self.db)
            .await
            .map_err(Into::into)
    }
}
