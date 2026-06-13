//! Distributed lock implementation using Redis
//!
//! This module provides the main lock types for distributed locking:
//!
//! - [`DistLock`]: The lock manager for acquiring and managing a batch of locks
//! - [`DistLockGuard`]: RAII guard that automatically releases the lock when dropped
//!
//! # Design
//!
//! One `DistLock` instance is created per configuration (Redis pool + prefix).
//! It manages **multiple lock keys** sharing a single `WatchdogManager` background task.
//!
//! ```text
//! DistLock (one per config)  →  WatchdogManager (one background task)
//!   try_lock("order:1", ttl)  →  HashMap entry: "order:1"
//!   try_lock("order:2", ttl)  →  HashMap entry: "order:2"
//!   try_lock("order:3", ttl)  →  HashMap entry: "order:3"
//! ```
//!
//! # Lock Acquisition Flow
//!
//! ```text
//! ┌─────────────┐
//! │ DistLock    │
//! │ ::new()     │
//! └──────┬──────┘
//!        │
//!        ▼
//! ┌─────────────────┐     ┌──────────────┐
//! │ try_lock(key,…) │────▶│ Redis SET NX │
//! └──────┬──────────┘     │    EX        │
//!        │                └──────┬───────┘
//!        │                   │
//!        │    ┌──────────────┴──────────────┐
//!        │    │                             │
//!        │    ▼                             ▼
//!        │  Success                      Failed
//!        │    │                             │
//!        │    ▼                             ▼
//!        │  ┌─────────────┐         ┌─────────────┐
//!        │  │ Start       │         │ Retry or    │
//!        │  │ Watchdog    │         │ Return Err  │
//!        │  └──────┬──────┘         └─────────────┘
//!        │         │
//!        │         ▼
//!        │  ┌─────────────┐
//!        │  │ Return      │
//!        └─▶│ Guard       │
//!           └─────────────┘
//! ```
//!
//! # Lock Release Flow
//!
//! The lock is released in two scenarios:
//!
//! 1. **Explicit drop**: When `DistLockGuard` is dropped, it:
//!    - Cancels the watchdog via `CancellationToken`
//!    - Unregisters from `WatchdogManager`
//!    - Executes Lua script to delete the lock key (if still owner)
//!
//! 2. **Lock lost**: When watchdog fails to renew, it:
//!    - Sends `DistLockLostReason` via channel
//!    - User can detect via `wait_lock_lost()` or `try_wait_lock_lost()`

use std::sync::Arc;
use std::time::Duration;

use tokio::time::sleep;
use tracing::{debug, info};

use super::config::DistLockConfig;
use super::guard::DistLockGuard;
use super::result::DistLockError;
use super::watchdog::WatchdogManager;

/// Retry strategy for lock acquisition
///
/// Used with [`DistLock::lock`] / [`DistLock::lock_with_watchdog`] to control
/// how long to keep retrying and the interval between attempts.
///
/// For a **single attempt with no retry**, use [`DistLock::try_lock`] /
/// [`DistLock::try_lock_with_watchdog`] instead.
///
/// ## Example
///
/// ```rust,ignore
/// // Wait up to 5 seconds, retry every 100ms
/// let strategy = RetryStrategy {
///     timeout: Duration::from_secs(5),
///     retry_interval: Duration::from_millis(100),
/// };
///
/// // Retry "forever" (until process restarts or lock is acquired)
/// let strategy = RetryStrategy {
///     timeout: Duration::MAX,
///     retry_interval: Duration::from_millis(100),
/// };
/// ```
#[derive(Debug, Clone)]
pub struct RetryStrategy {
    /// Total timeout for lock acquisition
    ///
    /// Retries will continue until this duration has elapsed, after which
    /// [`DistLockError::AcquireTimeout`] is returned.
    /// Use `Duration::MAX` for effectively infinite retries.
    pub timeout: Duration,
    /// Interval between retry attempts
    ///
    /// After each failed `try_lock()`, wait this duration before next attempt.
    /// Recommended: 50-500ms to balance responsiveness and Redis load.
    pub retry_interval: Duration,
}

impl Default for RetryStrategy {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            retry_interval: Duration::from_millis(100),
        }
    }
}

