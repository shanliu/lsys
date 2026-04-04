// 导出任务操作变更日志

use lsys_logger::dao::ChangeLogData;
use serde::Serialize;

/// 导出任务操作日志
#[derive(Serialize)]
pub(crate) struct LogExportTask<'t> {
    pub action: &'t str,
    pub task_id: u64,
    pub app_id: u64,
    pub user_id: u64,
    pub add_user_id: u64,
    pub export_type: &'t str,
}

impl ChangeLogData for LogExportTask<'_> {
    fn log_type() -> &'static str {
        "export-task"
    }
    fn message(&self) -> String {
        format!(
            "{} export_task {} [type={}]",
            self.action, self.task_id, self.export_type
        )
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}
