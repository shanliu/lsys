use lsys_logger::dao::ChangeLogData;
use serde::Serialize;

//发送系统日志

#[derive(Serialize)]
pub(crate) struct LogMessage {
    pub action: &'static str,
    pub sender_type: i8,
    pub body_id: u64,
    pub message_id: Option<u64>,
}

impl ChangeLogData for LogMessage {
    fn log_type<'t>() -> &'t str {
        "sender-message"
    }
    fn message(&self) -> String {
        format!("{} {}", self.sender_type, self.action)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}

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
        format!("{} message tpl ", self.action)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}

#[derive(Serialize)]
pub(crate) struct LogAppConfig {
    pub action: &'static str,
    pub sender_type: u8,
    pub app_id: u64,
    pub name: String,
    pub tpl_id: String,
    pub setting_id: u64,
    pub config_data: String,
}

impl ChangeLogData for LogAppConfig {
    fn log_type<'t>() -> &'t str {
        "sender-app-config"
    }
    fn message(&self) -> String {
        format!("{} app {} config {} ", self.action, self.app_id, self.name)
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