/// Watchdog configuration for automatic lock renewal
///
/// The watchdog is a background task that automatically extends the lock's TTL
/// to prevent it from expiring while the holder is still working.
///
/// ## How it works
///
/// 1. When lock is acquired, watchdog calculates `renew_interval = max(ttl * ratio, 1 s)`
///    (ratio clamped to `0.1..=0.8`); a random ≤10 % jitter is added to spread
///    simultaneous registrations across time
/// 2. Background task sleeps until the nearest `next_renew_time`, woken early when entries change
/// 3. All due locks are renewed in **one Redis pipeline** (1 RTT regardless of lock count)
/// 4. Renewal stops when: `max_duration` exceeded or lock lost
///
/// ## Example
///
/// ```rust,ignore
/// // Enable watchdog with 30% renewal ratio, max 5 minutes total
/// let config = WatchdogConfig {
///     enabled: true,
///     renew_interval_ratio: 0.3,  // Renew at 30% of TTL
///     max_duration: Some(Duration::from_secs(300)),  // Max 5 minutes
/// };
///
/// // Disable watchdog (lock will expire after TTL)
/// let config = WatchdogConfig {
///     enabled: false,
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone)]
pub struct WatchdogConfig {
    /// Renewal interval as ratio of TTL (e.g., 0.3 = 30% of TTL)
    ///
    /// If TTL is 10 seconds and ratio is 0.3, renewal happens every 3 seconds.
    /// This ensures lock is renewed well before expiration.
    ///
    /// **Clamped to `0.1..=0.8`** at registration time:
    /// - Below 0.1 would cause excessively frequent Redis calls
    /// - Above 0.8 leaves too little safety margin before expiry
    ///
    /// **Minimum interval: 1 s** — a Redis EVAL round-trip can be 50–200 ms
    /// across datacenters; anything below 1 s risks renewal calls overlapping
    /// with the next scheduled cycle.
    ///
    /// All due locks are renewed in a **single pipeline** per cycle, so Redis
    /// load is O(1) connections/RTT regardless of how many locks are held.
    pub renew_interval_ratio: f32,
    /// Maximum total duration for lock holding (None = unlimited)
    ///
    /// After this duration, watchdog stops renewing and lock expires.
    /// Useful for preventing indefinite lock holding in case of bugs.
    pub max_duration: Option<Duration>,
}

impl Default for WatchdogConfig {
    fn default() -> Self {
        Self {
            renew_interval_ratio: 0.3,
            max_duration: None,
        }
    }
}

/// Distributed lock client
///
/// This is the main type for acquiring distributed locks. Each instance
/// represents a lock on a specific key.
///
/// # Example
///
/// ```rust,no_run
/// use lsys_core::dist_lock::{DistLock, DistLockConfig, RetryStrategy};
/// use std::sync::Arc;
/// use std::time::Duration;
///
/// async fn example(redis_pool: deadpool_redis::Pool) {
///     // Create once, reuse for many keys
///     let locker = Arc::new(DistLock::new(Arc::new(
///         DistLockConfig::builder(redis_pool)
///             .key_prefix("myapp:")
///             .build()
///     )));
///
///     let ttl = Duration::from_secs(30);
///
///     // Acquire different keys through the same locker
///     match locker.try_lock("order:123", ttl).await {
///         Ok(guard) => println!("Lock held for order:123"),
///         Err(e) => eprintln!("Failed: {:?}", e),
///     }
///
///     // Acquire with retry strategy
///     let retry = RetryStrategy {
///         timeout: Some(Duration::from_secs(5)),
///         retry_interval: Duration::from_millis(100),
///     };
///     match locker.lock("order:456", ttl, retry).await {
///         Ok(guard) => println!("Lock acquired with retry!"),
///         Err(e) => eprintln!("Failed: {:?}", e),
///     }
///
///     // Acquire with watchdog (auto-renewal)
///     match locker.try_lock_with_watchdog("order:789", ttl).await {
///         Ok(guard) => println!("Lock held with watchdog!"),
///         Err(e) => eprintln!("Failed: {:?}", e),
///     }
/// }
/// ```
pub struct DistLock {
    /// Configuration (Redis pool, watchdog config)
    config: Arc<DistLockConfig>,
    /// Unique client identifier (hostname:uuid)
    ///
    /// Generated once at construction, shared across all lock acquisitions.
    client_id: String,
    /// Watchdog manager shared across all locks acquired via this instance
    watchdog_manager: Arc<WatchdogManager>,
}

impl DistLock {
    /// Create a new distributed lock manager
    ///
    /// One instance should be created per configuration and **reused** for
    /// acquiring multiple lock keys. All locks acquired through the same
    /// `DistLock` share one `WatchdogManager` background task.
    ///
    /// # Parameters
    ///
    /// - `config`: Lock configuration (Redis pool, key prefix, watchdog config)
    ///
    /// # Client ID Generation
    ///
    /// The client ID is generated once at construction:
    /// - Hostname: For machine identification
    /// - UUID: For global uniqueness
    ///
    /// Format: `{hostname}:{uuid}`
    pub fn new(config: Arc<DistLockConfig>) -> Self {
        let client_id = Self::generate_client_id();
        let watchdog_manager = Arc::new(WatchdogManager::new(config.redis.clone()));

        Self {
            config,
            client_id,
            watchdog_manager,
        }
    }

