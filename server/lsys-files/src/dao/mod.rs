mod file_config;
mod file_dao;
mod file_download;
mod file_helpers;
mod file_log;
mod logger;

pub use file_config::*;
pub use file_dao::*;
pub use file_download::*;
pub use file_helpers::*;
pub use file_log::*;

// Re-export common types
pub use crate::common::*;

use lsys_core::AppCore;
use lsys_logger::dao::ChangeLoggerDao;
use sqlx::{MySql, Pool};
use std::sync::Arc;

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
    /// 需要在 tokio runtime 中调用 (因为下载管理器和清理管理器需要 spawn)
    pub fn build(db: Pool<MySql>, app_core: &AppCore, logger: Arc<ChangeLoggerDao>) -> FileDao {
        let config = FileConfig::from_config(app_core);
        Self::build_with_config(db, config, logger)
    }

    /// 构建文件 DAO（直接传入配置）
    /// 需要在 tokio runtime 中调用 (因为下载管理器需要 spawn)
    pub fn build_with_config(db: Pool<MySql>, config: FileConfig, logger: Arc<ChangeLoggerDao>) -> FileDao {
        let helper = Arc::new(FileHelper::new(db.clone(), config.clone()));
        let download = Arc::new(FileDownloadManager::new(helper.clone()));

        FileDao::new(helper, download, logger)
    }
}
