use std::sync::Arc;

use async_trait::async_trait;
use lsys_core::cache::{LocalCache, LocalCacheClearItem};

use crate::common::FileError;
use crate::model::{FileLocalModel, FileModel, FileOssModel, FileRefModel};
use lsys_core::fluent_message;

use super::{FileDao, FileResult, UnifiedFileStream};

pub enum FileLocalCacheClear {
    FileUrl(Arc<LocalCache<u64, Option<String>>>),
    FileRef(Arc<LocalCache<u64, FileRefModel>>),
    FileModel(Arc<LocalCache<u64, FileModel>>),
    FileLocal(Arc<LocalCache<u64, FileLocalModel>>),
    FileOss(Arc<LocalCache<u64, FileOssModel>>),
    OssConfig(
        Arc<LocalCache<String, Option<lsys_setting::dao::SettingData<crate::dao::OssSettingData>>>>,
    ),
}

impl FileLocalCacheClear {
    pub fn new_clears(file_dao: &FileDao) -> Vec<Self> {
        vec![
            Self::FileUrl(file_dao.file_url_cache.clone()),
            Self::FileRef(file_dao.file_ref_cache.clone()),
            Self::FileModel(file_dao.file_model_cache.clone()),
            Self::FileLocal(file_dao.file_local_cache.clone()),
            Self::FileOss(file_dao.file_oss_cache.clone()),
            Self::OssConfig(file_dao.oss_config_cache.clone()),
        ]
    }
}