    /// Build the full Redis key by prepending the configured prefix
    fn make_full_key(&self, key: &str) -> String {
        if self.config.key_prefix.is_empty() {
            key.to_string()
        } else {
            format!("{}{}", self.config.key_prefix, key)
        }
    }

    /// Generate a unique client ID
    ///
    /// Format: `{hostname}:{uuid}`
    ///
    /// This ensures uniqueness across:
    /// - Different machines (hostname)
    /// - Different lock instances (uuid)
    fn generate_client_id() -> String {
        let hostname = hostname::get()
            .ok()
            .and_then(|h| {
                let s = h.to_string_lossy().to_string();
                if s.is_empty() { None } else { Some(s) }
            })
            .unwrap_or_else(|| "unknown".to_string());
        let uuid = uuid::Uuid::new_v4();
        format!("{}:{}", hostname, uuid)
    }

    /// Acquire lock on `key` with custom TTL (single attempt, no retry)
    ///
    /// **Watchdog is NOT enabled** - lock will expire after TTL.
    ///
    /// # Parameters
    ///
    /// - `key`: Lock key (prefix from config is prepended automatically)
    /// - `ttl`: Lock expiration time
    ///
    /// # Returns
    ///
    /// - `Ok(guard)`: Lock acquired
    /// - `Err(AcquireFailed)`: Lock is held by another client
    pub async fn try_lock(
        &self,
        key: impl AsRef<str>,
        ttl: Duration,
    ) -> Result<DistLockGuard, DistLockError> {
        self.try_lock_internal(&self.make_full_key(key.as_ref()), ttl, None)
            .await
    }

    /// Acquire lock on `key` with custom TTL and watchdog (single attempt, no retry)
    ///
    /// Watchdog will automatically renew the lock according to `watchdog_config`.
    /// Use [`WatchdogConfig::default()`] for sensible defaults (ratio=0.3, no max_duration).
    ///
    /// # Parameters
    ///
    /// - `key`: Lock key (prefix from config is prepended automatically)
    /// - `ttl`: Lock expiration time
    /// - `watchdog_config`: Per-call watchdog settings (renewal ratio, max duration)
    ///
    /// # Returns
    ///
    /// - `Ok(guard)`: Lock acquired, watchdog started
    /// - `Err(AcquireFailed)`: Lock is held by another client
    pub async fn try_lock_with_watchdog(
        &self,
        key: impl AsRef<str>,
        ttl: Duration,
        watchdog_config: WatchdogConfig,
    ) -> Result<DistLockGuard, DistLockError> {
        self.try_lock_internal(
            &self.make_full_key(key.as_ref()),
            ttl,
            Some(watchdog_config),
        )
        .await
    }

    /// Acquire lock on `key` with custom TTL and retry strategy
    ///
    /// **Watchdog is NOT enabled** - lock will expire after TTL.
    ///
    /// # Parameters
    ///
    /// - `key`: Lock key (prefix from config is prepended automatically)
    /// - `ttl`: Lock expiration time
    /// - `retry`: Retry strategy (timeout and interval)
    ///
    /// # Returns
    ///
    /// - `Ok(guard)`: Lock acquired
    /// - `Err(AcquireTimeout)`: Timeout exceeded
    pub async fn lock(
        &self,
        key: impl AsRef<str>,
        ttl: Duration,
        retry: RetryStrategy,
    ) -> Result<DistLockGuard, DistLockError> {
        let full_key = self.make_full_key(key.as_ref());
        self.lock_internal(&full_key, ttl, retry, None).await
    }

