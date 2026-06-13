// Web 导出任务模块

mod export_check;
mod export_task;

use std::sync::Arc;

// 重新导出公共类型
pub use export_check::{WebExportCheckParam, WebExporterCheck};
pub use export_task::WebExportTask;

/// Web 导出任务
///
/// 简单包装 `Arc<WebExportTask>`，提供统一的访问接口
pub struct WebExport {
    pub export_task: Arc<WebExportTask>,
}

impl WebExport {
    /// 创建 Web 导出任务
    ///
    /// 直接接收已启动后台任务的 `Arc<WebExportTask>`
    pub fn new(export_task: Arc<WebExportTask>) -> Self {
        Self { export_task }
    }
}
