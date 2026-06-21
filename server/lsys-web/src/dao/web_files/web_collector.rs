// Web 文件收集器

use std::sync::Arc;

use lsys_core::app_core::AppCore;
use lsys_core::task_lifecycle::TaskNode;
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
        task_node: Arc<TaskNode>,
    ) -> WebResult<Self> {
        let collector = Arc::new(FileCollector::new(db, file_dao, change_logger, app_core)?);

        // 启动 collector 后台任务
        let task_loop_node = task_node.child("collector-task-loop");
        let c_task = collector.clone();
        task_loop_node.spawn(move |token| {
            async move {
                c_task.run_task_loop(token).await;
            }
        });

        let cache_cleanup_node = task_node.child("collector-cache-cleanup");
        let c_cleanup = collector.clone();
        cache_cleanup_node.spawn(move |token| {
            async move {
                c_cleanup.run_cache_cleanup(token).await;
            }
        });

        Ok(Self { collector })
    }
}
