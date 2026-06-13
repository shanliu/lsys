// 数据导出任务模块
//
// 拆分为以下子模块：
//   exporter  — 统一导出 Trait 定义
//   task      — 调度循环 + 单任务执行
//   query     — 提交 / 查询 / 超时检测 / 软删除

pub mod exporter;
pub(crate) mod logger;
mod query;
mod task;

pub use query::{ExportTaskFileItem, ExportTaskItem, SubmitExportTaskParam};
pub mod writer;

use std::collections::HashMap;
use std::sync::Arc;

use lsys_core::fluents::FluentMgr;
use lsys_file::dao::FileDao;
use lsys_logger::dao::ChangeLoggerDao;
use sqlx::{MySql, Pool};
use tokio::sync::{Semaphore, mpsc};
use tracing::warn;

use crate::dao::export_task::exporter::{Exporter, ExporterAdapter};

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
pub struct ExportTask {
    pub(crate) db: Pool<MySql>,
    pub(crate) file_dao: Arc<FileDao>,
    pub(crate) logger: Arc<ChangeLoggerDao>,
    pub(crate) config: ExportTaskConfig,
    /// 已注册的 exporter，key = export_type
    /// 使用 FileManagerError 作为统一的错误类型
    pub(crate) exporters: Arc<HashMap<String, Box<dyn Exporter<crate::dao::FileManagerError>>>>,
    /// 并发控制信号量
    pub(crate) semaphore: Arc<Semaphore>,
    /// 触发信号发送端
    pub(crate) trigger_tx: mpsc::Sender<()>,
    /// 触发信号接收端（Option 包装，可以被 take 出来给 dispatcher）
    pub(crate) trigger_rx: Option<mpsc::Receiver<()>>,
    /// 多语言管理器，供 Exporter::export() 解析 locale
    pub(crate) fluent_mgr: Arc<FluentMgr>,
}

impl ExportTask {
    /// 创建导出任务管理器
    pub fn new(
        db: Pool<MySql>,
        file_dao: Arc<FileDao>,
        logger: Arc<ChangeLoggerDao>,
        app_core: &lsys_core::app_core::AppCore,
        fluent_mgr: Arc<FluentMgr>,
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
            trigger_rx: Some(trigger_rx),
            fluent_mgr,
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
    pub fn register<E>(
        &mut self,
        export_type: &str,
        exporter: impl Exporter<E> + 'static,
    ) -> Result<(), String>
    where
        E: Into<crate::dao::FileManagerError> + Send + 'static,
    {
        match Arc::get_mut(&mut self.exporters) {
            Some(map) => {
                // 创建适配器，将 Exporter<E> 转换为 Exporter<FileManagerError>
                let adapter = ExporterAdapter {
                    inner: Box::new(exporter),
                };
                map.insert(export_type.to_string(), Box::new(adapter));
                Ok(())
            }
            None => Err("cannot register exporter after dispatch loop started".to_string()),
        }
    }

    /// 获取已注册的 export_type 列表
    pub fn registered_types(&self) -> Vec<&str> {
        self.exporters.keys().map(|k| k.as_str()).collect()
    }
     /// 触发导出：向 channel 发送信号
    ///
    /// 多处可调用（submit 后、定时任务等），
    /// 内部只是发信号，不阻塞。
    pub fn trigger(&self) {
        if let Err(e) = self.trigger_tx.try_send(()) {
            warn!("export_task: trigger send failed: {}", e);
        }
    }

    /// 创建调度器
    ///
    /// 从 `ExportTask` 中 take 出 `trigger_rx`，并 clone/Arc 共享其他字段。
    /// 调用此方法后，`trigger_rx` 将变为 `None`。
    ///
    /// # 返回
    /// 返回 `Option<task::ExportTaskDispatcher>`：
    /// - `Some(dispatcher)`: 成功创建调度器
    /// - `None`: `trigger_rx` 已被 take（调度器已创建过）
    pub fn create_dispatcher(&mut self) -> Option<task::ExportTaskDispatcher> {
        let trigger_rx = self.trigger_rx.take()?;
        
        Some(task::ExportTaskDispatcher {
            db: self.db.clone(),
            file_dao: Arc::clone(&self.file_dao),
            config: self.config.clone(),
            exporters: Arc::clone(&self.exporters),
            semaphore: Arc::clone(&self.semaphore),
            trigger_rx,
            fluent_mgr: Arc::clone(&self.fluent_mgr),
        })
    }
}
