mod file_cache;
mod file_config;
mod file_dao;
mod file_data;
mod file_download;
mod file_from_local;
mod file_from_oss;
mod file_from_upload;
mod file_from_url;
mod file_helpers;
mod file_log;
mod file_op_context;
mod file_oss_config;
mod file_tag;
mod logger;

pub use file_cache::*;
pub use file_config::*;
pub use file_dao::*;
pub use file_data::*;
pub use file_download::*;
pub use file_from_local::*;
pub use file_from_upload::*;
pub use file_helpers::*;
pub use file_log::*;
pub use file_op_context::*;
pub use file_oss_config::*;
pub use file_tag::*;

// Re-export common types
pub use crate::common::*;

use lsys_core::app_core::AppCore;
use lsys_core::cache::{LocalCache, LocalCacheConfig};
use lsys_core::remote_notify::RemoteNotify;
use lsys_logger::dao::ChangeLoggerDao;
use lsys_setting::dao::MultipleSetting;
use sqlx::{MySql, Pool};
use std::sync::Arc;

use crate::oss::OssProviderRegistry;

/// 返回文件模块的操作日志类型列表
pub fn log_types() -> Vec<&'static str> {
    use logger::{LogFileCopy, LogFileCreate, LogFileDelete, LogFileSync, LogFileUpload};
    use lsys_logger::dao::ChangeLogData;
    vec![
        LogFileCreate::log_type(),
        LogFileUpload::log_type(),
        LogFileDelete::log_type(),
        LogFileCopy::log_type(),
        LogFileSync::log_type(),
    ]
}

/// 文件服务 DAO 构建器
pub struct FileDaoBuilder;

impl FileDaoBuilder {
    /// 构建文件 DAO（从 AppCore 读取配置）
    pub fn build(
        db: Pool<MySql>,
        app_core: &AppCore,
        setting: Arc<MultipleSetting>,
        registry: Arc<OssProviderRegistry>,
        logger: Arc<ChangeLoggerDao>,
        remote_notify: Arc<RemoteNotify>,
    ) -> FileDao {
        let config = FileConfig::from_config(app_core);
        Self::build_with_config(db, config, setting, registry, logger, remote_notify)
    }

    /// 构建文件 DAO（直接传入配置）
    pub fn build_with_config(
        db: Pool<MySql>,
        config: FileConfig,
        setting: Arc<MultipleSetting>,
        registry: Arc<OssProviderRegistry>,
        logger: Arc<ChangeLoggerDao>,
        remote_notify: Arc<RemoteNotify>,
    ) -> FileDao {
        let helper = Arc::new(FileHelper::new(db.clone(), config.clone()));
        let download = Arc::new(FileDownloadManager::new(helper.clone()));
        let oss_config = Arc::new(FileOssConfigDao::new(db, setting, registry));
        
        // 创建文件 URL 缓存
        let file_url_cache = Arc::new(LocalCache::new(
            remote_notify,
            LocalCacheConfig::new("file_url", Some(1000), Some(300)),
        ));
        
        FileDao::new(helper, download, oss_config, logger, file_url_cache)
    }
}
