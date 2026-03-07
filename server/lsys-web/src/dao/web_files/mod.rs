//文件模块封装

pub mod collector;
pub mod upload_token;

use std::sync::Arc;

use lsys_core::app_core::AppCore;
use lsys_files::dao::{FileDao, FileDaoBuilder};
use lsys_logger::dao::ChangeLoggerDao;
use sqlx::{MySql, Pool};

use self::collector::WebFileCollector;
use self::upload_token::UploadTokenDao;
use super::result::WebResult;

/// 上传配置：最大文件大小 & 分片规则
#[derive(Debug, Clone)]
pub struct UploadConfig {
    /// 最大上传文件大小(字节), 默认 100MB
    pub max_upload_size: u64,
    /// 分片阈值：文件大小超过此值时必须分片上传(字节), 默认 5MB
    pub chunk_threshold: u64,
    /// 推荐分片大小(字节), 默认 2MB
    pub default_chunk_size: u64,
}

impl Default for UploadConfig {
    fn default() -> Self {
        Self {
            max_upload_size: 2048 * 1024 * 1024, // 2GB
            chunk_threshold: 5 * 1024 * 1024,    // 5MB
            default_chunk_size: 2 * 1024 * 1024, // 2MB
        }
    }
}

impl UploadConfig {
    /// 从 AppCore 配置加载，读取以下配置键：
    /// - file_max_upload_size: 最大上传文件大小(字节), 默认 100MB
    /// - file_chunk_threshold: 分片阈值(字节), 默认 5MB
    /// - file_default_chunk_size: 推荐分片大小(字节), 默认 2MB
    pub fn from_config(app_core: &AppCore) -> Self {
        let config = lsys_core::config!(app_core.config);
        let defaults = Self::default();
        Self {
            max_upload_size: config
                .get_int("file_max_upload_size")
                .map(|v| v as u64)
                .unwrap_or(defaults.max_upload_size),
            chunk_threshold: config
                .get_int("file_chunk_threshold")
                .map(|v| v as u64)
                .unwrap_or(defaults.chunk_threshold),
            default_chunk_size: config
                .get_int("file_default_chunk_size")
                .map(|v| v as u64)
                .unwrap_or(defaults.default_chunk_size),
        }
    }
}

pub struct WebFiles {
    pub file_dao: Arc<FileDao>,
    pub upload_config: UploadConfig,
    pub upload_token: Arc<UploadTokenDao>,
    pub collector: Arc<WebFileCollector>,
    db: Pool<MySql>,
}

impl WebFiles {
    pub fn new(
        db: Pool<MySql>,
        redis: deadpool_redis::Pool,
        app_core: &AppCore,
        logger: Arc<ChangeLoggerDao>,
    ) -> WebResult<Self> {
        let file_dao = Arc::new(FileDaoBuilder::build(db.clone(), app_core, logger.clone()));
        let upload_config = UploadConfig::from_config(app_core);
        let upload_token = Arc::new(UploadTokenDao::new(redis));
        let collector = Arc::new(
            WebFileCollector::new(db.clone(), file_dao.clone(), logger, app_core)?,
        );
        Ok(Self {
            file_dao,
            upload_config,
            upload_token,
            collector,
            db,
        })
    }

    pub fn db(&self) -> &Pool<MySql> {
        &self.db
    }
}
