//! Runtime module – JS engine (`JsEngine`) and individual JS containers (`JsRuntime`).
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │  JsEngine  (one per application)                            │
//! │  ┌────────────────────────────────────────────────────────┐ │
//! │  │  EngineConfig:                                         │ │
//! │  │    cache, fetch semaphore, max_runtimes                │ │
//! │  │    max_runtimes (concurrency limiter)                  │ │
//! │  └────────────────────────────────────────────────────────┘ │
//! │       │                    │                   │            │
//! │  ┌────▼────┐         ┌────▼────┐         ┌────▼────┐      │
//! │  │JsRuntime│         │JsRuntime│         │JsRuntime│      │
//! │  │  (own   │         │  (own   │         │  (own   │      │
//! │  │ QuickJS │         │ QuickJS │         │ QuickJS │      │
//! │  │ +workdir│         │ +workdir│         │ +workdir│      │
//! │  └─────────┘         └─────────┘         └─────────┘      │
//! └─────────────────────────────────────────────────────────────┘
//! ```

use std::collections::HashSet;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use rquickjs::{AsyncContext, AsyncRuntime, CatchResultExt, Object};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

use crate::core::cache::{new_shared_cache, SharedCache};
use crate::core::file::FileTracker;

// ─── Well-known message type identifiers ────────────────────────────────────

/// Message type identifier used by `runtime.std.getParam`.
pub const MESSAGE_TYPE_GET_PARAM: &str = "get_param";

/// Message type identifier used by `runtime.std.getEnv`.
pub const MESSAGE_TYPE_GET_ENV: &str = "get_env";

// ─── Log level constants (exposed to JS as core.LogLevel) ───────────────────

pub const LOG_LEVEL_TRACE: u32 = 0;
pub const LOG_LEVEL_DEBUG: u32 = 1;
pub const LOG_LEVEL_INFO: u32 = 2;
pub const LOG_LEVEL_WARN: u32 = 3;
pub const LOG_LEVEL_ERROR: u32 = 4;

/// The type of the `core.message` async callback.
///
/// The host provides an async closure that receives the message-type string
/// and the payload `serde_json::Value` (serialised from the JS argument)
/// and returns a `serde_json::Value` that will be deserialised back to JS.
///
/// ```rust,ignore
/// let handler: MessageHandler = Arc::new(|namespace, msg_type, data| {
///     Box::pin(async move {
///         // namespace: which runtime (namespace) the call came from
///         serde_json::json!({ "ok": true })
///     })
/// });
/// ```
pub type MessageHandler = Arc<
    dyn Fn(
            Option<String>,    // runtime namespace (if set)
            String,            // message type
            serde_json::Value, // payload
        ) -> Pin<Box<dyn Future<Output = serde_json::Value> + Send>>
        + Send
        + Sync,
>;

/// The type of the `core.log` async callback.
///
/// The host provides an async closure that receives the namespace, log level,
/// and message string.  This allows the host to route logs to a database,
/// file, network sink, or any async destination.
///
/// ```rust,ignore
/// use lsys_lib_jsrun::{LogHandler, LOG_LEVEL_INFO};
///
/// let handler: LogHandler = Arc::new(|namespace, level, msg| {
///     Box::pin(async move {
///         // e.g. insert into database
///         println!("[ns={:?}] [level={}] {}", namespace, level, msg);
///     })
/// });
/// ```
pub type LogHandler = Arc<
    dyn Fn(
            Option<String>, // runtime namespace (if set)
            u32,            // log level (LOG_LEVEL_*)
            String,         // message
        ) -> Pin<Box<dyn Future<Output = ()> + Send>>
        + Send
        + Sync,
>;

