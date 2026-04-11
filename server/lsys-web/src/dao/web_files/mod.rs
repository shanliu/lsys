//文件模块封装

mod export_task;
pub mod upload_token;

use std::sync::Arc;

use lsys_core::app_core::AppCore;
use lsys_file::dao::FileDao;
use lsys_logger::dao::ChangeLoggerDao;
use sqlx::{MySql, Pool};

// 使用 lsys-file-manager 中的类型
pub use self::export_task::{ExportCheckParam, WebExportTask, WebExporter};
use self::upload_token::UploadTokenDao;
use super::result::WebResult;
use lsys_file_manager::FileCollector;

// 重新导出 lsys-file-manager 的类型，供 web 层使用
pub use lsys_file_manager::ExportTask;
pub use lsys_file_manager::FileCollector as FileCollectorType;

// 重新导出 export_task 子模块
pub mod export_task_types {
    pub use lsys_file_manager::dao::export_task::exporter;
    pub use lsys_file_manager::dao::export_task::writer;
    pub use lsys_file_manager::dao::export_task::{
        ExportTaskFileItem, ExportTaskItem, ExportTaskListAttr,
    };
}

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
    pub collector: Arc<FileCollector>,
    pub export_task: Arc<WebExportTask>,
    db: Pool<MySql>,
}

impl WebFiles {
    pub fn new(
        db: Pool<MySql>,
        redis: deadpool_redis::Pool,
        app_core: &AppCore,
        file_dao: Arc<FileDao>,
        export_task: Arc<WebExportTask>,
        logger: Arc<ChangeLoggerDao>,
    ) -> WebResult<Self> {
        let upload_config = UploadConfig::from_config(app_core);
        let upload_token = Arc::new(UploadTokenDao::new(redis));
        let collector = Arc::new(FileCollector::new(
            db.clone(),
            file_dao.clone(),
            logger.clone(),
            app_core,
        )?);

        // 启动文件相关后台任务
        tokio::spawn({
            let d = file_dao.clone();
            async move {
                d.run_download_listener().await;
            }
        });
        tokio::spawn({
            let c = collector.clone();
            async move {
                c.run_task_loop().await;
            }
        });
        tokio::spawn({
            let c = collector.clone();
            async move {
                c.run_cache_cleanup().await;
            }
        });
        tokio::spawn({
            let export_task_bg = export_task.clone();
            async move {
                export_task_bg.dispatch_loop().await;
            }
        });

        Ok(Self {
            file_dao,
            upload_config,
            upload_token,
            collector,
            export_task,
            db,
        })
    }

    pub fn db(&self) -> &Pool<MySql> {
        &self.db
    }
}
