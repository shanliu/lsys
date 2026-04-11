// Web 层导出任务查询和权限检查
//
// 提供带权限检查的导出任务提交功能

use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::{ExportTask, WebResult};
use lsys_core::utils::RequestEnv;
use lsys_file_manager::dao::export_task::exporter::Exporter;

/// 导出权限检查参数
///
/// 封装导出任务权限检查所需的上下文信息
#[derive(Debug, Clone)]
pub struct ExportCheckParam<'a> {
    /// 应用 ID
    pub app_id: u64,
    /// 应用用户 ID
    pub app_user_id: u64,
    /// 用户 ID
    pub user_id: u64,
    /// 导出类型
    pub export_type: &'a str,
    /// 导出参数
    pub params: &'a serde_json::Value,
}

/// Web 层导出器 trait
///
/// 继承自 lsys-file-manager 的 Exporter trait，并添加权限检查方法
/// 使用 WebError 作为错误类型
#[async_trait::async_trait]
pub trait WebExporter: Exporter<crate::dao::WebError> {
    /// 权限检查
    ///
    /// 在执行导出前检查用户是否有权限执行此导出操作
    ///
    /// # 参数
    /// - `check_env`: RBAC 访问检查环境
    /// - `param`: 导出检查参数
    async fn check(
        &self,
        check_env: &RbacAccessCheckEnv<'_>,
        param: &ExportCheckParam<'_>,
    ) -> WebResult<()>;
}

/// Web 层导出任务管理器
///
/// 包装 lsys-file-manager 的 ExportTask，添加权限检查功能
pub struct WebExportTask {
    /// 底层导出任务管理器
    export_task: Arc<ExportTask>,
    /// 导出器映射表（包含权限检查逻辑）
    web_exporters: HashMap<String, Arc<dyn WebExporter + Send + Sync>>,
}

impl WebExportTask {
    /// 创建新的 Web 导出任务管理器
    pub fn new(export_task: Arc<ExportTask>) -> Self {
        Self {
            export_task,
            web_exporters: HashMap::new(),
        }
    }

    /// 注册导出器
    ///
    /// 注册一个实现了 WebExporter trait 的导出器
    pub fn register(
        &mut self,
        export_type: &str,
        exporter: Arc<dyn WebExporter + Send + Sync>,
    ) -> WebResult<()> {
        self.web_exporters.insert(export_type.to_string(), exporter);
        Ok(())
    }

    /// 提交导出任务（带权限检查）
    ///
    /// 在提交任务前会先调用对应导出器的 check() 方法进行权限检查
    #[allow(clippy::too_many_arguments)]
    pub async fn submit(
        &self,
        check_env: &RbacAccessCheckEnv<'_>,
        app_id: u64,
        app_user_id: u64,
        user_id: u64,
        export_type: &str,
        params: serde_json::Value,
        env_data: Option<&RequestEnv>,
    ) -> WebResult<u64> {
        // 1. 查找对应的导出器
        let exporter = self.web_exporters.get(export_type).ok_or_else(|| {
            crate::dao::WebError::Message(lsys_core::fluent_message!(
                "export-type-not-found",
                format!("Export type '{}' not found", export_type)
            ))
        })?;

        // 2. 执行权限检查
        let check_param = ExportCheckParam {
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

        Ok(self
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
                },
                env_data,
            )
            .await?)
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
