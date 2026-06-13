use lsys_core::db::{QueryBuilderExt, TableMeta};
use sqlx::{MySql, QueryBuilder};

use super::FileDataDao;
use crate::common::FileResult;
use crate::model::*;

impl FileDataDao {
    /// 获取单个文件的 URL
    pub async fn get_file_url(&self, file: &FileModel) -> FileResult<Option<String>> {
        let urls = self.get_file_urls(std::slice::from_ref(file)).await?;
        Ok(urls.get(&file.id).cloned().flatten())
    }

    /// 批量获取文件 URL
    ///
    /// 传入多个 FileModel，一次性查询所有 local / oss 记录，返回 file_id -> Option<url> 的映射。
    pub async fn get_file_urls(
        &self,
        files: &[FileModel],
    ) -> FileResult<std::collections::HashMap<u64, Option<String>>> {
        use std::collections::HashMap;

        let mut result: HashMap<u64, Option<String>> = HashMap::new();
        if files.is_empty() {
            return Ok(result);
        }

        let mut local_public_ids: Vec<u64> = Vec::new();
        let mut local_private_ids: Vec<u64> = Vec::new();
        let mut oss_files: Vec<&FileModel> = Vec::new();

        for f in files {
            if !FileStatus::Normal.eq(f.status) {
                continue;
            }
            if f.storage_type == FileModel::STORAGE_TYPE_LOCAL_PUBLIC {
                local_public_ids.push(f.id);
            } else if f.is_local() {
                local_private_ids.push(f.id);
            } else {
                oss_files.push(f);
            }
        }

        if !local_public_ids.is_empty() {
            let mut qb: QueryBuilder<MySql> =
                QueryBuilder::new(format!("SELECT * FROM {}", FileLocalModel::table_name()));
            qb.push_where()
                .field_in_copied("file_id", &local_public_ids);
            let locals: Vec<FileLocalModel> =
                qb.build_query_as().fetch_all(&self.helper.db).await?;

            let prefix = self.runtime_setting.get_local_public_url_prefix().await?;

            for local_rec in &locals {
                if !local_rec.local_path.is_empty() {
                    result.insert(
                        local_rec.file_id,
                        Some(format!("{}{}", prefix, local_rec.local_path)),
                    );
                }
            }
        }

        for file_id in local_private_ids {
            result.insert(file_id, None);
        }

        if !oss_files.is_empty() {
            let mut storage_types: std::collections::HashSet<String> =
                std::collections::HashSet::new();
            for f in &oss_files {
                storage_types.insert(f.storage_type.clone());
            }

            let mut private_storage_types: std::collections::HashSet<String> =
                std::collections::HashSet::new();
            for storage_type in &storage_types {
                if let Ok(Some(config)) = self.oss_config.find_by_config_key(storage_type).await
                    && config.is_private
                {
                    private_storage_types.insert(storage_type.clone());
                }
            }

            let mut public_oss_ids: Vec<u64> = Vec::new();
            let mut private_oss_ids: Vec<u64> = Vec::new();
            for f in &oss_files {
                if private_storage_types.contains(&f.storage_type) {
                    private_oss_ids.push(f.id);
                } else {
                    public_oss_ids.push(f.id);
                }
            }

            for file_id in private_oss_ids {
                result.insert(file_id, None);
            }

            if !public_oss_ids.is_empty() {
                let mut qb: QueryBuilder<MySql> =
                    QueryBuilder::new(format!("SELECT * FROM {}", FileOssModel::table_name()));
                qb.push_where().field_in_copied("file_id", &public_oss_ids);
                let osses: Vec<FileOssModel> =
                    qb.build_query_as().fetch_all(&self.helper.db).await?;
                for oss_rec in &osses {
                    if !oss_rec.object_url.is_empty() {
                        result.insert(oss_rec.file_id, Some(oss_rec.object_url.clone()));
                    }
                }
            }
        }

        Ok(result)
    }
}
