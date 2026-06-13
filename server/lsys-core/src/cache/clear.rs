//基于REDIS
// 多节点缓存同步清理
//当一个节点发送清理命令,所有节点完成缓存删除
use async_trait::async_trait;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::warn;

use crate::remote_notify::{MsgSendBody, RemoteTask};

use super::REMOTE_NOTIFY_TYPE_CACHE;

#[derive(Serialize, Deserialize, Clone)]
pub struct LocalCacheMessage {
    pub cache_name: String,
    pub message: String,
    /// 为 true 时表示清空该 cache 的全部条目，接收方应忽略 message 字段
    #[serde(default)]
    pub clear_all: bool,
}
impl LocalCacheMessage {
    pub fn new(cache_name: &str, message: &str) -> Self {
        Self {
            cache_name: cache_name.to_string(),
            message: message.to_string(),
            clear_all: false,
        }
    }
    pub fn new_clear_all(cache_name: &str) -> Self {
        Self {
            cache_name: cache_name.to_string(),
            message: String::new(),
            clear_all: true,
        }
    }
}

#[async_trait]
pub trait LocalCacheClearItem<'t>: Sync + Send + 't {
    fn cache_name(&self) -> &str;
    async fn clear_from_message(&self, msg: &str, clear_all: bool) -> Result<(), String>;
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
                if let Err(e) = user_cache_type.clear_from_message(&cache_msg.message, cache_msg.clear_all).await {
                    warn!("user cache clear parse fail:{}", e);
                }
                return Ok(None);
            }
        }
        warn!("not find clear cache name:{}", cache_msg.cache_name);
        Ok(None)
    }
}
