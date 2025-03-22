use lsys_app_notify::dao::{NotifyDao, NotifyData};
use lsys_app_sender::dao::NotifySmsItem;
use lsys_core::IntoFluentMessage;
use std::sync::Arc;
use tracing::error;
pub struct AppNotify {
    pub notify_dao: Arc<NotifyDao>,
}

impl AppNotify {
    pub fn new(notify_dao: Arc<NotifyDao>) -> Self {
        let notify_task = notify_dao.clone();
        tokio::spawn(async move {
            if let Err(err) = notify_task.task().await {
                error!("notify error:{}", err.to_fluent_message().default_format())
            }
        });
        AppNotify { notify_dao }
    }
    //返回所有支持的通知方式
    pub fn notify_method_list(&self) -> Vec<String> {
        vec![NotifySmsItem::method()]
    }
}
