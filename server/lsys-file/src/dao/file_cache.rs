use std::sync::Arc;

use async_trait::async_trait;
use lsys_core::cache::{LocalCache, LocalCacheClearItem};

use crate::model::FileModel;

use super::{FileDao, FileResult};

pub enum FileLocalCacheClear {
    FileUrl(Arc<LocalCache<u64, Option<String>>>),
}

impl FileLocalCacheClear {
    pub fn new_clears(file_dao: &FileDao) -> Vec<Self> {
        vec![Self::FileUrl(file_dao.file_url_cache.clone())]
    }
}

#[async_trait]
impl LocalCacheClearItem<'_> for FileLocalCacheClear {
    fn cache_name(&self) -> &str {
        match self {
            Self::FileUrl(cache) => cache.config().cache_name,
        }
    }
    async fn clear_from_message(&self, msg: &str) -> Result<(), String> {
        match self {
            Self::FileUrl(cache) => {
                cache
                    .del(&msg.parse::<u64>().map_err(|e| e.to_string())?)
                    .await
            }
        };
        Ok(())
    }
}

impl FileDao {
    pub fn cache(&'_ self) -> FileDaoCache<'_> {
        FileDaoCache { dao: self }
    }
}

pub struct FileDaoCache<'t> {
    pub dao: &'t FileDao,
}

impl FileDaoCache<'_> {
    /// 获取文件 URL（带缓存）
    pub async fn get_file_url(&self, file: &FileModel) -> FileResult<Option<String>> {
        self.dao
            .file_url_cache
            .get_or_fetch(&file.id, || self.dao.get_file_url(file))
            .await
    }

    /// 批量获取文件 URL（带缓存）
    pub async fn get_file_urls(
        &self,
        files: &[FileModel],
    ) -> FileResult<std::collections::HashMap<u64, String>> {
        use std::collections::HashMap;

        if files.is_empty() {
            return Ok(HashMap::new());
        }

        // 收集所有文件 ID 和对应的文件
        let file_ids: Vec<u64> = files.iter().map(|f| f.id).collect();
        let files_map: HashMap<u64, FileModel> = files.iter().map(|f| (f.id, f.clone())).collect();

        // 批量从缓存获取
        let cached_results = self
            .dao
            .file_url_cache
            .get_or_fetch_many(&file_ids, |missing_ids| async move {
                // 对于缓存未命中的 ID，从数据库查询
                let missing_files: Vec<FileModel> = missing_ids
                    .iter()
                    .filter_map(|id| files_map.get(id).cloned())
                    .collect();

                let urls = self.dao.get_file_urls(&missing_files).await?;

                // 转换为 HashMap<u64, Option<String>>
                let mut result_map = HashMap::new();
                for id in missing_ids {
                    result_map.insert(id, urls.get(&id).cloned());
                }

                Ok::<HashMap<u64, Option<String>>, crate::common::FileError>(result_map)
            })
            .await?;

        // 过滤掉 None 值，只返回有 URL 的文件
        let mut result: HashMap<u64, String> = HashMap::new();
        for (file_id, url_opt) in cached_results {
            if let Some(url) = url_opt {
                result.insert(file_id, url);
            }
        }

        Ok(result)
    }
}
