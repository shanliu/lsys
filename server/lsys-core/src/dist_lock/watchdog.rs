//! Watchdog module for automatic lock renewal
//!
//! The watchdog is a background task that automatically extends lock TTL
//! to prevent locks from expiring while the holder is still working.
//!
//! ## Architecture
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────────┐
//! │                    WatchdogManager                            │
//! │  ┌─────────────────────────────────────────────────────────┐ │
//! │  │  Background Task (dynamic interval per renew_interval)  │ │
//! │  │  ┌─────────────────────────────────────────────────────┐│ │
//! │  │  │  1. Collect locks needing renewal                   ││ │
//! │  │  │  2. Check limits (max_duration)                      ││ │
//! │  │  │  3. Execute Lua script to extend TTL                ││ │
//! │  │  │  4. Update renewal info or notify lock lost         ││ │
//! │  │  └─────────────────────────────────────────────────────┘│ │
//! │  └─────────────────────────────────────────────────────────┘ │
//! │                                                               │
//! │  entries: HashMap<String, WatchdogEntry>                     │
//! │  ┌─────────────────────────────────────────────────────────┐ │
//! │  │  key → WatchdogEntry                                    │ │
//! │  │         - client_id                                     │ │
//! │  │         - ttl, renew_interval                           │ │
//! │  │         - next_renew_time                               │ │
//! │  │         - max_duration                                  │ │
//! │  │         - start_time, renew_count                       │ │
//! │  │         - lock_lost_tx (notification channel)           │ │
//! │  │         - cancel_token                                  │ │
//! │  └─────────────────────────────────────────────────────────┘ │
//! └──────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Renewal Flow
//!
//! 1. When lock is acquired, `register()` adds entry to `entries`
//! 2. Background task sleeps until the nearest `next_renew_time`, then calls `renew_expired_locks()`
//!    (woken early by `Notify` when entries change)
//! 3. For each entry where `next_renew_time <= now`:
//!    - Check if `max_duration` exceeded → notify and remove
//!    - Execute Lua script: `if GET key == client_id then EXPIRE key ttl`
//!    - If success: update `renew_count`, `next_renew_time`
//!    - If fail: notify via `lock_lost_tx`, remove entry
//! 4. When lock is released, `unregister()` removes entry
//!
//! ## Error Handling
//!
//! - Connection errors: retry after 1 second (don't remove entry)
//! - Renewal failed (not owner): notify and remove (lock definitely lost)
//! - Limits exceeded: notify and remove (intentional stop)

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use deadpool_redis::Pool;
use parking_lot::{Mutex, RwLock};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::{Instrument, debug, error, info, warn};

use super::WatchdogConfig;
use super::result::DistLockLostReason;

/// Watchdog manager for automatic lock renewal
///
/// Each `DistLockConfig` has one `WatchdogManager` that manages all locks
/// created with that configuration. The manager runs a background task
/// that periodically checks and renews locks.
///
/// ## Lifecycle
///
/// 1. Created with `DistLockConfig`
/// 2. `start()` spawns background task (called on first lock acquisition)
/// 3. `register()` adds lock entry when lock is acquired
/// 4. Background task renews locks periodically
/// 5. `unregister()` removes entry when lock is released
/// 6. `stop()` cancels background task (called on Drop)
///
/// ## Example
///
/// ```rust,ignore
/// // Created internally by DistLock
/// let manager = WatchdogManager::new(redis_pool, config);
///
/// // Register a lock for renewal
/// let (cancel_token, lock_lost_rx) = manager.register(
///     "my_lock_key",
///     "client_id_123",
///     Duration::from_secs(10),
///     &watchdog_config,
/// );
///
/// // Unregister when lock is released
/// manager.unregister("my_lock_key", "client_id_123");
///
/// // Stop background task
/// manager.stop();
/// ```
pub struct WatchdogManager {
    /// Redis connection pool for renewal operations
    redis: Pool,
    /// Lock registry: key → WatchdogEntry
    ///
    /// Uses `Arc<RwLock>` for safe concurrent access.
    /// Read lock for checking, write lock for updates.
    entries: Arc<RwLock<HashMap<String, WatchdogEntry>>>,
    /// Global cancellation token for the background task
    ///
    /// When cancelled, background task stops.
    cancel_token: CancellationToken,
    /// Handle to the background task
    ///
    /// Used to abort the task on stop.
    task_handle: Mutex<Option<JoinHandle<()>>>,
    /// Wake-up signal sent by `register()` so the background task
    /// immediately recalculates its sleep duration instead of waiting
    /// out the previous interval.
    notify: Arc<tokio::sync::Notify>,
    /// Whether the background task is currently running.
    ///
    /// Set to `false` while holding the entries **read lock** (inside the task)
    /// and checked while holding the entries **write lock** (in `register()`).
    /// This mutual exclusion via the entries lock eliminates the TOCTOU race
    /// between "task decides to exit" and "register() decides not to spawn".
    task_running: Arc<AtomicBool>,
}