/// The type of the `file.local_sync()` async callback.
///
/// The host provides an async closure that receives the runtime namespace,
/// the file's full path, and the working directory.  The host is expected to
/// implement file synchronisation (e.g. upload to object storage, replicate
/// to another node, etc.) and return a `serde_json::Value` result on success.
///
/// ```rust,ignore
/// use lsys_lib_jsrun::FileLocalSyncHandler;
///
/// let handler: FileLocalSyncHandler = Arc::new(|namespace, file_path, work_dir| {
///     Box::pin(async move {
///         // e.g. upload file_path to remote storage
///         Ok(serde_json::json!({ "synced": true }))
///     })
/// });
/// ```
pub type FileLocalSyncHandler = Arc<
    dyn Fn(
            Option<String>, // runtime namespace (if set)
            PathBuf,        // full file path
            PathBuf,        // work directory
        ) -> Pin<Box<dyn Future<Output = Result<serde_json::Value, String>> + Send>>
        + Send
        + Sync,
>;

// ─── Engine Configuration ────────────────────────────────────────────────────

/// Configuration for the JS engine (shared across all runtimes).
///
/// These parameters are shared across **all** `JsRuntime` instances created
/// from the same `JsEngine`.
#[derive(Clone)]
pub struct EngineConfig {
    /// Maximum concurrent fetch requests (shared across all runtimes).
    pub max_concurrent_fetches: usize,
    /// Cache capacity (number of entries).
    pub cache_capacity: u64,
    /// Default cache TTL.
    pub cache_default_ttl: Duration,
    /// Interval for periodic cache cleanup. Set to `Duration::ZERO` to disable.
    pub cache_cleanup_interval: Duration,
    /// Maximum number of concurrent `JsRuntime` instances.
    /// `create_runtime` will wait when this limit is reached;
    /// `try_create_runtime` will return `None` immediately.
    pub max_runtimes: usize,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            max_concurrent_fetches: 8,
            cache_capacity: 1024,
            cache_default_ttl: Duration::from_secs(300),
            cache_cleanup_interval: Duration::from_secs(30),
            max_runtimes: 4,
        }
    }
}

// ─── Per-Runtime Configuration ──────────────────────────────────────────────

/// Configuration for an individual `JsRuntime` instance.
#[derive(Clone)]
pub struct RuntimeConfig {
    /// Maximum JS heap memory in bytes (0 = unlimited).
    pub memory_limit: usize,
    /// Maximum execution wall-clock time.
    pub execution_timeout: Duration,
    /// Per-fetch request timeout.
    pub fetch_timeout: Duration,
    /// Optional host allow-list for fetch.  `None` = all hosts allowed.
    pub host_allow_list: Option<HashSet<String>>,
    /// Deny requests to private / internal IPs (SSRF protection).
    pub deny_private_ip: bool,
    /// Working directory for file operations (each runtime can have its own).
    pub work_dir: PathBuf,
    /// The `core.message` async callback.  JS calls `core.message.postMessage(type, data)`
    /// and this closure receives the type string + serialised payload, returns the result.
    /// Defaults to an identity function that echoes the data back.
    pub message_handler: Option<MessageHandler>,
    /// The `core.log` async callback.  JS calls `core.log(level, msg)` and this
    /// closure receives the namespace, level, and message.  Defaults to a
    /// tracing-based logger.
    pub log_handler: Option<LogHandler>,
    /// The `file.local_sync()` async callback.  JS calls `file.local_sync()` and this
    /// closure receives the namespace, file full path, and work directory.
    /// The host implements the actual file synchronisation logic.
    /// Defaults to `None` – calling `file.local_sync()` without a handler will throw.
    pub file_sync_handler: Option<FileLocalSyncHandler>,
    /// Optional namespace for this runtime.
    ///
    /// When set, cache keys are automatically prefixed with `"{namespace}:"`
    /// to achieve cache isolation, and the namespace string is forwarded to
    /// the `MessageHandler` so the host can identify which runtime the
    /// request came from.
    pub namespace: Option<String>,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            memory_limit: 64 * 1024 * 1024, // 64 MiB
            execution_timeout: Duration::from_secs(30),
            fetch_timeout: Duration::from_secs(15),
            host_allow_list: None,
            deny_private_ip: true,
            message_handler: None,
            log_handler: None,
            file_sync_handler: None,
            work_dir: std::env::temp_dir().join("lsys-jsrun-files"),
            namespace: None,
        }
    }
}

