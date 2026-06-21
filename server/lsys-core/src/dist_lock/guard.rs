//! Lock guard implementation for automatic release
//!
//! This module provides [`DistLockGuard`], an RAII guard that automatically
//! releases the distributed lock when dropped.

use std::future::Future;
use std::sync::Arc;

use deadpool_redis::Pool;
use tokio::sync::mpsc::Receiver;
use tokio_util::sync::CancellationToken;
use tracing::{Instrument, debug, error, warn};

use super::result::{DistLockError, DistLockLostReason};
use super::watchdog::WatchdogManager;

/// RAII guard for distributed lock
///
/// This guard represents a held lock. When dropped, it automatically
/// releases the lock via a spawned async task.
///
/// ## Monitoring Lock State
///
/// - `is_watchdog_enabled()`: Check if watchdog is active
/// - `is_held()`: Check if lock is still held in Redis
/// - `wait_lock_lost()`: Wait for lock lost notification (blocking)
/// - `try_wait_lock_lost()`: Check for lock lost notification (non-blocking)
/// - `with_lock()`: Execute code with automatic lock state checking
///
/// ## Automatic Release
///
/// When the guard is dropped:
///
/// 1. Cancels the watchdog via `CancellationToken`
/// 2. Unregisters from `WatchdogManager` (if watchdog was enabled)
/// 3. Spawns async task to delete lock key in Redis
///
/// # Example
///
/// ```rust,no_run
/// use lsys_core::dist_lock::{DistLock, DistLockConfig, DistLockLostReason};
/// use std::sync::Arc;
///
/// async fn example(redis_pool: deadpool_redis::Pool) {
///     let config = Arc::new(DistLockConfig::builder(redis_pool).build());
///     let locker = DistLock::new(config);
///
///     let mut guard = locker.try_lock("resource:123", std::time::Duration::from_secs(30)).await.unwrap();
///
///     // Monitor lock state
///     if let Some(reason) = guard.try_wait_lock_lost() {
///         println!("Lock lost: {:?}", reason);
///         return;
///     }
///
///     // Do work safely
///     // ...
///
///     // Lock released when guard is dropped
/// }
/// ```
pub struct DistLockGuard {
    /// Full key (with prefix, used in Redis)
    full_key: String,
    /// Client ID (unique identifier for this lock holder)
    client_id: String,
    /// Redis connection pool
    redis: Pool,
    /// Watchdog state: `Some(...)` when watchdog is enabled, `None` otherwise.
    ///
    /// Bundles everything needed for watchdog lifecycle:
    /// - `Arc<WatchdogManager>`: to call `unregister()` on drop
    /// - `CancellationToken`: cancelled on drop to stop renewal task
    /// - `Receiver`: receives lock-loss notifications from watchdog
    watchdog: Option<(
        Arc<WatchdogManager>,
        CancellationToken,
        Receiver<DistLockLostReason>,
    )>,
    /// Whether the lock has already been explicitly released via `release()`.
    ///
    /// When `true`, `Drop` skips the Redis DEL to avoid a double-release.
    released: bool,
}

impl DistLockGuard {
    /// Create a new guard
    ///
    /// This is called internally by `DistLock` when a lock is acquired.
    pub(crate) fn new(
        full_key: String,
        client_id: String,
        redis: Pool,
        watchdog: Option<(
            Arc<WatchdogManager>,
            CancellationToken,
            Receiver<DistLockLostReason>,
        )>,
    ) -> Self {
        Self {
            full_key,
            client_id,
            redis,
            watchdog,
            released: false,
        }
    }

    /// Explicitly release the lock and await the result
    ///
    /// Preferred over relying on `Drop` when you need to:
    /// - Know whether the release succeeded
    /// - Release the lock before the guard goes out of scope
    ///
    /// After calling this, `Drop` will not attempt a second Redis DEL.
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Lock released (or we were no longer the owner)
    /// - `Err(...)`: Redis connection / protocol error
    pub async fn release(mut self) -> Result<(), DistLockError> {
        // Stop watchdog first
        if let Some((manager, cancel_token, _)) = &self.watchdog {
            cancel_token.cancel();
            manager.unregister(&self.full_key, &self.client_id);
        }
        // Mark released so Drop skips the Redis call
        self.released = true;
        // Perform the actual Redis DEL and surface any error
        Self::do_release_redis(&self.redis, &self.full_key, &self.client_id).await
    }

    /// Core Redis release logic shared by `release()` and `Drop`
    async fn do_release_redis(
        redis: &Pool,
        full_key: &str,
        client_id: &str,
    ) -> Result<(), DistLockError> {
        let mut conn = redis.get().await?;
        let script = r#"
            if redis.call("GET", KEYS[1]) == ARGV[1] then
                return redis.call("DEL", KEYS[1])
            else
                return 0
            end
        "#;
        let result: i32 = redis::Script::new(script)
            .key(full_key)
            .arg(client_id)
            .invoke_async(&mut *conn)
            .await?;
        if result == 1 {
            debug!(key = %full_key, client_id = %client_id, "lock released successfully");
        } else {
            warn!(key = %full_key, client_id = %client_id, "lock release: not the owner or already expired");
        }
        Ok(())
    }

