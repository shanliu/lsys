use std::{ops::Deref, sync::Arc};

use lsys_core::{AppCore, WaitItem, WaitNotify, WaitNotifyResult};
use tokio::sync::oneshot::Receiver;
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct SenderWaitItem {
    body_id: u64,
    message_snid: u64,
}

impl WaitItem for SenderWaitItem {
    fn eq(&self, mq_wait_item: &Self) -> bool {
        (self.message_snid == mq_wait_item.message_snid)//某个消息ID相同
            || (self.body_id == mq_wait_item.body_id//内容相同且mq通知未标明具体消息ID
                && mq_wait_item.message_snid == 0)
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
    pub async fn message_wait(
        &self,
        body_id: u64,
        message_snid: u64,
    ) -> Receiver<WaitNotifyResult> {
        self.wait(SenderWaitItem {
            body_id,
            message_snid,
        })
        .await
    }
    pub async fn body_notify(&self, host: &str, body_id: u64, res: WaitNotifyResult) {
        if host.is_empty() {
            return;
        }
        //整个消息发送完成通知
        let _ = self
            .0
            .notify(
                host,
                SenderWaitItem {
                    body_id,
                    message_snid: 0,
                },
                res,
            )
            .await;
    }
    pub async fn msg_notify(&self, host: &str, msg_snid: u64, res: WaitNotifyResult) {
        if host.is_empty() {
            return;
        }
        //单个消息发送完成通知
        let _ = self
            .0
            .notify(
                host,
                SenderWaitItem {
                    body_id: 0,
                    message_snid: msg_snid,
                },
                res,
            )
            .await;
    }
}