// ─── Shared state visible to core APIs ──────────────────────────────────────

pub struct RuntimeState {
    pub engine: Arc<EngineInner>,
    pub config: RuntimeConfig,
    pub message_handler: MessageHandler,
    pub log_handler: LogHandler,
    pub file_tracker: FileTracker,
    // pub namespace: Option<String>,
    // The tokio runtime handle used by core APIs to bridge async → sync.
    // This must point to a **multi-threaded** runtime so that
    // `block_in_place` works correctly.
    // pub tokio_handle: tokio::runtime::Handle,
}

// ─── JsEngine ───────────────────────────────────────────────────────────────

/// The JS engine – manages shared resources and controls the total number
/// of concurrent `JsRuntime` instances.
///
/// Create one per application, then call [`create_runtime`](Self::create_runtime)
/// to spawn isolated JS contexts that share the cache, fetch concurrency pool,
/// and message handler.
///
/// ```rust,no_run
/// use lsys_lib_jsrun::{JsEngine, EngineConfig, RuntimeConfig};
///
/// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let engine = JsEngine::new(EngineConfig::default())?;
///
/// let rt1 = engine.create_runtime(RuntimeConfig::default()).await?;
/// let rt2 = engine.create_runtime(RuntimeConfig::default()).await?;
///
/// let r1 = rt1.eval("1 + 2").await?;
/// let r2 = rt2.eval("3 + 4").await?;
/// # Ok(()) }
/// ```
/// Shared inner state of a `JsEngine`, wrapped in `Arc` so that `RuntimeState`
/// can hold a reference without tying lifetimes to `&self`.
///
pub struct EngineInner {
    pub http_client: reqwest::Client,
    pub fetch_semaphore: Arc<Semaphore>,
    pub cache: SharedCache,
    pub runtime_semaphore: Arc<Semaphore>,
    /// Handle to the tokio runtime that created this engine.
    pub tokio_handle: tokio::runtime::Handle,
}

pub struct JsEngine {
    inner: Arc<EngineInner>,
    cache_cleanup_interval: Duration,
}

impl JsEngine {


    /// Create a new JS engine with the given configuration.
    pub fn new(config: EngineConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let cache = new_shared_cache(config.cache_capacity, config.cache_default_ttl);

        Ok(Self {
            inner: Arc::new(EngineInner {
                http_client: reqwest::Client::new(),
                fetch_semaphore: Arc::new(Semaphore::new(config.max_concurrent_fetches)),
                cache,
                runtime_semaphore: Arc::new(Semaphore::new(config.max_runtimes)),
                tokio_handle: tokio::runtime::Handle::current(),
            }),
            cache_cleanup_interval: config.cache_cleanup_interval,
        })
    }

    /// 运行缓存清理后台循环。
    /// 若 `cache_cleanup_interval` 为零则立即返回。
    /// 通常通过 `tokio::spawn` 调用：
    /// ```rust,ignore
    /// let engine = Arc::new(JsEngine::new(config)?);
    /// tokio::spawn({ let e = engine.clone(); async move { e.run_cache_cleanup().await; } });
    /// ```
    pub async fn run_cache_cleanup(&self) {
        if self.cache_cleanup_interval.is_zero() {
            return;
        }
        Self::cache_cleanup_loop(self.inner.cache.clone(), self.cache_cleanup_interval).await;
    }

        /// Internal: background task loop for periodic cache cleanup.
    async fn cache_cleanup_loop(cache: SharedCache, interval: Duration) {
        let mut ticker = tokio::time::interval(interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            ticker.tick().await;
            let removed = cache.cleanup_expired();
            if removed > 0 {
                tracing::debug!(
                    target: "jsrun::cache",
                    removed = removed,
                    "cache cleanup removed expired entries"
                );
            }
        }
    }

