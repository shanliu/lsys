use async_trait::async_trait;
use futures_util::StreamExt;

use redis::Client;
use tracing::{debug, error, info, warn};

#[async_trait]
pub trait LocalCacheClearItem {
    fn cache_name(&self) -> &str;
    async fn clear_from_message(&self, msg: &str) -> Result<(), String>;
}

/// 订阅远程通知清理本地缓存
pub struct LocalCacheClear {
    cache_list: Vec<Box<dyn LocalCacheClearItem + Sync + Send + 'static>>,
}

pub const REDIS_CHANNEL_NAME: &str = "user-clear-channel";

pub fn channel_message_create(channel_key: &'static str, message: String) -> String {
    let selfname = hostname::get().unwrap_or_default();
    let selfname = selfname.to_string_lossy().to_string();
    format!("{}\n{}\n{}", selfname, channel_key, message)
}

impl LocalCacheClear {
    pub fn new(cache_list: Vec<Box<dyn LocalCacheClearItem + Sync + Send + 'static>>) -> Self {
        LocalCacheClear { cache_list }
    }
    async fn clear_from_message(&self, notify_message: String) {
        let mut msg = notify_message.split('\n');
        let selfname = hostname::get().unwrap_or_default();
        match msg.next() {
            Some(host) => {
                if *host == selfname {
                    return;
                }
            }
            None => {
                return;
            }
        }
        if let Some(message) = msg.next() {
            for user_cache_type in self.cache_list.iter() {
                if user_cache_type.cache_name() == message {
                    if let Some(key) = msg.next() {
                        if let Err(e) = user_cache_type.clear_from_message(key).await {
                            warn!("user cache clear parse fail:{}", e);
                        }
                    }
                }
            }
        };
    }
    pub async fn listen(&self, client: Client) {
        let con_res = client.get_async_connection().await;
        match con_res {
            Ok(con) => {
                let mut pubsub = con.into_pubsub();
                let res = pubsub.subscribe(REDIS_CHANNEL_NAME).await;
                if let Err(err) = res {
                    error!("listen sub :{}", err);
                    return;
                } else {
                    info!("listen redis clear cache channel:{}", REDIS_CHANNEL_NAME);
                }
                let mut pubsub_stream = pubsub.on_message();
                loop {
                    match pubsub_stream.next().await {
                        Some(msg) => match msg.get_payload() {
                            Ok(pubsub_msg) => {
                                debug!("clear listen msg:{}", pubsub_msg);
                                self.clear_from_message(pubsub_msg).await;
                            }
                            Err(err) => {
                                error!("clear listen parse :{}", err);
                            }
                        },
                        None => {
                            continue;
                        }
                    }
                }
            }
            Err(err) => {
                error!("clear conn redis:{}", err);
            }
        }
    }
}
