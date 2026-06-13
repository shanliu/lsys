//! Error types for distributed lock operations
//!
//! This module defines all error types used by the distributed lock:
//! - `DistLockLostReason`: Why a lock was lost (for monitoring)
//! - `DistLockError`: All possible errors during lock operations
//!
//! ## Error Handling Philosophy
//!
//! 1. **No Panic**: All errors are returned as enum variants
//! 2. **I18n Support**: All errors implement `IntoFluentMessage`
//! 3. **Detailed Context**: Each error includes relevant information (key, timeout, etc.)
//! 4. **Recovery**: Connection errors allow retry, other errors are terminal

use std::time::Duration;

use deadpool_redis::PoolError;
use redis::RedisError;

use crate::fluent_message;
use crate::fluents::{FluentMessage, IntoFluentMessage};

/// Reason why a lock was lost
///
/// This enum is used to notify the lock holder why their lock is no longer valid.
/// Sent via the `lock_lost_rx` channel when watchdog detects a problem.
///
/// ## Usage
///
/// ```rust,ignore
/// // Check for lock loss notification
/// if let Some(reason) = guard.try_wait_lock_lost() {
///     match reason {
///         DistLockLostReason::ConnectionError => {
///             // Redis connection failed, may recover
///         }
///         DistLockLostReason::RenewFailed => {
///             // Another client acquired the lock
///         }
///         DistLockLostReason::MaxDurationExceeded => {
///             // Lock held too long, intentional limit
///         }
///         _ => {}
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub enum DistLockLostReason {
    /// Renewal failed: lock was acquired by another client
    ///
    /// This means another client successfully acquired the lock.
    /// The lock is definitely lost and work should be aborted.
    RenewFailed,
    /// Maximum lock holding duration exceeded
    ///
    /// Configured via `WatchdogConfig.max_duration`.
    /// The watchdog intentionally stopped renewing to prevent indefinite holding.
    MaxDurationExceeded,
}

/// Error types for distributed lock operations
///
/// All errors that can occur during lock acquisition, release, or monitoring.
/// Each variant includes relevant context information.
///
/// ## Error Categories
///
/// | Category | Variants | Recovery |
/// |----------|----------|----------|
/// | Acquisition | `AcquireFailed`, `AcquireTimeout` | Retry or abort |
/// | Lock Lost | `LockLost` | Abort work |
/// | Ownership | | Bug in code |
/// | Infrastructure | `Redis`, `RedisPool` | Retry after fix |
///
/// ## Example Handling
///
/// ```rust,ignore
/// match lock.lock_timeout(Duration::from_secs(5)).await {
///     Ok(guard) => {
///         // Work with lock
///     }
///     Err(DistLockError::AcquireTimeout { key, timeout }) => {
///         // Lock busy, timeout exceeded
///         log::warn!("Lock {} busy after {}s", key, timeout.as_secs());
///     }
///     Err(DistLockError::Redis(e)) => {
///         // Redis connection issue
///         log::error!("Redis error: {}", e);
///     }
///     Err(e) => {
///         // Other errors
///         log::error!("Lock error: {}", e.to_fluent_message());
///     }
/// }
/// ```
#[derive(Debug)]
pub enum DistLockError {
    /// Lock acquisition failed after all retries
    ///
    /// The lock is held by another client and retry limit was reached.
    /// Contains the key that failed to acquire.
    AcquireFailed { key: String },
    /// Lock acquisition timed out
    ///
    /// The lock was still held by another client when timeout expired.
    /// Contains the key and the timeout duration.
    AcquireTimeout { key: String, timeout: Duration },
    /// Lock was lost while holding it
    ///
    /// The watchdog detected that the lock is no longer valid.
    /// Contains the key and the reason for loss.
    LockLost {
        key: String,
        reason: DistLockLostReason,
    },
    /// Redis operation error
    ///
    /// Low-level Redis error (connection, protocol, etc.)
    /// Can be transient or permanent depending on the underlying error.
    Redis(RedisError),
    /// Redis connection pool error
    ///
    /// Error getting connection from pool (timeout, exhausted, etc.)
    RedisPool(PoolError),
}

/// Convert error to internationalized message
///
/// Each error type maps to a Fluent message key for i18n support.
/// Message keys are defined in locale files.
///
/// ## Message Keys
///
/// | Error | Key | Parameters |
/// |-------|-----|------------|
/// | AcquireFailed | `lock-acquire-failed` | `key` |
/// | AcquireTimeout | `lock-acquire-timeout` | `key`, `timeout` |
/// | LockLost | `lock-lost` | `key`, `reason` |
/// | Redis/RedisPool | `redis-error` | error message |
impl IntoFluentMessage for DistLockError {
    fn to_fluent_message(&self) -> FluentMessage {
        match self {
            DistLockError::AcquireFailed { key } => {
                fluent_message!("lock-acquire-failed", key)
            }
            DistLockError::AcquireTimeout { key, timeout } => {
                fluent_message!("lock-acquire-timeout", {"key": key, "timeout": timeout.as_secs()})
            }
            DistLockError::LockLost { key, reason } => {
                let reason_str = match reason {
                    DistLockLostReason::RenewFailed => "renew_failed",
                    DistLockLostReason::MaxDurationExceeded => "max_duration_exceeded",
                };
                fluent_message!("lock-lost", {"key": key, "reason": reason_str})
            }
            DistLockError::Redis(err) => {
                fluent_message!("redis-error", err)
            }
            DistLockError::RedisPool(err) => {
                fluent_message!("redis-error", err)
            }
        }
    }
}

/// Convert RedisError to DistLockError
///
/// Allows using `?` operator with Redis operations.
impl From<RedisError> for DistLockError {
    fn from(err: RedisError) -> Self {
        DistLockError::Redis(err)
    }
}

/// Convert PoolError to DistLockError
///
/// Allows using `?` operator with pool operations.
impl From<PoolError> for DistLockError {
    fn from(err: PoolError) -> Self {
        DistLockError::RedisPool(err)
    }
}