    /// Build a [`RuntimeState`] by combining shared resources with per-runtime config.
    fn build_state(&self, config: RuntimeConfig) -> RuntimeState {
        // default message handler: echo data back
        let message_handler: MessageHandler = config
            .message_handler
            .clone()
            .unwrap_or_else(|| Arc::new(|_ns, _msg_type, data| Box::pin(async move { data })));

        // default log handler: delegate to tracing
        let log_handler: LogHandler = config.log_handler.clone().unwrap_or_else(|| {
            Arc::new(|_ns, level, msg| {
                Box::pin(async move {
                    match level {
                        LOG_LEVEL_TRACE => tracing::trace!(target: "jsrun::user", "{}", msg),
                        LOG_LEVEL_DEBUG => tracing::debug!(target: "jsrun::user", "{}", msg),
                        LOG_LEVEL_INFO => tracing::info!(target: "jsrun::user", "{}", msg),
                        LOG_LEVEL_WARN => tracing::warn!(target: "jsrun::user", "{}", msg),
                        LOG_LEVEL_ERROR => tracing::error!(target: "jsrun::user", "{}", msg),
                        _ => tracing::info!(target: "jsrun::user", "{}", msg),
                    }
                })
            })
        });

        RuntimeState {
            engine: self.inner.clone(),
            message_handler,
            log_handler,
            file_tracker: FileTracker::new(
                config.work_dir.clone(),
                config.file_sync_handler.clone(),
                config.namespace.clone(),
                self.inner.tokio_handle.clone(),
            ),
            config,
        }
    }

    /// Internal helper: initialise a QuickJS context and wire up core + std.
    async fn init_runtime(
        &self,
        config: RuntimeConfig,
        permit: OwnedSemaphorePermit,
    ) -> Result<JsRuntime, Box<dyn std::error::Error>> {
        // ensure per-runtime work directory exists
        std::fs::create_dir_all(&config.work_dir)?;

        let rt = AsyncRuntime::new()?;

        if config.memory_limit > 0 {
            rt.set_memory_limit(config.memory_limit).await;
        }

        let ctx = AsyncContext::full(&rt).await?;
        let state = Arc::new(self.build_state(config.clone()));

        // install core & std inside the context
        let st = state.clone();
        ctx.with(|ctx| {
            // Store FileTracker as context userdata
            let _ = ctx
                .store_userdata(st.file_tracker.clone())
                .map_err(|_| rquickjs::Error::Unknown)?;

            // Create `runtime` global namespace
            let globals = ctx.globals();
            let runtime_obj = Object::new(ctx.clone())?;

            // Create `runtime.core` and register all core APIs
            let core_obj = Object::new(ctx.clone())?;
            crate::core::register_core_api(&ctx, &core_obj, &st)?;
            runtime_obj.set("core", core_obj)?;
            globals.set("runtime", runtime_obj)?;

            // Inject std JS scripts (console, import, utils)
            crate::std::inject_std(&ctx)?;

            Ok::<(), rquickjs::Error>(())
        })
        .await?;

        Ok(JsRuntime {
            rt,
            ctx,
            config,
            state,
            _permit: permit,
        })
    }

    /// Create a new `JsRuntime` instance.
    ///
    /// If the maximum number of concurrent runtimes has been reached this method
    /// will **wait** until a slot becomes available (i.e. another `JsRuntime` is
    /// dropped).  Use [`try_create_runtime`](Self::try_create_runtime) for a
    /// non-blocking alternative.
    pub async fn create_runtime(
        &self,
        config: RuntimeConfig,
    ) -> Result<JsRuntime, Box<dyn std::error::Error>> {
        let permit = self
            .inner
            .runtime_semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|e| format!("Failed to acquire runtime permit: {}", e))?;

