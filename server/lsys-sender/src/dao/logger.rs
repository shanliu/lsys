use lsys_logger::dao::ChangeLogData;
use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct LogMessageTpls {
    pub action: &'static str,
    pub sender_type: i8,
    pub tpl_id: String,
    pub tpl_data: String,
}

impl ChangeLogData for LogMessageTpls {
    fn log_type<'t>() -> &'t str {
        "sender-tpl"
    }
    fn message(&self) -> String {
        format!("{} role tag ", self.action)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}

#[derive(Serialize)]
pub(crate) struct LogMailAppConfig {
    pub action: &'static str,
    pub app_id: u64,
    pub name: String,
    pub tpl_id: String,
    pub from_email: String,
    pub smtp_config_id: u64,
    pub subject_tpl_id: String,
    pub body_tpl_id: String,
    pub max_try_num: u16,
}

impl ChangeLogData for LogMailAppConfig {
    fn log_type<'t>() -> &'t str {
        "sender-mail-app-config"
    }
    fn message(&self) -> String {
        format!(
            "{} mail app {} config {} ",
            self.action, self.app_id, self.name
        )
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}

#[derive(Serialize)]
pub(crate) struct LogSenderConfig {
    pub action: &'static str,
    pub app_id: u64,
    pub priority: i8,
    pub sender_type: i8,
    pub config_type: i8,
    pub config_data: String,
}

impl ChangeLogData for LogSenderConfig {
    fn log_type<'t>() -> &'t str {
        "sender-config"
    }
    fn message(&self) -> String {
        format!(" {} sender config :{} ", self.action, self.app_id,)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}
