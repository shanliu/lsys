// Web 层导出任务管理器
//
// 包装 lsys-file-manager 的 ExportTask，添加权限检查功能

use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

use crate::dao::result::{WebError, WebResult};
use crate::dao::ExportTask;
use crate::dao::web_files::web_export::export_check::WebExporterCheck;
use crate::dao::web_files::web_export::export_check::WebExportCheckParam;
use lsys_core::app_core::AppCore;
use lsys_core::fluents::FluentMgr;
use lsys_core::task_lifecycle::TaskNode;
use lsys_core::utils::RequestEnv;
use lsys_file::dao::FileDao;
use lsys_file_manager::export_task::exporter::Exporter;
use lsys_logger::dao::ChangeLoggerDao;
use sqlx::{MySql, Pool};


/// Web 层导出任务管理器
///
/// 包装 lsys-file-manager 的 ExportTask，添加权限检查功能
pub struct WebExportTask {
    /// 底层导出任务管理器（直接拥有，不使用 Arc）
    export_task: ExportTask,
    /// Web 层导出器注册表（用于权限检查）
    export_checks: HashMap<String, Arc<dyn WebExporterCheck>>,
    /// 多语言管理器，供注册导出器时分发
    pub fluent_mgr: Arc<FluentMgr>,
}

impl WebExportTask {
    /// 创建新的 Web 导出任务管理器
    ///
    /// 在内部初始化 ExportTask，但不启动后台调度循环。
    /// 需要在注册完所有导出器后调用 `start_dispatch()` 启动后台任务。
    ///
    /// # 参数
    /// - `db`: 数据库连接池
    /// - `file_dao`: 文件管理 DAO
    /// - `logger`: 变更日志 DAO
    /// - `app_core`: 应用核心配置
    pub fn new(
        db: Pool<MySql>,
        file_dao: Arc<FileDao>,
        logger: Arc<ChangeLoggerDao>,
        app_core: &AppCore,
        fluent_mgr: Arc<FluentMgr>,
    ) -> Self {
        let export_task = ExportTask::new(db, file_dao, logger, app_core, fluent_mgr.clone());

        Self {
            export_task,
            export_checks: HashMap::new(),
            fluent_mgr,
        }
    }

    /// 启动后台调度循环
    ///
    /// 必须在注册完所有导出器后调用。
    /// 创建调度器并启动后台任务。
    pub fn start_dispatch(&mut self, task_node: Arc<TaskNode>) {
        if let Some(dispatcher) = self.export_task.create_dispatcher() {
            let node = task_node.child("export-dispatch");
            node.spawn(|token| async move {
                dispatcher.dispatch_loop(token).await;
            });
        }
    }

    /// 注册导出器和权限检查器
    ///
    /// 分别注册底层的 Exporter 和 Web 层的权限检查器。
    ///
    /// # 参数
    /// - `export_type`: 导出类型标识
    /// - `exporter`: 底层导出器实现（必须使用 WebError）
    /// - `check`: Web 层权限检查器
    pub fn register(
        &mut self,
        export_type: &str,
        exporter: impl Exporter<WebError> + 'static,
        check: impl WebExporterCheck + 'static,
    ) -> WebResult<()> {
        // 注册底层导出器
        self.export_task
            .register(export_type, exporter)
            .map_err(|e| WebError::Message(lsys_core::fluent_message!(
                "export-register-failed",
                e
            )))?;
        
        // 注册权限检查器
        self.export_checks.insert(export_type.to_owned(), Arc::new(check));
        
        Ok(())
    }

   

    /// 提交导出任务（带权限检查）
    ///
    /// 在提交任务前会先调用对应导出器的 check() 方法进行权限检查
    #[allow(clippy::too_many_arguments)]
    pub async fn submit(
        &self,
        check_env: &crate::dao::access::RbacAccessCheckEnv<'_>,
        app_id: u64,
        app_user_id: u64,
        user_id: u64,
        export_type: &str,
        params: serde_json::Value,
        env_data: Option<&RequestEnv>,
    ) -> WebResult<u64> {
        // 1. 查找对应的 Web 导出器
        let exporter = self.export_checks.get(export_type)
            .ok_or_else(|| WebError::Message(lsys_core::fluent_message!(
                "export-type-not-found",
                format!("Export type '{}' not found", export_type)
            )))?;

        // 2. 执行权限检查
        let check_param = WebExportCheckParam {
            app_id,
            app_user_id,
            user_id,
            export_type,
            params: &params,
        };
        exporter.check(check_env, &check_param).await?;

        // 3. 权限检查通过后，调用底层的 submit 方法
        let request_id = env_data
            .and_then(|env| env.request_id.as_deref())
            .unwrap_or("");
        let lang = env_data
            .and_then(|env| env.request_lang.as_deref())
            .unwrap_or("");
        let data = self
            .export_task
            .submit(
                lsys_file_manager::SubmitExportTaskParam {
                    app_id,
                    app_user_id,
                    user_id,
                    add_user_id: user_id,
                    export_type,
                    params: &params,
                    request_id,
                    lang,
                },
                env_data,
            )
            .await?;
        Ok(data)
    }
}

/// 实现 Deref，允许直接访问底层 ExportTask 的方法
///
/// 这样可以直接调用 list_tasks、count_tasks 等查询方法，
/// 而 submit 方法被 WebExportTask 重写以添加权限检查
impl Deref for WebExportTask {
    type Target = ExportTask;

    fn deref(&self) -> &Self::Target {
        &self.export_task
    }
}