        self.init_runtime(config, permit).await
    }

    /// Try to create a new `JsRuntime` **without** waiting.
    ///
    /// Returns `Ok(None)` immediately if the maximum number of concurrent
    /// runtimes has been reached.
    pub async fn try_create_runtime(
        &self,
        config: RuntimeConfig,
    ) -> Result<Option<JsRuntime>, Box<dyn std::error::Error>> {
        let permit = match self.inner.runtime_semaphore.clone().try_acquire_owned() {
            Ok(p) => p,
            Err(_) => return Ok(None),
        };

        Ok(Some(self.init_runtime(config, permit).await?))
    }

    /// Access the shared cache (readable from the Rust side).
    pub fn cache(&self) -> &SharedCache {
        &self.inner.cache
    }

    /// Number of runtime slots still available before `create_runtime` blocks.
    pub fn available_runtimes(&self) -> usize {
        self.inner.runtime_semaphore.available_permits()
    }

    /// Returns the tokio runtime handle captured when the engine was created.
    pub fn tokio_handle(&self) -> &tokio::runtime::Handle {
        &self.inner.tokio_handle
    }
}

// ─── JsRuntime ──────────────────────────────────────────────────────────────

/// An isolated JavaScript execution context.
///
/// Created via [`JsEngine::create_runtime`].  Each instance has its own
/// QuickJS heap and context but shares the cache, fetch concurrency pool,
/// and message handler with its siblings.  Each runtime has its own `work_dir`.
///
/// Dropping the `JsRuntime` releases the concurrency permit back to the pool,
/// closes all files opened by this runtime, and tears down the QuickJS context.
pub struct JsRuntime {
    rt: AsyncRuntime,
    ctx: AsyncContext,
    config: RuntimeConfig,
    state: Arc<RuntimeState>,
    /// Held for its `Drop` impl – releasing the slot back to the pool.
    _permit: OwnedSemaphorePermit,
}

impl JsRuntime {
    /// Evaluate a JavaScript string and return the result as a JSON-ish string.
    pub async fn eval(&self, code: &str) -> Result<String, Box<dyn std::error::Error>> {
        let code = code.to_string();
        let timeout = self.config.execution_timeout;

        let result = tokio::time::timeout(timeout, async {
            self.ctx
                .with(|ctx| {
                    let val: rquickjs::Value = ctx.eval(code.clone()).catch(&ctx).map_err(|e| {
                        let msg = match e {
                            rquickjs::CaughtError::Error(e) => format!("{e}"),
                            rquickjs::CaughtError::Exception(ex) => {
                                format!("Exception: {}", ex)
                            }
                            rquickjs::CaughtError::Value(v) => {
                                format!("Thrown: {:?}", v)
                            }
                        };
                        rquickjs::Error::new_from_js_message("eval", "value", msg)
                    })?;

                    // try to JSON-stringify the result for a stable representation
                    if val.is_undefined() {
                        Ok("undefined".to_string())
                    } else if val.is_null() {
                        Ok("null".to_string())
                    } else {
                        match ctx.json_stringify(val) {
                            Ok(Some(s)) => s.to_string().map_err(|e| {
                                rquickjs::Error::new_from_js_message(
                                    "value",
                                    "string",
                                    e.to_string(),
                                )
                            }),
                            Ok(None) => Ok("undefined".to_string()),
                            Err(_) => Ok("[unstringifiable]".to_string()),
                        }
                    }
                })
                .await
        })
        .await;

        match result {
            Ok(inner) => Ok(inner?),
            Err(_) => {
                // timeout – interrupt the runtime
                self.rt.set_interrupt_handler(Some(Box::new(|| true))).await;
                Err("Script execution timed out".into())
            }
        }
    }

    /// Access the underlying shared cache.
    pub fn cache(&self) -> &SharedCache {
        &self.state.engine.cache
    }
}

impl Drop for JsRuntime {
    fn drop(&mut self) {
        // close all open files tracked by this runtime
        self.state.file_tracker.close_all();
        // `_permit` is automatically released when dropped,
        // freeing a slot in the shared pool.
    }
}
