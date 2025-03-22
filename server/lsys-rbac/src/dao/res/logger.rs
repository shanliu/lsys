//RBAC中 操作日志
use lsys_logger::dao::ChangeLogData;
use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct LogRes<'t> {
    pub action: &'t str,
    pub res_name: &'t str,
    pub res_type: &'t str,
    pub app_id: u64,
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
pub(crate) struct LogResTypeOp<'t> {
    pub action: &'t str,
    pub res_type: &'t str,
    pub res_user_id: u64,
    pub res_app_id: u64,
    pub op_id_data: Vec<u64>,
}

impl ChangeLogData for LogResTypeOp<'_> {
    fn log_type() -> &'static str {
        "rbac-res-op"
    }
    fn message(&self) -> String {
        format!("{} {} [{:?}]", self.action, self.res_type, self.op_id_data)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}