    /// Acquire lock with custom TTL, retry strategy and watchdog
    ///
    /// Watchdog will automatically renew the lock.
    ///
    /// # Parameters
    ///
    /// - `ttl`: Lock expiration time
    /// - `retry`: Retry strategy (timeout and interval)
    ///
    /// # Returns
    ///
    /// - `Ok(guard)`: Lock acquired, watchdog started
    /// - `Err(AcquireTimeout)`: Timeout exceeded
    ///
    /// Acquire lock with custom TTL, retry strategy and watchdog.
    /// Watchdog will automatically renew the lock according to `watchdog_config`.
    /// Use [`WatchdogConfig::default()`] for sensible defaults (ratio=0.3, no max_duration).
    ///
    /// # Parameters
    ///
    /// - `key`: Lock key (prefix from config is prepended automatically)
    /// - `ttl`: Lock expiration time
    /// - `retry`: Retry strategy (timeout and interval)
    /// - `watchdog_config`: Per-call watchdog settings (renewal ratio, max duration)
    ///
    /// # Returns
    ///
    /// - `Ok(guard)`: Lock acquired, watchdog started
    /// - `Err(AcquireTimeout)`: Timeout exceeded
    pub async fn lock_with_watchdog(
        &self,
        key: impl AsRef<str>,
        ttl: Duration,
        retry: RetryStrategy,
        watchdog_config: WatchdogConfig,
    ) -> Result<DistLockGuard, DistLockError> {
        let full_key = self.make_full_key(key.as_ref());
        self.lock_internal(&full_key, ttl, retry, Some(watchdog_config))
            .await
    }

    /// Internal implementation for lock with retry
    async fn lock_internal(
        &self,
        full_key: &str,
        ttl: Duration,
        retry: RetryStrategy,
        watchdog_config: Option<WatchdogConfig>,
    ) -> Result<DistLockGuard, DistLockError> {
        let start = std::time::Instant::now();
        loop {
            match self
                .try_lock_internal(full_key, ttl, watchdog_config.clone())
                .await
            {
                Ok(guard) => return Ok(guard),
                Err(DistLockError::AcquireFailed { .. }) => {
                    if start.elapsed() >= retry.timeout {
                        return Err(DistLockError::AcquireTimeout {
                            key: full_key.to_string(),
                            timeout: retry.timeout,
                        });
                    }
                    sleep(retry.retry_interval).await;
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Try to acquire lock with custom TTL (non-blocking)
    ///
    /// This is the core lock acquisition method. It uses a Lua script to
    /// atomically set the lock key with NX (only if not exists) and EX (expiration).
    ///
    /// ## Lua Script
    ///
    /// ```lua
    /// local result = redis.call("SET", KEYS[1], ARGV[1], "NX", "EX", ARGV[2])
    /// if result then
    ///     return 1
    /// else
    ///     return 0
    /// end
    /// ```
    ///
    /// ## Lock Acquisition Steps
    ///
    /// 1. Execute Lua script to atomically set key with TTL
    /// 2. If successful and `enable_watchdog=true`, start watchdog
    /// 3. Register lock with watchdog manager
    /// 4. Return guard
    ///
    /// ## Parameters
    ///
    /// - `ttl`: Lock expiration time
    /// - `enable_watchdog`: Whether to enable auto-renewal
    ///
    /// ## Returns
    ///
    /// - `Ok(guard)`: Lock acquired (watchdog started if enabled)
    /// - `Err(AcquireFailed)`: Lock is held by another client
    /// - `Err(Redis/RedisPool)`: Connection error
    async fn try_lock_internal(
        &self,
        full_key: &str,
        ttl: Duration,
        watchdog_config: Option<WatchdogConfig>,
    ) -> Result<DistLockGuard, DistLockError> {
        let mut conn = self.config.redis.get().await?;

        // Lua script: atomic lock acquisition with TTL
        let script = r#"
            local result = redis.call("SET", KEYS[1], ARGV[1], "NX", "EX", ARGV[2])
            if result then
                return 1
            else
                return 0
            end
        "#;

        let result: i32 = redis::Script::new(script)
            .key(full_key)
            .arg(&self.client_id)
            .arg(ttl.as_secs() as i32)
            .invoke_async(&mut *conn)
            .await?;

        if result == 1 {
            info!(
                key = %full_key,
                client_id = %self.client_id,
                ttl = %ttl.as_secs(),
                watchdog = watchdog_config.is_some(),
                "lock acquired successfully"
            );

            // Build watchdog handle only when config is provided
            let watchdog = if let Some(wc) = watchdog_config {
                let (cancel_token, lock_lost_rx) = self.watchdog_manager.register(
                    full_key.to_string(),
                    self.client_id.clone(),
                    ttl,
                    &wc,
                );
                Some((self.watchdog_manager.clone(), cancel_token, lock_lost_rx))
            } else {
                None
            };

            Ok(DistLockGuard::new(
                full_key.to_string(),
                self.client_id.clone(),
                self.config.redis.clone(),
                watchdog,
            ))
        } else {
            debug!(
                key = %full_key,
                client_id = %self.client_id,
                "lock acquisition failed: already held by another client"
            );
            Err(DistLockError::AcquireFailed {
                key: full_key.to_string(),
            })
        }
    }
}