/// Entry in the watchdog registry
///
/// Contains all information needed to renew a lock and track its state.
/// Each lock has one entry while it's being held.
struct WatchdogEntry {
    /// Client ID that owns the lock
    ///
    /// Used in Lua script to verify ownership before renewal.
    client_id: String,
    /// Lock TTL (time to live)
    ///
    /// Each renewal extends the lock by this duration.
    ttl: Duration,
    /// Interval between renewals
    ///
    /// Calculated as `ttl * renew_interval_ratio`.
    /// Lock is renewed when `elapsed >= renew_interval`.
    renew_interval: Duration,
    /// Next scheduled renewal time
    ///
    /// Background task checks this to determine if renewal is needed.
    next_renew_time: Instant,
    /// Maximum total holding duration (optional)
    ///
    /// After this time, watchdog stops renewing.
    max_duration: Option<Duration>,
    /// Time when lock was acquired
    ///
    /// Used to check `max_duration` limit.
    start_time: Instant,
    /// Number of successful renewals
    renew_count: usize,
    /// Channel to notify lock holder of loss
    ///
    /// Sends `DistLockLostReason` when lock is lost.
    lock_lost_tx: Sender<DistLockLostReason>,
    /// Individual cancellation token
    ///
    /// When cancelled, this specific lock's renewal stops.
    cancel_token: CancellationToken,
}

/// Lock information for processing (collected before async operations)
///
/// This struct is used to avoid holding `RwLock` across await points.
/// Information is collected synchronously, then processed asynchronously.
struct LockProcessInfo {
    key: String,
    client_id: String,
    ttl: Duration,
    max_duration: Option<Duration>,
    start_time: Instant,
    lock_lost_tx: Sender<DistLockLostReason>,
    cancel_token: CancellationToken,
}

