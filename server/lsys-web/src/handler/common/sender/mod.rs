mod mailer;
mod mailer_smtp;
mod smser;
mod tpl;
mod tpl_config;
use lsys_logger::dao::ChangeLogData;
pub use mailer::*;
pub use mailer_smtp::*;
use serde::Serialize;
pub use smser::*;
pub use tpl::*;
pub use tpl_config::*;

mod smser_aliyun;
mod smser_cloopen;
mod smser_hwyun;
mod smser_jdyun;
mod smser_netease;
mod smser_tenyun;
pub use smser_aliyun::*;
pub use smser_cloopen::*;
pub use smser_hwyun::*;
pub use smser_jdyun::*;
pub use smser_netease::*;
pub use smser_tenyun::*;

#[derive(Serialize)]
pub(crate) struct MessageView {
    pub msg_type: &'static str,
    pub id: u64,
}

impl ChangeLogData for MessageView {
    fn log_type<'t>() -> &'t str {
        "message-view"
    }
    fn message(&self) -> String {
        format!("see {} data on {}", self.msg_type, self.id)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}
