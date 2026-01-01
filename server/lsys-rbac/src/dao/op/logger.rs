//RBAC中 操作日志
use lsys_logger::dao::ChangeLogData;
use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct LogOp<'t> {
    pub action: &'t str,
    pub op_name: &'t str,
    pub op_key: &'t str,
    pub user_id: u64,
    pub app_id: u64,
}

impl ChangeLogData for LogOp<'_> {
    fn log_type() -> &'static str {
        "rbac-op"
    }
    fn message(&self) -> String {
        format!("{} {} [{}]", self.action, self.op_name, self.op_key)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}
