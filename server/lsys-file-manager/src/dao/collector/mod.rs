// JS 采集下载功能 DAO — 模块入口
//
// 拆分为以下子模块：
//   script  — 脚本 CRUD + 列表 + 计数
//   task    — 提交采集任务
//   record  — 执行记录查询 + 计数
//   log     — 日志写入 + 查询

mod log;
pub(crate) mod logger;
mod record;
mod script;
mod task;

pub use record::{CollectorRecordItem, CollectorRecordListAttr, RecordFileItem, RecordFileTag};
pub use script::{ScriptFileItem, ScriptFileTag};

use std::path::PathBuf;
use std::sync::Arc;

use lsys_core::fluent_message;
use lsys_file::dao::FileDao;
use lsys_lib_jsrun::runner::JsTaskRunner;
use lsys_lib_jsrun::{EngineConfig, JsEngine, RuntimeConfig};
use lsys_logger::dao::ChangeLoggerDao;
use sqlx::{MySql, Pool};

use crate::dao::result::{FileManagerError, FileManagerResult};

/// 采集功能配置
#[derive(Clone)]
pub struct CollectorConfig {
    /// JS 引擎最大并发运行时数
    pub max_runtimes: usize,
    /// 采集文件工作目录
    pub work_base_dir: PathBuf,
    /// 脚本最大执行时间（秒），0 表示不限制
    pub max_timeout_secs: u32,
    /// 脚本最大内存使用（字节），0 表示不限制
    pub max_memory_limit: u64,
}

impl Default for CollectorConfig {
    fn default() -> Self {
        Self {
            max_runtimes: 4,
            work_base_dir: std::env::temp_dir().join("lsys-collector"),
            max_timeout_secs: 0,
            max_memory_limit: 0,
        }
    }
}

impl CollectorConfig {
    pub fn from_config(app_core: &lsys_core::app_core::AppCore) -> Self {
        let config = lsys_core::config!(app_core.config);
        let defaults = Self::default();
        Self {
            max_runtimes: config
                .get_int("collector_max_runtimes")
                .map(|v| v as usize)
                .unwrap_or(defaults.max_runtimes),
            work_base_dir: config
                .get_string("collector_work_base_dir")
                .map(PathBuf::from)
                .unwrap_or(defaults.work_base_dir),
            max_timeout_secs: config
                .get_int("collector_max_timeout_secs")
                .map(|v| v as u32)
                .unwrap_or(defaults.max_timeout_secs),
            max_memory_limit: config
                .get_int("collector_max_memory_limit")
                .map(|v| v as u64)
                .unwrap_or(defaults.max_memory_limit),
        }
    }
}

/// JS 采集 DAO
pub struct FileCollector {
    pub(crate) db: Pool<MySql>,
    pub(crate) runner: JsTaskRunner,
    pub(crate) file_dao: Arc<FileDao>,
    pub(crate) logger: Arc<ChangeLoggerDao>,
    pub(crate) config: CollectorConfig,
}

impl FileCollector {
    /// 创建采集器。
    pub fn new(
        db: Pool<MySql>,
        file_dao: Arc<FileDao>,
        logger: Arc<ChangeLoggerDao>,
        app_core: &lsys_core::app_core::AppCore,
    ) -> FileManagerResult<Self> {
        let config = CollectorConfig::from_config(app_core);

        let engine_config = EngineConfig {
            max_runtimes: config.max_runtimes,
            ..EngineConfig::default()
        };

        let engine = JsEngine::new(engine_config).map_err(|e| {
            FileManagerError::Message(fluent_message!("collector-engine-init-error", e))
        })?;
        let runner = JsTaskRunner::new(engine, RuntimeConfig::default());

        Ok(Self {
            db,
            runner,
            file_dao,
            logger,
            config,
        })
    }

    /// 运行 JS 任务派发后台循环。通常通过 `tokio::spawn` 调用。
    pub async fn run_task_loop(&self, cancel_token: tokio_util::sync::CancellationToken) {
        self.runner.run(cancel_token).await;
    }

    /// 运行 JS 引擎缓存清理后台循环。通常通过 `tokio::spawn` 调用。
    pub async fn run_cache_cleanup(&self, cancel_token: tokio_util::sync::CancellationToken) {
        self.runner.run_engine_cleanup(cancel_token).await;
    }

    /// 从 RequestEnv 提取 request_id，若不存在则自动生成
    pub fn resolve_request_id(req_env: &lsys_core::utils::RequestEnv) -> String {
        match &req_env.request_id {
            Some(rid) if !rid.trim().is_empty() => rid.trim().to_string(),
            _ => lsys_core::utils::rand_str(lsys_core::utils::RandType::LowerHex, 32),
        }
    }

    /// 获取文件 DAO 引用（供 handler 层查询关联文件）
    pub fn file_dao(&self) -> &Arc<FileDao> {
        &self.file_dao
    }
}
