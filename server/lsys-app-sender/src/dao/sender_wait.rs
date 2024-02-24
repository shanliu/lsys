use std::{ops::Deref, sync::Arc};

use lsys_core::{AppCore, WaitItem, WaitNotify, WaitNotifyResult};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct SenderWaitItem(pub u64, pub u64);

impl WaitItem for SenderWaitItem {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 || self.1 == other.1
    }
}

pub(crate) struct SenderWaitNotify(WaitNotify<SenderWaitItem>);

impl Deref for SenderWaitNotify {
    type Target = WaitNotify<SenderWaitItem>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SenderWaitNotify {
    pub fn new(
        channel_name: &str,
        redis: deadpool_redis::Pool,
        app_core: Arc<AppCore>,
        clear_timeout: u8,
    ) -> Self {
        SenderWaitNotify(WaitNotify::<SenderWaitItem>::new(
            channel_name,
            redis,
            app_core,
            clear_timeout,
        ))
    }
    pub async fn body_notify(&self, host: &str, body_id: u64, res: WaitNotifyResult) {
        if host.is_empty() {
            return;
        }
        let _ = self.0.notify(host, SenderWaitItem(body_id, 0), res).await;
    }
    pub async fn msg_notify(&self, host: &str, msg_id: u64, res: WaitNotifyResult) {
        if host.is_empty() {
            return;
        }
        let _ = self.0.notify(host, SenderWaitItem(0, msg_id), res).await;
    }
}
