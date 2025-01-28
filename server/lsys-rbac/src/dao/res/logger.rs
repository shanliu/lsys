//RBAC中 操作日志
use lsys_logger::dao::ChangeLogData;
use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct LogRes<'t> {
    pub action: &'t str,
    pub res_name: &'t str,
    pub res_type: &'t str,
    pub res_data: &'t str,
    pub user_id: u64,
}

impl ChangeLogData for LogRes<'_> {
    fn log_type() -> &'static str {
        "rbac-res"
    }
    fn message(&self) -> String {
        format!(
            "{} {} {} [{}]",
            self.action, self.res_name, self.res_type, self.res_data
        )
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}

#[derive(Serialize)]
pub(crate) struct LogResOp<'t> {
    pub action: &'t str,
    pub res_type: &'t str,
    pub res_data: Vec<u64>,
    pub user_id: u64,
}

impl ChangeLogData for LogResOp<'_> {
    fn log_type() -> &'static str {
        "rbac-res-op"
    }
    fn message(&self) -> String {
        format!("{} {} [{:?}]", self.action, self.res_type, self.res_data)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}
