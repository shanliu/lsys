// Web 文件收集器

use std::sync::Arc;

use lsys_core::app_core::AppCore;
use lsys_file::dao::FileDao;
use lsys_file_manager::FileCollector;
use lsys_logger::dao::ChangeLoggerDao;
use sqlx::{MySql, Pool};

use crate::dao::result::WebResult;

/// Web 文件收集器
pub struct WebCollector {
    pub collector: Arc<FileCollector>,
}

impl WebCollector {
    pub fn new(
        db: Pool<MySql>,
        file_dao: Arc<FileDao>,
        change_logger: Arc<ChangeLoggerDao>,
        app_core: &AppCore,
    ) -> WebResult<Self> {
        let collector = Arc::new(FileCollector::new(db, file_dao, change_logger, app_core)?);

        // 启动 collector 后台任务
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

        Ok(Self { collector })
    }
}