    /// Check if watchdog is enabled for this guard
    ///
    /// # Returns
    ///
    /// - `true`: Watchdog is auto-renewing the lock
    /// - `false`: Lock will expire after TTL
    pub fn is_watchdog(&self) -> bool {
        self.watchdog.is_some()
    }

    /// Check if lock is still held in Redis
    ///
    /// This method uses a Lua script to atomically check if the lock key
    /// exists and has our client ID as the value, minimizing network transfer.
    ///
    /// # Returns
    ///
    /// - `Ok(true)`: We still hold the lock
    /// - `Ok(false)`: Lock is gone or held by another client
    /// - `Err(...)`: Redis connection error
    pub async fn is_held(&self) -> Result<bool, DistLockError> {
        let mut conn = self.redis.get().await?;

        // Lua script: check ownership in Redis, only return 0 or 1
        let script = r#"
            if redis.call("GET", KEYS[1]) == ARGV[1] then
                return 1
            else
                return 0
            end
        "#;

        let result: i32 = redis::Script::new(script)
            .key(&self.full_key)
            .arg(&self.client_id)
            .invoke_async(&mut *conn)
            .await?;

        Ok(result == 1)
    }

    /// Wait for lock lost notification (blocking)
    ///
    /// This method blocks until the watchdog sends a lock lost notification.
    /// Use this when you need to react to lock loss immediately.
    ///
    /// **Note**: Only useful when watchdog is enabled.
    ///
    /// Without watchdog this future **never resolves**, which makes it safe
    /// to use inside `tokio::select!` alongside real work — the lock-lost
    /// branch simply never fires:
    ///
    /// ```rust,no_run
    /// # async fn example(mut guard: lsys_core::dist_lock::DistLockGuard) {
    /// tokio::select! {
    ///     reason = guard.wait_lock_lost() => {
    ///         // Lock lost — abort work immediately
    ///         eprintln!("lock lost: {:?}", reason);
    ///     }
    ///     result = some_long_running_task() => {
    ///         // Normal completion
    ///         let _ = result;
    ///     }
    /// }
    /// # }
    /// # async fn some_long_running_task() -> i32 { 42 }
    /// ```
    ///
    /// # Returns
    ///
    /// - `Some(reason)`: Lock was lost, reason explains why
    /// - `None`: Channel closed unexpectedly (watchdog task panicked)
    ///
    /// Without watchdog: this future never resolves (pending forever).
    pub async fn wait_lock_lost(&mut self) -> Option<DistLockLostReason> {
        match self.watchdog {
            Some((_, _, ref mut rx)) => rx.recv().await,
            // No watchdog: stay pending so select! never picks this branch.
            // Returning None immediately would cause select! to treat it as
            // "lock lost" and incorrectly abort the business logic.
            None => std::future::pending().await,
        }
    }

    /// Try to get lock lost notification (non-blocking)
    ///
    /// Returns immediately without touching Redis. Suitable as a cheap
    /// checkpoint inside a processing loop:
    ///
    /// ```rust,no_run
    /// # fn process_item(_: u32) {}
    /// # async fn example(mut guard: lsys_core::dist_lock::DistLockGuard) {
    /// for item in 0..1000_u32 {
    ///     if let Some(reason) = guard.try_wait_lock_lost() {
    ///         eprintln!("lock lost: {:?}", reason);
    ///         break;
    ///     }
    ///     process_item(item);
    /// }
    /// # }
    /// ```
    ///
    /// **Note**: Only detects loss that the watchdog has already reported.
    /// There is a gap of up to `renew_interval` between the lock expiring and
    /// the watchdog noticing. For a hard guarantee before committing a side
    /// effect, follow up with [`is_held()`](Self::is_held).
    ///
    /// # Returns
    ///
    /// - `Some(reason)`: Lock was lost (watchdog confirmed)
    /// - `None`: No pending notification — either no watchdog, or watchdog
    ///   has not detected a loss yet
    pub fn try_wait_lock_lost(&mut self) -> Option<DistLockLostReason> {
        self.watchdog
            .as_mut()
            .and_then(|(_, _, rx)| rx.try_recv().ok())
    }

