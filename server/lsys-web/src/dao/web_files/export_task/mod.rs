// 数据导出任务模块
//
// 拆分为以下子模块：
//   exporter  — 统一导出 Trait 定义
//   exporters — 内置 Exporter 实现集合（各列表接口对应的 CSV 导出器）
//   task      — 调度循环 + 单任务执行
//   query     — 提交 / 查询 / 超时检测 / 软删除

pub mod exporter;
pub(crate) mod logger;
mod query;
mod task;

pub use query::{ExportTaskFileItem, ExportTaskItem, ExportTaskListAttr};
pub mod writer;
use std::collections::HashMap;
use std::sync::Arc;

use lsys_files::dao::FileDao;
use lsys_logger::dao::ChangeLoggerDao;
use sqlx::{MySql, Pool};
use tokio::sync::{mpsc, Mutex, Semaphore};

use crate::dao::export_task::exporter::Exporter;

/// 导出任务配置
#[derive(Clone, Debug)]
pub struct ExportTaskConfig {
    /// 最大并发导出数
    pub limit_branch: u64,
    /// Running 超时秒数，超过后标记为 Failed
    pub timeout_secs: u64,
}

impl Default for ExportTaskConfig {
    fn default() -> Self {
        Self {
            limit_branch: 4,
            timeout_secs: 3600,
        }
    }
}

impl ExportTaskConfig {
    /// 从 AppCore 配置加载
    ///
    /// 配置键：
    /// - `export_task_limit_branch`: 最大并发导出数，默认 4
    /// - `export_task_timeout_secs`: Running 超时秒数，默认 3600
    pub fn from_config(app_core: &lsys_core::app_core::AppCore) -> Self {
        let config = lsys_core::config!(app_core.config);
        let defaults = Self::default();
        Self {
            limit_branch: config
                .get_int("export_task_limit_branch")
                .map(|v| v as u64)
                .unwrap_or(defaults.limit_branch),
            timeout_secs: config
                .get_int("export_task_timeout_secs")
                .map(|v| v as u64)
                .unwrap_or(defaults.timeout_secs),
        }
    }
}

/// 数据导出任务管理器
pub struct WebExportTask {
    pub(crate) db: Pool<MySql>,
    pub(crate) file_dao: Arc<FileDao>,
    pub(crate) logger: Arc<ChangeLoggerDao>,
    pub(crate) config: ExportTaskConfig,
    /// 已注册的 exporter，key = export_type
    pub(crate) exporters: Arc<HashMap<String, Box<dyn Exporter>>>,
    /// 并发控制信号量
    pub(crate) semaphore: Arc<Semaphore>,
    /// 触发信号发送端
    pub(crate) trigger_tx: mpsc::Sender<()>,
    /// 触发信号接收端（Mutex 包装，供 dispatch_loop 独占使用）
    pub(crate) trigger_rx: Mutex<mpsc::Receiver<()>>,
}

impl WebExportTask {
    /// 创建导出任务管理器
    pub fn new(
        db: Pool<MySql>,
        file_dao: Arc<FileDao>,
        logger: Arc<ChangeLoggerDao>,
        app_core: &lsys_core::app_core::AppCore,
    ) -> Self {
        let config = ExportTaskConfig::from_config(app_core);
        let semaphore = Arc::new(Semaphore::new(config.limit_branch as usize));
        let (trigger_tx, trigger_rx) = mpsc::channel(config.limit_branch as usize);

        Self {
            db,
            file_dao,
            logger,
            config,
            exporters: Arc::new(HashMap::new()),
            semaphore,
            trigger_tx,
            trigger_rx: Mutex::new(trigger_rx),
        }
    }

    /// 注册导出器
    ///
    /// `export_type` 与 `Exporter` 实现自由组合，
    /// 同一个实现可以注册到不同的 `export_type` 下。
    ///
    /// ```ignore
    /// manager.register("collector_record", Arc::new(MyCsvExporter));
    /// manager.register("user_list", Arc::new(MyCsvExporter));
    /// ```
    pub fn register(
        &mut self,
        export_type: &str,
        exporter: Box<dyn Exporter>,
    ) -> Result<(), String> {
        match Arc::get_mut(&mut self.exporters) {
            Some(map) => {
                map.insert(export_type.to_string(), exporter);
                Ok(())
            }
            None => Err("cannot register exporter after dispatch loop started".to_string()),
        }
    }

    /// 获取已注册的 export_type 列表
    pub fn registered_types(&self) -> Vec<&str> {
        self.exporters.keys().map(|k| k.as_str()).collect()
    }
}
