// 采集脚本操作变更日志

use lsys_logger::dao::ChangeLogData;
use serde::Serialize;

/// 脚本 CRUD 操作日志
#[derive(Serialize)]
pub(crate) struct LogCollectorScript<'t> {
    pub action: &'t str,
    pub script_id: u64,
    pub user_id: u64,
    pub app_id: u64,
    pub name: &'t str,
}

impl ChangeLogData for LogCollectorScript<'_> {
    fn log_type() -> &'static str {
        "collector-script"
    }
    fn message(&self) -> String {
        format!("{} script {} [{}]", self.action, self.name, self.script_id)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}

/// 任务提交操作日志
#[derive(Serialize)]
pub(crate) struct LogCollectorTask<'t> {
    pub action: &'t str,
    pub record_id: u64,
    pub task_id: u64,
    pub script_id: u64,
    pub script_name: &'t str,
    pub user_id: u64,
    pub app_id: u64,
    pub request_id: &'t str,
}

impl ChangeLogData for LogCollectorTask<'_> {
    fn log_type() -> &'static str {
        "collector-task"
    }
    fn message(&self) -> String {
        format!(
            "{} task record={} task={} script={} [{}]",
            self.action, self.record_id, self.task_id, self.script_name, self.script_id
        )
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}
