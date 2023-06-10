mod clear;
#[macro_use]
mod macros;
use hashlink::LruCache;

use std::{
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tokio::sync::Mutex;
use tracing::{debug, warn};

pub use clear::*;

use crate::{now_time, LocalExecType, RemoteNotify};

pub const REMOTE_NOTIFY_TYPE_CACHE: u8 = 101;

#[derive(Clone, Debug)]
pub struct CacheData<T: Clone> {
    time_out: u64,
    data: T,
}

#[derive(Clone, Debug, Copy)]
pub struct LocalCacheConfig {
    /// 缓存前缀
    pub cache_name: &'static str,
    /// 缓存时间,设置为0不开启缓存
    pub cache_time: u64,
    /// 缓存总数量
    pub cache_size: usize,
    /// 缓存刷新时间
    pub refresh_time: u64,
}

impl LocalCacheConfig {
    pub fn new(cache_name: &'static str) -> Self {
        Self {
            cache_name,
            cache_time: 120,
            cache_size: 1024,
            refresh_time: 115,
        }
    }
}

/// 本地数据缓存
pub struct LocalCache<K, T>
where
    K: ToString + std::cmp::Eq + std::hash::Hash + FromStr,
    T: Clone,
{
    cache_config: LocalCacheConfig,
    cache_data: Mutex<LruCache<K, CacheData<T>>>,
    refresh_lock: AtomicBool,
    remote_notify: Arc<RemoteNotify>,
}

impl<K, T> LocalCache<K, T>
where
    K: ToString + std::cmp::Eq + std::hash::Hash + FromStr,
    T: Clone,
{
    pub fn new(remote_notify: Arc<RemoteNotify>, mut cache_config: LocalCacheConfig) -> Self {
        if cache_config.cache_size == 0 && cache_config.cache_time > 0 {
            cache_config.cache_size = 1;
        }
        if cache_config.refresh_time > cache_config.cache_time {
            cache_config.refresh_time = cache_config.cache_time;
        }
        LocalCache {
            remote_notify,
            cache_config,
            refresh_lock: AtomicBool::new(false),
            cache_data: Mutex::new(LruCache::new(cache_config.cache_size)),
        }
    }
    pub fn config(&self) -> &LocalCacheConfig {
        &self.cache_config
    }
    pub async fn get(&self, key: &K) -> Option<T> {
        if self.cache_config.cache_time == 0 {
            return None;
        }
        let now_time = now_time().unwrap_or_default();
        let mut lc = self.cache_data.lock().await;
        if let Some(ua) = lc.get(key) {
            debug!("cache get timeout:{} now_time:{}", ua.time_out, now_time);
            if ua.time_out > now_time {
                if self.cache_config.refresh_time > 0
                    && ua.time_out > now_time + self.cache_config.refresh_time
                    && self
                        .refresh_lock
                        .compare_exchange_weak(false, true, Ordering::Relaxed, Ordering::Relaxed)
                        .unwrap_or(false)
                {
                    return None;
                }
                return Some(ua.data.to_owned());
            }

            lc.remove(key);
        }
        None
    }
    pub async fn set(&self, key: K, data: T, mut set_time: u64) {
        if self.cache_config.cache_time == 0 {
            return;
        }
        let now_time = now_time().unwrap_or_default();
        let cache_time = self.cache_config.cache_time;
        if set_time > cache_time || set_time == 0 {
            set_time = cache_time;
        }
        self.cache_data.lock().await.insert(
            key,
            CacheData {
                time_out: now_time + set_time,
                data,
            },
        );
        if self.cache_config.refresh_time > 0 {
            self.refresh_lock.store(false, Ordering::Relaxed);
        }
    }
    pub async fn del(&self, key: &K) {
        if self.cache_config.cache_time == 0 {
            return;
        }
        self.cache_data.lock().await.remove(key);
    }
    pub async fn clear(&self, key: &K) {
        self.del(key).await;
        let send_msg =
            LocalCacheMessage::new(self.cache_config.cache_name.to_string(), key.to_string());
        if let Err(err) = self
            .remote_notify
            .call(
                REMOTE_NOTIFY_TYPE_CACHE,
                send_msg,
                None,
                LocalExecType::RemoteExec,
                None,
            )
            .await
        {
            warn!("notify clear cache error:{}", err);
        }
    }
}
