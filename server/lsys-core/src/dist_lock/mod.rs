//! Distributed Lock Module
//!
//! This module provides a Redis-based distributed lock implementation with the following features:
//!
//! ## Features
//!
//! - **Atomic Lock Acquisition**: Uses Redis SET NX EX command for atomic lock acquisition
//! - **Automatic Renewal (Watchdog)**: Background task automatically extends lock TTL to prevent expiration
//! - **Lock State Monitoring**: Provides `is_held()` to check lock status and `wait_lock_lost()` for notifications
//! - **Retry Strategy**: Configurable timeout and interval for lock acquisition retry
//! - **Safe Release**: Lua scripts ensure only the lock owner can release the lock
//! - **Graceful Degradation**: No panic on errors, proper logging with tracing
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
//! │   DistLock      │────▶│  WatchdogManager │────▶│     Redis       │
//! │  (Lock Client)  │     │  (Auto-Renewal)  │     │  (Storage)      │
//! └─────────────────┘     └──────────────────┘     └─────────────────┘
//!         │                       │
//!         ▼                       ▼
//! ┌─────────────────┐     ┌──────────────────┐
//! │ DistLockGuard   │     │  WatchdogEntry   │
//! │ (Lock Holder)   │     │  (Lock Info)     │
//! └─────────────────┘     └──────────────────┘
//! ```
//!
//! ## Usage Example
//!
//! ```rust,ignore
//! use lsys_core::dist_lock::{DistLock, DistLockConfig, WatchdogConfig, RetryStrategy};
//! use std::sync::Arc;
//! use std::time::Duration;
//!
//! // Create configuration
//! let config = Arc::new(DistLockConfig::builder(redis_pool)
//!     .key_prefix("myapp:")
//!     .watchdog(WatchdogConfig {
//!         enabled: true,
//!         renew_interval_ratio: 0.3,
//!         max_duration: Some(Duration::from_secs(300)),
//!     })
//!     .build());
//!
//! // Create locker instance (once per config, reuse for many keys)
//! let locker = Arc::new(DistLock::new(config));
//! let ttl = Duration::from_secs(30);
//!
//! // Single attempt, no retry
//! let guard = locker.try_lock("resource_key", ttl).await?;
//!
//! // Or with retry strategy
//! let retry = RetryStrategy {
//!     timeout: Some(Duration::from_secs(5)),
//!     retry_interval: Duration::from_millis(100),
//! };
//! let guard = locker.lock("resource_key", ttl, retry).await?;
//!
//! // Or with watchdog (auto-renewal)
//! let guard = locker.try_lock_with_watchdog("resource_key", ttl).await?;
//!
//! // Check lock status
//! if guard.is_held().await? {
//!     // Do work under lock protection
//! }
//!
//! // Lock is automatically released when guard is dropped
//! ```
//!
//! ## Lock Acquisition Flow
//!
//! 1. Client calls `try_lock()` or `lock()` with retry strategy
//! 2. Lua script executes `SET key value NX EX ttl` atomically
//! 3. If successful, watchdog is registered for auto-renewal (if enabled)
//! 4. `DistLockGuard` is returned to hold the lock
//! 5. On drop, lock is released via Lua script and watchdog is unregistered
//!
//! ## Watchdog Renewal Flow
//!
//! 1. WatchdogManager runs background task every 100ms
//! 2. For each registered lock, checks if renewal is needed
//! 3. If `next_renew_time` passed, executes Lua script to extend TTL
//! 4. If renewal fails, sends notification via `lock_lost_tx` channel
//! 5. Limits: `max_duration` can stop renewal
//!
//! ## Error Handling
//!
//! - All errors are returned as `DistLockError` enum
//! - No panic on any error condition
//! - All errors implement `IntoFluentMessage` for i18n
//! - Connection errors trigger retry, not immediate failure

mod config;
mod guard;
mod lock;
mod result;
mod watchdog;

pub use config::{DistLockConfig, DistLockConfigBuilder};
pub use guard::DistLockGuard;
pub use lock::{DistLock, RetryStrategy, WatchdogConfig};
pub use result::{DistLockError, DistLockLostReason};
