//! Configuration module for distributed lock
//!
//! This module provides configuration structures for the distributed lock:
//! - `WatchdogConfig`: Controls automatic lock renewal behavior
//! - `DistLockConfig`: Main configuration combining all settings
//! - `DistLockConfigBuilder`: Builder pattern for creating configuration

use deadpool_redis::Pool;

use crate::config::Config;

/// Distributed lock configuration
///
/// Main configuration structure that combines Redis connection, key settings,
/// and watchdog configuration.
///
/// ## Creation Methods
///
/// 1. `from_config()`: Load from AppCore config (for production)
/// 2. `builder()`: Use builder pattern (for testing/custom setup)
///
/// ## Config Keys (for `from_config()`)
///
/// - `dist_lock_key_prefix`: Lock key prefix (default: empty)
#[derive(Debug)]
pub struct DistLockConfig {
    /// Redis connection pool for lock operations
    ///
    /// All lock operations (acquire, release, renew) use this pool.
    /// Must be configured with appropriate connection limits.
    pub redis: Pool,
    /// Key prefix for all locks (e.g., "myapp:")
    ///
    /// Helps avoid key collisions between different applications.
    /// Final key format: `{prefix}{key}`
    pub key_prefix: String,
}

impl DistLockConfig {
    /// Create configuration from AppCore config
    ///
    /// Loads configuration values from the application config file.
    /// Missing values use defaults.
    ///
    /// ## Config Keys
    ///
    /// | Key | Type | Default | Description |
    /// |-----|------|---------|-------------|
    /// | `dist_lock_key_prefix` | string | "" | Lock key prefix |
    pub fn from_config(redis: Pool, config: &Config) -> Self {
        let cfg = config.find(None);
        let key_prefix = cfg.get_string("dist_lock_key_prefix").unwrap_or_default();
        Self { redis, key_prefix }
    }

    /// Create a builder for custom configuration
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// let config = DistLockConfig::builder(redis_pool)
    ///     .key_prefix("myapp:")
    ///     .watchdog(WatchdogConfig {
    ///         renew_interval_ratio: 0.3,
    ///         max_duration: Some(Duration::from_secs(300)),
    ///     })
    ///     .build();
    /// ```
    pub fn builder(redis: Pool) -> DistLockConfigBuilder {
        DistLockConfigBuilder::new(redis)
    }
}

/// Configuration builder for DistLockConfig
///
/// Provides a fluent API for creating custom lock configurations.
/// All settings have sensible defaults, only override what you need.
///
/// ## Default Values
///
/// - `key_prefix`: empty string
#[derive(Debug)]
pub struct DistLockConfigBuilder {
    config: DistLockConfig,
}

impl DistLockConfigBuilder {
    fn new(redis: Pool) -> Self {
        Self {
            config: DistLockConfig {
                redis,
                key_prefix: String::new(),
            },
        }
    }

    /// Set key prefix for all locks
    ///
    /// Prefix is prepended to all lock keys: `{prefix}{key}`
    /// Useful for separating locks from different applications or environments.
    pub fn key_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.config.key_prefix = prefix.into();
        self
    }

    /// Build the final configuration
    ///
    /// Returns a `DistLockConfig` ready for use with `DistLock::new()`.
    pub fn build(self) -> DistLockConfig {
        self.config
    }
}
