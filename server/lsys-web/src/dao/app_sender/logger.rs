use lsys_logger::dao::ChangeLogData;
use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct MessageView {
    pub msg_type: &'static str,
    pub body_id: u64,
    pub user_id: u64,
}

impl ChangeLogData for MessageView {
    fn log_type() -> &'static str {
        "message-view"
    }
    fn message(&self) -> String {
        format!("see {} data on {}", self.msg_type, self.body_id)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}
