use lsys_logger::dao::ChangeLogData;
use serde::Serialize;

//发送系统日志

#[derive(Serialize)]
pub(crate) struct LogMessage<'t> {
    pub action: &'t str,
    pub sender_type: i8,
    pub user_id: u64,
    pub body_id: u64,
    pub message_id: Option<u64>,
}

impl ChangeLogData for LogMessage<'_> {
    fn log_type() -> &'static str {
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
pub(crate) struct LogMessageTpls<'t> {
    pub action: &'t str,
    pub sender_type: i8,
    pub app_id: u64,
    pub tpl_id: &'t str,
    pub user_id: u64,
    pub tpl_data: &'t str,
}

impl ChangeLogData for LogMessageTpls<'_> {
    fn log_type() -> &'static str {
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
pub(crate) struct LogAppConfig<'t> {
    pub action: &'t str,
    pub sender_type: u8,
    pub app_id: u64,
    pub name: &'t str,
    pub tpl_key: &'t str,
    pub setting_id: u64,
    pub config_data: &'t str,
    pub user_id: u64,
}

impl ChangeLogData for LogAppConfig<'_> {
    fn log_type() -> &'static str {
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
pub(crate) struct LogSenderConfig<'t> {
    pub action: &'t str,
    pub app_id: u64,
    pub user_id: u64,
    pub priority: i8,
    pub sender_type: i8,
    pub config_type: i8,
    pub config_data: &'t str,
}

impl ChangeLogData for LogSenderConfig<'_> {
    fn log_type() -> &'static str {
        "sender-config"
    }
    fn message(&self) -> String {
        format!(" {} sender config :{} ", self.action, self.app_id,)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}