impl WatchdogManager {
    /// Create a new watchdog manager
    ///
    /// The manager is created but not started. Call `start()` to begin
    /// the background renewal task.
    ///
    /// ## Parameters
    ///
    /// - `redis`: Redis connection pool
    /// - `config`: Lock configuration (contains watchdog settings)
    pub fn new(redis: Pool) -> Self {
        Self {
            redis,
            entries: Arc::new(RwLock::new(HashMap::new())),
            cancel_token: CancellationToken::new(),
            task_handle: Mutex::new(None),
            notify: Arc::new(tokio::sync::Notify::new()),
            task_running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Ensure the background task is running.
    ///
    /// Called automatically by `register()`. Spawns a new task only when
    /// none is running (first registration or after the previous task exited).
    ///
    /// The task exits on its own when the entry map becomes empty, so
    /// there is never an idle task burning CPU between lock acquisitions.
    fn ensure_task_running(&self) {
        // NOTE: caller must hold entries write lock.
        // task_running is only set to false while the task holds the entries
        // read lock, and we check it here while holding the write lock —
        // so the two are mutually exclusive, eliminating the TOCTOU race.
        if self.task_running.load(Ordering::Acquire) {
            return;
        }

        let redis = self.redis.clone();
        let entries = self.entries.clone();
        let cancel_token = self.cancel_token.clone();
        let notify = self.notify.clone();
        let task_running = self.task_running.clone();

        self.task_running.store(true, Ordering::Release);

        let task = tokio::spawn(async move {
            info!("watchdog task started");

            loop {
                // Compute sleep until the nearest renewal deadline.
                // IMPORTANT: set task_running=false atomically while holding
                // the entries read lock, so register() (which holds the write
                // lock when it checks task_running) cannot race with us.
                let sleep_duration = {
                    let entries_guard = entries.read();
                    if entries_guard.is_empty() {
                        // Mark ourselves as stopped before releasing the read
                        // lock.  register() holds the write lock when it
                        // inserts + checks task_running, so it will see false
                        // and spawn a new task rather than relying on us.
                        task_running.store(false, Ordering::Release);
                        break;
                    }
                    let now = Instant::now();
                    entries_guard
                        .values()
                        .filter_map(|e| e.next_renew_time.checked_duration_since(now))
                        .min()
                        .unwrap_or(Duration::ZERO) // at least one is already due
                };

                tokio::select! {
                    _ = cancel_token.cancelled() => {
                        task_running.store(false, Ordering::Release);
                        break;
                    }
                    // register() notified us; re-evaluate sleep duration.
                    _ = notify.notified() => {}
                    _ = tokio::time::sleep(sleep_duration) => {
                        let task_id = crate::utils::rand_str(crate::utils::RandType::LowerHex, 8);
                        Self::renew_expired_locks(&redis, &entries)
                            .instrument(tracing::info_span!(
                                "background_task",
                                task = "dist-lock-renew",
                                task_id = task_id
                            ))
                            .await;
                    }
                }
            }

            info!("watchdog task stopped");
        });

        let mut handle = self.task_handle.lock();
        *handle = Some(task);
    }

    /// Stop the watchdog background task
    ///
    /// Cancels the global cancellation token and aborts the task.
    /// Called automatically on Drop.
    pub fn stop(&self) {
        self.cancel_token.cancel();
        let mut handle = self.task_handle.lock();
        if let Some(h) = handle.take() {
            h.abort();
        }
    }

    /// Register a lock for automatic renewal
    ///
    /// Called when a lock is successfully acquired. Adds an entry to the
    /// registry that will be checked by the background task.
    ///
    /// ## Parameters
    ///
    /// - `key`: Full lock key (with prefix)
    /// - `client_id`: Client ID that owns the lock
    /// - `ttl`: Lock TTL
    /// - `watchdog_config`: Watchdog settings for this lock
    ///
    /// ## Returns
    ///
    /// Tuple of:
    /// - `CancellationToken`: Token to cancel this lock's renewal
    /// - `Receiver<DistLockLostReason>`: Channel to receive lock loss notifications
    ///
    /// ## Logging
    ///
    /// ```text
    /// debug!(
    ///     key = %key,
    ///     client_id = %client_id,
    ///     ttl = %ttl.as_secs(),
    ///     renew_interval = %renew_interval.as_secs(),
    ///     "watchdog registered"
    /// );
    /// ```
    pub fn register(
        &self,
        key: String,
        client_id: String,
        ttl: Duration,
        watchdog_config: &WatchdogConfig,
    ) -> (CancellationToken, Receiver<DistLockLostReason>) {
        let cancel_token = CancellationToken::new();
        let (lock_lost_tx, lock_lost_rx) = tokio::sync::mpsc::channel::<DistLockLostReason>(1);

        // Clamp ratio to a safe range:
        //   < 0.1 → too frequent Redis calls (e.g. ratio=0.01, TTL=10s → 100ms interval)
        //   > 0.8 → too close to expiry, clock skew / network jitter can cause the
        //           lock to expire before the next renewal fires
        let ratio = (watchdog_config.renew_interval_ratio as f64).clamp(0.1, 0.8);
        // Minimum 1 s. Rationale:
        //   - A Redis EVAL round-trip can be 50–200 ms across datacenters.
        //   - Anything below 1 s risks the renewal call itself overlapping with
        //     the next scheduled renewal, causing redundant or racing calls.
        //   - Users who need shorter intervals should reconsider their TTL.
        let renew_ms = (ttl.as_millis() as f64 * ratio) as u64;
        let renew_interval = Duration::from_millis(renew_ms.max(1_000));

        let entry = WatchdogEntry {
            client_id: client_id.clone(),
            ttl,
            renew_interval,
            next_renew_time: Instant::now() + renew_interval,
            max_duration: watchdog_config.max_duration,
            start_time: Instant::now(),
            renew_count: 0,
            lock_lost_tx,
            cancel_token: cancel_token.clone(),
        };

        {
            let mut entries = self.entries.write();
            entries.insert(key.clone(), entry);
            // Check task_running while holding the write lock.
            // The task sets task_running=false only while holding the read
            // lock, so these two critical sections are mutually exclusive.
            self.ensure_task_running();
        }
        // Wake the task so it immediately re-evaluates sleep duration.
        self.notify.notify_one();

        debug!(
            key = %key,
            client_id = %client_id,
            ttl_ms = %ttl.as_millis(),
            renew_interval_ms = %renew_interval.as_millis(),
            "watchdog registered"
        );

        (cancel_token, lock_lost_rx)
    }

    /// Unregister a lock from automatic renewal
    ///
    /// Called when a lock is released. Removes the entry from the registry
    /// and cancels its individual cancellation token.
    ///
    /// ## Parameters
    ///
    /// - `key`: Full lock key
    /// - `client_id`: Client ID (for logging)
    ///
    /// ## Logging
    ///
    /// ```text
    /// debug!(
    ///     key = %key,
    ///     client_id = %client_id,
    ///     renew_count = entry.renew_count,
    ///     "watchdog unregistered"
    /// );
    /// ```
    pub fn unregister(&self, key: &str, client_id: &str) {
        let removed = {
            let mut entries = self.entries.write();
            entries.remove(key)
        };
        if let Some(entry) = removed {
            entry.cancel_token.cancel();
            // Wake the task so it can detect an empty map immediately
            // (e.g. last lock released) rather than waiting out its sleep.
            self.notify.notify_one();
            debug!(
                key = %key,
                client_id = %client_id,
                renew_count = entry.renew_count,
                "watchdog unregistered"
            );
        }
    }

    /// Renew locks that need renewal
    ///
    /// This is the main logic of the background task. It:
    /// 1. Collects locks that need renewal (next_renew_time <= now)
    /// 2. Handles max_duration / cancelled entries without touching Redis
    /// 3. Batches all remaining renewals into **one Redis pipeline** (1 RTT)
    /// 4. Updates state or notifies lock loss per result
    ///
    /// ## Why pipeline?
    ///
    /// Sequential per-lock calls multiply Redis RTT by the number of held locks.
    /// Under cross-datacenter latency (50–200 ms/call) and high lock counts this
    /// becomes a bottleneck.  A single pipeline carries N EVAL commands in one
    /// TCP round-trip, keeping renewal overhead constant regardless of scale.
    ///
    /// ## Error Handling
    ///
    /// | Error | Action |
    /// |-------|--------|
    /// | Guard dropped (cancel) | Skip, entry already removed by unregister() |
    /// | Max duration exceeded | Notify, remove (no Redis call) |
    /// | Pool exhausted / connect error | Retry all after 1 s (keep entries) |
    /// | Renewal result == 0 (not owner) | Notify RenewFailed, remove |
    async fn renew_expired_locks(redis: &Pool, entries: &RwLock<HashMap<String, WatchdogEntry>>) {
        // Phase 1: Collect locks needing renewal (synchronous, read lock)
        let locks_to_process: Vec<LockProcessInfo> = {
            let entries_guard = entries.read();
            entries_guard
                .iter()
                .filter(|(_, entry)| entry.next_renew_time <= Instant::now())
                .map(|(key, entry)| LockProcessInfo {
                    key: key.clone(),
                    client_id: entry.client_id.clone(),
                    ttl: entry.ttl,
                    max_duration: entry.max_duration,
                    start_time: entry.start_time,
                    lock_lost_tx: entry.lock_lost_tx.clone(),
                    cancel_token: entry.cancel_token.clone(),
                })
                .collect()
        };

        if locks_to_process.is_empty() {
            return;
        }

        // Phase 2a: Pre-filter — handle max_duration and cancelled entries
        //           without touching Redis.
        let mut to_renew: Vec<LockProcessInfo> = Vec::with_capacity(locks_to_process.len());
        for lock_info in locks_to_process {
            // Guard was dropped while we were waiting; entry already removed.
            if lock_info.cancel_token.is_cancelled() {
                continue;
            }
            if let Some(max_duration) = lock_info.max_duration
                && lock_info.start_time.elapsed() > max_duration {
                    warn!(
                        key = %lock_info.key,
                        client_id = %lock_info.client_id,
                        elapsed = ?lock_info.start_time.elapsed(),
                        max_duration = ?max_duration,
                        "lock exceeded max duration, stopping renewal"
                    );
                    if let Err(e) = lock_info
                        .lock_lost_tx
                        .send(DistLockLostReason::MaxDurationExceeded)
                        .await
                    {
                        error!(key = %lock_info.key, error = ?e,
                            "failed to send lock lost notification (channel closed)");
                    }
                    entries.write().remove(&lock_info.key);
                    continue;
                }
            to_renew.push(lock_info);
        }

        if to_renew.is_empty() {
            return;
        }

        // Phase 2b: Renew all due locks in a single Redis pipeline.
        //
        // N locks → 1 connection + 1 RTT, regardless of N.
        // Each command is an EVAL that atomically checks ownership and
        // extends TTL; results are returned in the same order.
        const LUA_SCRIPT: &str = r#"
            if redis.call("get", KEYS[1]) == ARGV[1] then
                return redis.call("expire", KEYS[1], ARGV[2])
            else
                return 0
            end
        "#;

        let mut conn = match redis.get().await {
            Ok(c) => c,
            Err(e) => {
                error!(
                    count = to_renew.len(),
                    error = ?e,
                    "failed to get Redis connection for batch renewal, will retry after 1s"
                );
                let retry_at = Instant::now() + Duration::from_secs(1);
                for lock_info in &to_renew {
                    Self::update_next_renew_time(entries, &lock_info.key, retry_at);
                }
                return;
            }
        };

        let mut pipe = redis::pipe();
        for lock_info in &to_renew {
            pipe.cmd("EVAL")
                .arg(LUA_SCRIPT)
                .arg(1)
                .arg(&lock_info.key)
                .arg(&lock_info.client_id)
                .arg(lock_info.ttl.as_secs() as i64);
        }

        match pipe.query_async::<Vec<i32>>(&mut *conn).await {
            Ok(results) => {
                for (lock_info, result) in to_renew.iter().zip(results.iter()) {
                    if *result == 1 {
                        let new_count = Self::update_renew_info(entries, &lock_info.key);
                        debug!(
                            key = %lock_info.key,
                            client_id = %lock_info.client_id,
                            renew_count = new_count,
                            ttl = ?lock_info.ttl,
                            "lock renewed successfully"
                        );
                    } else {
                        error!(
                            key = %lock_info.key,
                            client_id = %lock_info.client_id,
                            "lock renewal failed: held by another client or expired"
                        );
                        if let Err(e) = lock_info
                            .lock_lost_tx
                            .send(DistLockLostReason::RenewFailed)
                            .await
                        {
                            error!(key = %lock_info.key, error = ?e,
                                "failed to send lock lost notification (channel closed)");
                        }
                        entries.write().remove(&lock_info.key);
                    }
                }
            }
            Err(e) => {
                // Pipeline-level error (connection dropped mid-flight).
                // We cannot know which renewals succeeded, so conservatively
                // retry all after 1 second rather than notifying lock lost.
                error!(
                    count = to_renew.len(),
                    error = ?e,
                    "Redis pipeline error during batch renewal, will retry after 1s"
                );
                let retry_at = Instant::now() + Duration::from_secs(1);
                for lock_info in &to_renew {
                    Self::update_next_renew_time(entries, &lock_info.key, retry_at);
                }
            }
        }
    }

    /// Update renewal info after successful renewal
    ///
    /// Increments `renew_count` and calculates the next renewal time.
    /// Returns the new `renew_count` for logging.
    fn update_renew_info(entries: &RwLock<HashMap<String, WatchdogEntry>>, key: &str) -> usize {
        let mut entries_guard = entries.write();
        if let Some(entry) = entries_guard.get_mut(key) {
            entry.renew_count += 1;
            entry.next_renew_time = Instant::now() + entry.renew_interval;
            entry.renew_count
        } else {
            0
        }
    }

    /// Update next renewal time (used for retry after connection error)
    fn update_next_renew_time(
        entries: &RwLock<HashMap<String, WatchdogEntry>>,
        key: &str,
        next_time: Instant,
    ) {
        let mut entries_guard = entries.write();
        if let Some(entry) = entries_guard.get_mut(key) {
            entry.next_renew_time = next_time;
        }
    }
}

impl Drop for WatchdogManager {
    fn drop(&mut self) {
        self.stop();
    }
}
