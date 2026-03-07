//! `core.Cache` – In-memory key-value cache backed by moka.
//!
//! Exposed as `runtime.core.Cache` in JavaScript with `get` / `set` methods.

use std::sync::Arc;
use std::time::{Duration, Instant};

use moka::sync::Cache;
use rquickjs::{class::Trace, JsLifetime};

#[derive(Clone)]
struct CacheEntry {
    value: String,
    expires_at: Option<Instant>,
}

impl CacheEntry {
    fn is_expired(&self) -> bool {
        self.expires_at
            .map(|deadline| Instant::now() >= deadline)
            .unwrap_or(false)
    }
}

/// Shared cache handle used across runtime components.
#[derive(Clone)]
pub struct SharedCache {
    inner: Arc<Cache<String, CacheEntry>>,
}

impl SharedCache {
    pub fn get(&self, key: &str) -> Option<String> {
        if let Some(entry) = self.inner.get(key) {
            if entry.is_expired() {
                self.inner.remove(key);
                return None;
            }
            return Some(entry.value);
        }
        None
    }

    pub fn set(&self, key: String, value: String, ttl_ms: f64) {
        let ttl = if ttl_ms.is_finite() && ttl_ms > 0.0 {
            Some(Duration::from_secs_f64(ttl_ms / 1000.0))
        } else {
            None
        };

        let entry = CacheEntry {
            value,
            expires_at: ttl.and_then(|d| Instant::now().checked_add(d)),
        };

        self.inner.insert(key, entry);
    }

    pub fn remove(&self, key: &str) {
        self.inner.remove(key);
    }

    pub fn has(&self, key: &str) -> bool {
        if let Some(entry) = self.inner.get(key) {
            if entry.is_expired() {
                self.inner.remove(key);
                return false;
            }
            return true;
        }
        false
    }

    /// Scan and remove expired entries.
    ///
    /// Returns the number of removed keys.
    pub fn cleanup_expired(&self) -> usize {
        let mut expired_keys = Vec::new();
        for (key, entry) in self.inner.iter() {
            if entry.is_expired() {
                expired_keys.push((*key).clone());
            }
        }

        let removed = expired_keys.len();
        for key in expired_keys {
            self.inner.remove(&key);
        }

        removed
    }
}

/// Create a new shared moka cache with the given capacity and default TTL.
pub fn new_shared_cache(capacity: u64, default_ttl: Duration) -> SharedCache {
    SharedCache {
        inner: Arc::new(
            Cache::builder()
                .max_capacity(capacity)
                .time_to_live(default_ttl)
                .build(),
        ),
    }
}

/// JS-visible `Cache` class wrapping the moka cache.
///
/// When a `key_prefix` is set (via runtime namespace), all key operations
/// are transparently prefixed with `"{prefix}:"` so that different runtimes
/// sharing the same underlying cache see isolated key-spaces.
#[derive(Trace, JsLifetime)]
#[rquickjs::class(rename = "Cache")]
pub struct JsCache {
    #[qjs(skip_trace)]
    inner: SharedCache,
    #[qjs(skip_trace)]
    key_prefix: Option<String>,
}

impl JsCache {
    pub fn new(cache: SharedCache, namespace: Option<String>) -> Self {
        Self {
            inner: cache,
            key_prefix: namespace,
        }
    }

    /// Apply the namespace prefix to a key (if configured).
    fn prefixed_key(&self, key: &str) -> String {
        match &self.key_prefix {
            Some(prefix) => format!("{}:{}", prefix, key),
            None => key.to_string(),
        }
    }
}

#[rquickjs::methods]
impl JsCache {
    /// `Cache.get(key)` – returns the cached value or `undefined`.
    #[qjs(rename = "get")]
    pub fn js_get(&self, key: String) -> Option<String> {
        self.inner.get(&self.prefixed_key(&key))
    }

    /// `Cache.set(key, value, ttlMs)` – store a value with per-entry TTL (ms).
    #[qjs(rename = "set")]
    pub fn js_set(&self, key: String, value: String, ttl_ms: f64) {
        self.inner.set(self.prefixed_key(&key), value, ttl_ms);
    }

    /// `Cache.remove(key)` – remove a cached entry.
    #[qjs(rename = "remove")]
    pub fn js_remove(&self, key: String) {
        self.inner.remove(&self.prefixed_key(&key));
    }

    /// `Cache.has(key)` – check if key exists.
    #[qjs(rename = "has")]
    pub fn js_has(&self, key: String) -> bool {
        self.inner.has(&self.prefixed_key(&key))
    }
}