#[async_trait]
impl LocalCacheClearItem<'_> for FileLocalCacheClear {
    fn cache_name(&self) -> &str {
        match self {
            Self::FileUrl(cache) => cache.config().cache_name,
            Self::FileRef(cache) => cache.config().cache_name,
            Self::FileModel(cache) => cache.config().cache_name,
            Self::FileLocal(cache) => cache.config().cache_name,
            Self::FileOss(cache) => cache.config().cache_name,
            Self::OssConfig(cache) => cache.config().cache_name,
        }
    }
    async fn clear_from_message(&self, msg: &str, clear_all: bool) -> Result<(), String> {
        match self {
            Self::FileUrl(cache) => {
                if clear_all {
                    cache.clear_all_local().await;
                    return Ok(());
                }
                let id = msg.parse::<u64>().map_err(|e| e.to_string())?;
                cache.del(&id).await;
            }
            Self::FileRef(cache) => {
                let id = msg.parse::<u64>().map_err(|e| e.to_string())?;
                cache.del(&id).await;
            }
            Self::FileModel(cache) => {
                let id = msg.parse::<u64>().map_err(|e| e.to_string())?;
                cache.del(&id).await;
            }
            Self::FileLocal(cache) => {
                let id = msg.parse::<u64>().map_err(|e| e.to_string())?;
                cache.del(&id).await;
            }
            Self::FileOss(cache) => {
                let id = msg.parse::<u64>().map_err(|e| e.to_string())?;
                cache.del(&id).await;
            }
            Self::OssConfig(cache) => {
                // msg is config_key (String)
                cache.del(&msg.to_string()).await;
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
    /// 按单个 id 查询 FileModel（带缓存）
    pub async fn find_file_by_id(&self, id: u64) -> FileResult<FileModel> {
        self.dao
            .file_model_cache
            .get_or_fetch(&id, || self.dao.data_dao.find_file_by_id(id))
            .await
    }

    /// 获取文件 URL（带缓存）
    pub async fn get_file_url(&self, file: &FileModel) -> FileResult<Option<String>> {
        self.dao
            .file_url_cache
            .get_or_fetch(&file.id, || self.dao.data_dao.get_file_url(file))
            .await
    }

    /// 批量获取文件 URL（带缓存）
    ///
    /// 返回值说明：
    /// - `Some(Some(url))`: 文件可以生成公开访问 URL
    /// - `Some(None)`: 文件存在但不能生成 URL（私有存储或非 public 类型）
    /// - 不在 HashMap 中: 文件不存在或状态不正常
    pub async fn get_file_urls(
        &self,
        files: &[FileModel],
    ) -> FileResult<std::collections::HashMap<u64, Option<String>>> {
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

                let urls = self.dao.data_dao.get_file_urls(&missing_files).await?;

                // 转换为 HashMap<u64, Option<String>>
                let mut result_map = HashMap::new();
                for id in missing_ids {
                    result_map.insert(id, urls.get(&id).cloned().flatten());
                }

                Ok::<HashMap<u64, Option<String>>, crate::common::FileError>(result_map)
            })
            .await?;

        // 返回完整结果，包括 None 值
        Ok(cached_results)
    }

    /// 按单个 id 查询 FileRefModel（带缓存）
    pub async fn find_file_ref_by_id(&self, id: u64) -> FileResult<FileRefModel> {
        self.dao
            .file_ref_cache
            .get_or_fetch(&id, || self.dao.data_dao.find_file_ref_by_id(id))
            .await
    }

    /// 批量获取 FileRefModel（带缓存）
    ///
    /// 返回 id -> FileRefModel，不存在的 id 不出现在结果中。
    pub async fn find_file_refs_by_ids(
        &self,
        ids: &[u64],
    ) -> FileResult<std::collections::HashMap<u64, FileRefModel>> {
        use std::collections::HashMap;
        if ids.is_empty() {
            return Ok(HashMap::new());
        }
        self.dao
            .file_ref_cache
            .get_or_fetch_many(ids, |missing_ids| async move {
                let rows = self
                    .dao
                    .data_dao
                    .find_file_refs_by_ids(&missing_ids)
                    .await?;
                Ok::<HashMap<u64, FileRefModel>, crate::common::FileError>(rows)
            })
            .await
    }

    /// 查询 file_local 记录（带缓存）
    pub async fn find_file_local_by_file_id(&self, file_id: u64) -> FileResult<FileLocalModel> {
        self.dao
            .file_local_cache
            .get_or_fetch(&file_id, || async move {
                self.dao
                    .helper
                    .find_file_local_by_file_id(file_id)
                    .await?
                    .ok_or_else(|| {
                        FileError::System(fluent_message!(
                            "file-error",
                            &format!("File local record not found: {}", file_id)
                        ))
                    })
            })
            .await
    }

    /// 查询 file_oss 记录（带缓存）
    pub async fn find_file_oss_by_file_id(&self, file_id: u64) -> FileResult<FileOssModel> {
        self.dao
            .file_oss_cache
            .get_or_fetch(&file_id, || async move {
                self.dao
                    .helper
                    .find_file_oss_by_file_id(file_id)
                    .await?
                    .ok_or_else(|| {
                        FileError::System(fluent_message!(
                            "file-error",
                            &format!("File OSS record not found: {}", file_id)
                        ))
                    })
            })
            .await
    }

    /// 通过 file_model 读取文件流
    ///
    /// 根据文件存储类型（本地或 OSS）读取相应的文件流
    pub async fn read_file_stream(
        &self,
        file_model: &FileModel,
        offset: u64,
        length: Option<u64>,
    ) -> FileResult<UnifiedFileStream> {
        let stream = if file_model.is_local() {
            let local_record = self.find_file_local_by_file_id(file_model.id).await?;
            let iter = self
                .dao
                .data_dao
                .read_local_file_from_record(file_model, &local_record, offset, length)
                .await?;
            UnifiedFileStream::Local(iter)
        } else {
            let oss_record = self.find_file_oss_by_file_id(file_model.id).await?;
            let result = self
                .dao
                .data_dao
                .read_oss_file_from_record(file_model, &oss_record, offset, length)
                .await?;

            match result {
                crate::common::OssDownloadResult::RangeSupported(s) => {
                    UnifiedFileStream::OssRangeSupported(s)
                }
                crate::common::OssDownloadResult::FullStreamOnly(s) => {
                    UnifiedFileStream::OssFullStream {
                        stream: s,
                        skip_bytes: offset,
                        skipped: 0,
                        read_limit: length,
                        read_bytes: 0,
                    }
                }
            }
        };

        Ok(stream)
    }

    /// 按 config_key 查找 OSS 配置（带缓存）
    ///
    /// 内部调用 `FileOssConfigDao::find_by_config_key`
    pub async fn find_oss_config_by_key(
        &self,
        config_key: &str,
    ) -> FileResult<Option<lsys_setting::dao::SettingData<crate::dao::OssSettingData>>> {
        self.dao
            .oss_config_cache
            .get_or_fetch(&config_key.to_string(), || async {
                self.dao.oss_config.find_by_config_key(config_key).await
            })
            .await
    }

    /// 判断文件是否为私有（带缓存）
    ///
    /// 内部使用缓存的 OSS 配置来判断
    pub async fn is_private(&self, file: &FileModel) -> FileResult<bool> {
        // 本地私有存储类型判断
        if file.storage_type == FileModel::STORAGE_TYPE_LOCAL_PRIVATE
            || file.storage_type == FileModel::STORAGE_TYPE_LOCAL_CRYPTO
        {
            return Ok(true);
        }

        // 本地公开存储
        if file.storage_type == FileModel::STORAGE_TYPE_LOCAL_PUBLIC {
            return Ok(false);
        }

        // OSS 存储：查询缓存的配置中的 is_private 字段
        match self.find_oss_config_by_key(&file.storage_type).await? {
            Some(config) => Ok(config.is_private),
            // 配置不存在，默认视为私有（安全起见）
            None => Ok(true),
        }
    }
}
