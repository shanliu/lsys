//基于REDIS
// 多节点缓存同步清理
//当一个节点发送清理命令,所有节点完成缓存删除
use async_trait::async_trait;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::warn;

use crate::{MsgSendBody, RemoteTask};

use super::REMOTE_NOTIFY_TYPE_CACHE;

#[derive(Serialize, Deserialize, Clone)]
pub struct LocalCacheMessage {
    pub cache_name: String,
    pub message: String,
}
impl LocalCacheMessage {
    pub fn new(cache_name: &str, message: &str) -> Self {
        Self {
            cache_name: cache_name.to_string(),
            message: message.to_string(),
        }
    }
}

#[async_trait]
pub trait LocalCacheClearItem<'t>: Sync + Send + 't {
    fn cache_name(&self) -> &str;
    async fn clear_from_message(&self, msg: &str) -> Result<(), String>;
}

/// 订阅远程通知清理本地缓存
pub struct LocalCacheClear<'t> {
    cache_list: Vec<Box<dyn LocalCacheClearItem<'t>>>,
}
impl<'t> LocalCacheClear<'t> {
    pub fn new(cache_list: Vec<Box<dyn LocalCacheClearItem<'t>>>) -> Self {
        LocalCacheClear { cache_list }
    }
}

#[async_trait]
impl RemoteTask for LocalCacheClear<'_> {
    fn msg_type(&self) -> u8 {
        REMOTE_NOTIFY_TYPE_CACHE
    }
    async fn run(&self, msg: MsgSendBody) -> Result<Option<Value>, String> {
        let cache_msg =
            serde_json::from_value::<LocalCacheMessage>(msg.data).map_err(|e| e.to_string())?;
        for user_cache_type in self.cache_list.iter() {
            if user_cache_type.cache_name() == cache_msg.cache_name {
                if let Err(e) = user_cache_type.clear_from_message(&cache_msg.message).await {
                    warn!("user cache clear parse fail:{}", e);
                }
                return Ok(None);
            }
        }
        warn!("not find clear cache name:{}", cache_msg.cache_name);
        Ok(None)
    }
}
