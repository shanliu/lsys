use std::collections::HashMap;

use lsys_core::db::QueryBuilderExt;
use lsys_core::db::utils::Fetch;
use sqlx::MySql;

use crate::common::FileResult;
use crate::model::{FileModel, FileRefModel};

use super::FileDataDao;

impl FileDataDao {
    /// 按单个 id 查询 FileModel
    pub async fn find_file_by_id(&self, id: u64) -> FileResult<FileModel> {
        Ok(Fetch::<MySql, FileModel>::one(&self.helper.db, |qb| {
            qb.field_eq("id", id);
        })
        .await?)
    }

    /// 按单个 id 查询 FileRefModel
    pub async fn find_file_ref_by_id(&self, id: u64) -> FileResult<FileRefModel> {
        Ok(Fetch::<MySql, FileRefModel>::one(&self.helper.db, |qb| {
            qb.field_eq("id", id);
        })
        .await?)
    }

    /// 批量查询 FileRefModel，返回 id -> FileRefModel 的 HashMap（一次 IN 查询）
    pub async fn find_file_refs_by_ids(
        &self,
        ids: &[u64],
    ) -> FileResult<HashMap<u64, FileRefModel>> {
        if ids.is_empty() {
            return Ok(HashMap::new());
        }
        let map = Fetch::<MySql, FileRefModel>::map(
            &self.helper.db,
            |qb| {
                qb.field_in_copied("id", ids);
            },
            |r| r.id,
        )
        .await?;
        Ok(map)
    }
}
