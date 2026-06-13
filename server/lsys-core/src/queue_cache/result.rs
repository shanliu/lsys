//! Error types for queue cache operations
//!
//! This module defines all error types used by the queue cache layer:
//! - `QueueCacheError`: All possible errors during queue operations
//! - `QueueResult<T>`: Result type alias for queue operations
//!
//! ## Error Handling Philosophy
//!
//! 1. **No Panic**: All errors are returned as enum variants
//! 2. **I18n Support**: All errors implement `IntoFluentMessage` for internationalization
//! 3. **Detailed Context**: Each error includes relevant information (capacity, timeout, attempts, etc.)
//! 4. **From Conversions**: Automatic conversion from underlying library errors
//!
//! ## Error Categories
//!
//! | Category | Variants | Recovery |
//! |----------|----------|----------|
//! | Queue Operations | `QueueFull`, `QueueClosed` | Retry or abort |
//! | Serialization | `Serialization` | Check message format |
//! | Connection | `RabbitMQ`, `RabbitMQPool` | Retry with backoff |
//! | Timeout | `Timeout` | Retry or abort |
//! | Configuration | `Config` | Fix configuration |
//! | System | `System` | Depends on underlying issue |

use std::time::Duration;

use crate::fluent_message;
use crate::fluents::{FluentMessage, IntoFluentMessage};

/// Error types for queue cache operations
///
/// All errors that can occur during queue operations.
/// Each variant includes relevant context information for debugging and i18n.
///
/// ## Example Handling
///
/// ```rust,ignore
/// match queue.push(message).await {
///     Ok(()) => {
///         log::info!("Message pushed successfully");
///     }
///     Err(QueueCacheError::QueueFull { capacity }) => {
///         log::warn!("Queue full (capacity: {}), retrying later", capacity);
///         // Implement backoff and retry
///     }
///     Err(QueueCacheError::Serialization(e)) => {
///         log::error!("Invalid message format: {}", e);
///         // Log and reject message
///     }
///     Err(QueueCacheError::RabbitMQ(e)) => {
///         log::error!("RabbitMQ error: {}", e);
///         // Attempt reconnection
///     }
///     Err(e) => {
///         log::error!("Queue error: {}", e.to_fluent_message().default_format());
///     }
/// }
/// ```
#[derive(Debug)]
pub enum QueueCacheError {
    /// Queue is full and cannot accept more messages
    ///
    /// Occurs when pushing to a MemoryQueue that has reached its capacity limit.
    /// Contains the queue capacity for context.
    QueueFull { capacity: usize },

    /// Queue is closed and no longer accepts operations
    ///
    /// Occurs after calling shutdown() on a queue or consumer.
    QueueClosed,

    /// Message serialization/deserialization error
    ///
    /// Occurs when converting messages to/from bytes fails.
    /// Wraps the underlying serde_json error.
    Serialization(serde_json::Error),

    /// RabbitMQ operation error
    ///
    /// Low-level RabbitMQ error (connection, protocol, channel, etc.)
    /// Can be transient or permanent depending on the underlying error.
    #[cfg(feature = "queue-cache-rabbitmq")]
    RabbitMQ(lapin::Error),

    /// Yaque (disk-backed queue) I/O error
    ///
    /// Wraps std::io::Error from yaque filesystem operations.
    #[cfg(feature = "queue-cache-yaque")]
    Yaque(std::io::Error),

    /// Operation timeout
    ///
    /// Occurs when an operation (pop_blocking, shutdown) exceeds timeout duration.
    Timeout { timeout: Duration },

    /// Configuration error
    ///
    /// Invalid or missing configuration parameters.
    Config(String),

    /// System error
    ///
    /// General system-level errors (channel closed, lock poisoning, etc.)
    System(String),
}

/// Convert error to internationalized message
///
/// Each error type maps to a Fluent message key for i18n support.
/// Message keys should be defined in locale files.
///
/// ## Message Keys
///
/// | Error | Key | Parameters |
/// |-------|-----|------------|
/// | QueueFull | `queue-full` | `capacity` |
/// | QueueClosed | `queue-closed` | - |
/// | Serialization | `queue-serialization-error` | error message |
/// | RabbitMQ | `rabbitmq-error` | error message |
/// | RabbitMQPool | `rabbitmq-pool-error` | error message |
/// | Timeout | `queue-timeout` | `timeout` |
/// | Config | `queue-config-error` | error message |
/// | System | `queue-system-error` | error message |

impl IntoFluentMessage for QueueCacheError {
    fn to_fluent_message(&self) -> FluentMessage {
        match self {
            QueueCacheError::QueueFull { capacity } => {
                fluent_message!("queue-full", {"capacity": capacity})
            }
            QueueCacheError::QueueClosed => fluent_message!("queue-closed"),
            QueueCacheError::Serialization(err) => {
                fluent_message!("queue-serialization-error", err)
            }
            #[cfg(feature = "queue-cache-rabbitmq")]
            QueueCacheError::RabbitMQ(err) => {
                fluent_message!("rabbitmq-error", err)
            }
            #[cfg(feature = "queue-cache-yaque")]
            QueueCacheError::Yaque(err) => {
                fluent_message!("yaque-error", err)
            }
            QueueCacheError::Timeout { timeout } => {
                fluent_message!("queue-timeout", {"timeout": timeout.as_secs()})
            }
            QueueCacheError::Config(err) => {
                fluent_message!("queue-config-error", err)
            }
            QueueCacheError::System(err) => {
                fluent_message!("queue-system-error", err)
            }
        }
    }
}

/// Convert serde_json::Error to QueueCacheError
///
/// Allows using `?` operator with serialization operations.
impl From<serde_json::Error> for QueueCacheError {
    fn from(err: serde_json::Error) -> Self {
        QueueCacheError::Serialization(err)
    }
}

/// Convert lapin::Error to QueueCacheError
///
/// Allows using `?` operator with RabbitMQ operations.
#[cfg(feature = "queue-cache-rabbitmq")]
impl From<lapin::Error> for QueueCacheError {
    fn from(err: lapin::Error) -> Self {
        QueueCacheError::RabbitMQ(err)
    }
}

/// Convert std::io::Error to QueueCacheError
///
/// Allows using `?` operator with yaque filesystem operations.
#[cfg(feature = "queue-cache-yaque")]
impl From<std::io::Error> for QueueCacheError {
    fn from(err: std::io::Error) -> Self {
        QueueCacheError::Yaque(err)
    }
}

/// Result type alias for queue operations
///
/// All queue operations return this type.
/// `Ok(T)` indicates success, `Err(QueueCacheError)` indicates failure.
///
/// ## Usage
///
/// ```rust,ignore
/// async fn push_message(queue: &impl QueueBackend) -> QueueResult<()> {
///     queue.push(message).await?;
///     Ok(())
/// }
/// ```
pub type QueueResult<T> = Result<T, QueueCacheError>;
