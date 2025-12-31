mod adapter;
mod logger;
mod message_cancel;
mod message_logs;
mod message_reader;
mod message_tpls;
mod sender_config;
mod sender_mailer;
mod sender_smser;
mod sender_task;
mod sender_tpl_config;
mod sender_wait;
pub use adapter::*;
pub use message_cancel::*;
pub use message_logs::*;
pub use message_reader::*;
pub use message_tpls::*;
pub use sender_config::*;
pub use sender_mailer::*;
pub use sender_smser::*;
pub use sender_task::*;
pub use sender_tpl_config::*;
pub(crate) use sender_wait::*;
mod result;
pub use result::*;

pub fn log_types() -> Vec<&'static str> {
    use logger::{LogAppConfig, LogMessage, LogMessageTpls, LogSenderConfig};
    use lsys_logger::dao::ChangeLogData;
    vec![
        LogMessage::log_type(),
        LogMessageTpls::log_type(),
        LogAppConfig::log_type(),
        LogSenderConfig::log_type(),
    ]
}