    /// Execute code with automatic lock state checking and interruption
    ///
    /// Uses `tokio::select!` to race the provided future against the watchdog's
    /// lock-lost notification. If the lock is lost **while** `f` is running,
    /// the future is **cancelled immediately** rather than left to run to completion.
    ///
    /// ## Execution flow
    ///
    /// 1. **Pre-check** (`try_wait_lock_lost`): reject immediately if lock already lost
    /// 2. **Race** (`select!`): run `f()` and `wait_lock_lost()` concurrently;
    ///    if lock is lost mid-flight, `f` is cancelled (dropped) at its next await point
    ///
    /// Without watchdog, `wait_lock_lost` never resolves (pending forever), so
    /// `select!` always picks the `f` branch — behaviour is identical to a plain `.await`.
    ///
    /// ## Cancellation safety
    ///
    /// If the lock-lost branch wins, the `f` future is **dropped** (cancelled).
    /// Ensure `f` is cancellation-safe, or perform cleanup in a `Drop` guard
    /// inside `f` before using this method.
    ///
    /// # Returns
    ///
    /// - `Ok(result)`: `f` ran to completion, lock held throughout
    /// - `Err(LockLost)`: lock lost before or during execution; `f` was either
    ///   never started (pre-check) or cancelled mid-flight (select branch)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use lsys_core::dist_lock::{DistLock, DistLockConfig};
    /// use std::sync::Arc;
    ///
    /// async fn example(redis_pool: deadpool_redis::Pool) -> Result<(), lsys_core::dist_lock::DistLockError> {
    ///     let config = Arc::new(DistLockConfig::builder(redis_pool).build());
    ///     let locker = DistLock::new(config);
    ///
    ///     let mut guard = locker.try_lock_with_watchdog("resource:123", std::time::Duration::from_secs(30)).await?;
    ///
    ///     let value = guard.with_lock(|| async {
    ///         // If lock is lost here, this future is cancelled immediately
    ///         heavy_work().await
    ///     }).await?;
    ///
    ///     Ok(())
    /// }
    /// # async fn heavy_work() -> i32 { 42 }
    /// ```
    pub async fn with_lock<T, F, Fut>(&mut self, f: F) -> Result<T, DistLockError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = T>,
    {
        // Step 1: Pre-check — consume any already-pending notification before
        // entering select!. Without this, if the channel already has a message
        // AND f() is immediately ready, select! would pick randomly (50 % chance
        // of ignoring the lost lock). try_recv drains the message first so the
        // lock-lost path is taken deterministically.
        if let Some(reason) = self.try_wait_lock_lost() {
            return Err(DistLockError::LockLost {
                key: self.full_key.clone(),
                reason,
            });
        }

        // Step 2: Race f() against lock-lost notification.
        // Borrow note: wait_lock_lost() holds &mut self for the lifetime of the
        // select! block. Extract the key clone *before* entering select! so the
        // lock-lost arm can use it without a second borrow of self.
        let key = self.full_key.clone();
        tokio::select! {
            reason = self.wait_lock_lost() => {
                // Channel closed (None) means watchdog died unexpectedly;
                // treat conservatively as RenewFailed.
                Err(DistLockError::LockLost {
                    key,
                    reason: reason.unwrap_or(DistLockLostReason::RenewFailed),
                })
            }
            result = f() => Ok(result),
        }
    }
}

/// Automatic lock release on drop
///
/// When `DistLockGuard` is dropped, it:
///
/// 1. Cancels the watchdog via `CancellationToken`
/// 2. Unregisters from `WatchdogManager` (if watchdog was enabled)
/// 3. Spawns an async task to delete the lock key in Redis
///
/// ## Async Drop Handling
///
/// Since Rust doesn't support async drop, we spawn a task using
/// `tokio::runtime::Handle::try_current()`. If no runtime is available,
/// we log a warning (lock will expire naturally via TTL).
///
/// ## Lua Script for Release
///
/// ```lua
/// if redis.call("GET", KEYS[1]) == ARGV[1] then
///     return redis.call("DEL", KEYS[1])
/// else
///     return 0
/// end
/// ```
///
/// This ensures only the owner can release the lock.
impl Drop for DistLockGuard {
    fn drop(&mut self) {
        // Step 1: Skip everything if already released via release()
        if self.released {
            return;
        }
        // Step 2: Cancel watchdog and unregister
        if let Some((manager, cancel_token, _)) = &self.watchdog {
            cancel_token.cancel();
            manager.unregister(&self.full_key, &self.client_id);
        }
        // Step 3: Spawn async task to release lock in Redis
        let full_key = self.full_key.clone();
        let client_id = self.client_id.clone();
        let redis = self.redis.clone();
        if let Ok(rt) = tokio::runtime::Handle::try_current() {
            let span_key = full_key.clone();
            let span_client_id = client_id.clone();
            rt.spawn(
                async move {
                    if let Err(e) = Self::do_release_redis(&redis, &full_key, &client_id).await {
                        error!(key = %full_key, error = ?e, "failed to release lock in Drop");
                    }
                }
                .instrument(tracing::info_span!(
                    "background_task",
                    task = "dist-lock-release",
                    key = %span_key,
                    task_id = %span_client_id
                )),
            );
        } else {
            warn!(key = %full_key, "cannot release lock in Drop: no active tokio runtime");
        }
    }
}
