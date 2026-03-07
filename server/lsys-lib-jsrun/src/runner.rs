//! Batch task runner – continuously accept and execute JS code in the background.
//!
//! ## Architecture
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────────────┐
//! │  JsTaskRunner                                                    │
//! │                                                                  │
//! │  submit(code, callback) ──► mpsc::channel ──► background loop    │
//! │                                                    │             │
//! │                                     ┌──────────────┴──────┐     │
//! │                                     │ spawn_blocking      │     │
//! │                                     │ per task             │     │
//! │                                     └──────────────┬──────┘     │
//! │                                                    │             │
//! │                                JsEngine::create_runtime()        │
//! │                                runtime.eval(code)                │
//! │                                drop(runtime) → free slot         │
//! │                                                    │             │
//! │                                  per-task callback(result)       │
//! │                                  oneshot → caller awaits         │
//! └──────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```rust,no_run
//! use lsys_lib_jsrun::{JsEngine, EngineConfig, RuntimeConfig};
//! use lsys_lib_jsrun::runner::JsTaskRunner;
//!
//! # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let engine = JsEngine::new(EngineConfig::default())?;
//! let runner = JsTaskRunner::new(engine, RuntimeConfig::default());
//!
//! // Submit with per-task callback (async)
//! let h1 = runner.submit("1 + 2", None, Some(|result: lsys_lib_jsrun::runner::TaskResult| async move {
//!     println!("Task {} done: {:?}", result.task_id, result.outcome);
//! }));
//!
//! // Submit without callback – just await the handle
//! let h2 = runner.submit_simple("3 * 4", None);
//!
//! let result = h1.await_result().await;
//! println!("Got: {:?}", result);
//!
//! runner.shutdown().await;
//! # Ok(()) }
//! ```

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use tokio::sync::{mpsc, oneshot, Notify};

use crate::runtime::{JsEngine, RuntimeConfig};

// ─── Task Outcome ────────────────────────────────────────────────────────────

/// The outcome of a single JS task execution.
#[derive(Debug, Clone)]
pub enum TaskOutcome {
    /// JS evaluation succeeded – contains the JSON-stringified return value.
    Success(String),
    /// JS evaluation failed or timed out.
    Error(String),
}

impl TaskOutcome {
    /// Returns `true` if the task succeeded.
    pub fn is_success(&self) -> bool {
        matches!(self, TaskOutcome::Success(_))
    }

    /// Returns `true` if the task failed.
    pub fn is_error(&self) -> bool {
        matches!(self, TaskOutcome::Error(_))
    }

    /// Convert into a `Result<String, String>`.
    pub fn into_result(self) -> std::result::Result<String, String> {
        match self {
            TaskOutcome::Success(v) => Ok(v),
            TaskOutcome::Error(e) => Err(e),
        }
    }
}

// ─── Task Result ─────────────────────────────────────────────────────────────

/// The full result of a completed task, including metadata.
#[derive(Debug, Clone)]
pub struct TaskResult {
    /// Unique task identifier (monotonically increasing).
    pub task_id: u64,
    /// The JS code that was executed.
    pub code: String,
    /// Execution outcome.
    pub outcome: TaskOutcome,
    /// Wall-clock execution duration.
    pub elapsed: std::time::Duration,
}

// ─── Per-task callback type ──────────────────────────────────────────────────

/// Async callback invoked when a specific task completes.
/// Returns a boxed future so the callback can perform async work.
pub type TaskCallback =
    Box<dyn FnOnce(TaskResult) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

// ─── Task Handle ─────────────────────────────────────────────────────────────

/// Handle returned by [`JsTaskRunner::submit`] – allows the caller to await
/// the result of a specific task.
pub struct TaskHandle {
    /// The unique task ID.
    pub task_id: u64,
    rx: oneshot::Receiver<TaskResult>,
}

impl TaskHandle {
    /// Wait for this specific task to complete and return its result.
    ///
    /// Returns `None` if the runner was dropped before the task completed.
    pub async fn await_result(self) -> Option<TaskResult> {
        self.rx.await.ok()
    }
}

// ─── Internal task envelope ──────────────────────────────────────────────────

struct TaskEnvelope {
    task_id: u64,
    code: String,
    /// Per-task RuntimeConfig override. `None` → use the runner default.
    runtime_config: Option<RuntimeConfig>,
    /// Optional per-task completion callback.
    on_complete: Option<TaskCallback>,
    /// Sender for the individual awaiter.
    result_tx: oneshot::Sender<TaskResult>,
}

// ─── JsTaskRunner ────────────────────────────────────────────────────────────

/// A batch task runner that continuously accepts and executes JS code
/// in the background using a [`JsEngine`].
///
/// Tasks are submitted via [`submit`](Self::submit) and executed concurrently
/// up to the engine's `max_runtimes` limit.  Each task gets its own
/// `JsRuntime` (and thus its own QuickJS heap) – isolation is preserved.
///
/// Each submitted task can carry its own completion callback.
pub struct JsTaskRunner {
    tx: mpsc::Sender<TaskEnvelope>,
    #[allow(dead_code)]
    engine: Arc<JsEngine>,
    next_id: AtomicU64,
    shutdown: Arc<Notify>,
    in_flight: Arc<AtomicU64>,
    all_done: Arc<Notify>,
    _bg_handle: tokio::task::JoinHandle<()>,
}

impl JsTaskRunner {
    /// Create and start a new task runner.
    ///
    /// * `engine` – shared JS engine (wrapped in `Arc` internally).
    /// * `default_config` – used for tasks that don't supply their own `RuntimeConfig`.
    pub fn new(engine: JsEngine, default_config: RuntimeConfig) -> Self {
        Self::with_capacity(engine, default_config, 256)
    }

    /// Create a runner with a custom channel capacity (back-pressure).
    pub fn with_capacity(
        engine: JsEngine,
        default_config: RuntimeConfig,
        channel_capacity: usize,
    ) -> Self {
        let engine = Arc::new(engine);
        let (tx, rx) = mpsc::channel::<TaskEnvelope>(channel_capacity);
        let shutdown = Arc::new(Notify::new());
        let in_flight = Arc::new(AtomicU64::new(0));
        let all_done = Arc::new(Notify::new());

        let bg_handle = {
            let engine = engine.clone();
            let shutdown = shutdown.clone();
            let in_flight = in_flight.clone();
            let all_done = all_done.clone();

            tokio::spawn(async move {
                background_loop(rx, engine, default_config, shutdown, in_flight, all_done).await;
            })
        };

        JsTaskRunner {
            tx,
            engine,
            next_id: AtomicU64::new(1),
            shutdown,
            in_flight,
            all_done,
            _bg_handle: bg_handle,
        }
    }

    // ── Submitting tasks ─────────────────────────────────────

    /// Submit JS code for background execution **with a per-task callback**.
    ///
    /// * `code` – JavaScript source to evaluate.
    /// * `runtime_config` – per-task config override (`None` = use default).
    /// * `on_complete` – optional callback fired when *this* task finishes.
    ///
    /// Returns a [`TaskHandle`] that can also be awaited for the result.
    ///
    /// This method will **not** block even if all runtime slots are occupied –
    /// the task is queued and executed as soon as a slot opens up.
    pub fn submit<F, Fut>(
        &self,
        code: impl Into<String>,
        runtime_config: Option<RuntimeConfig>,
        on_complete: Option<F>,
    ) -> TaskHandle
    where
        F: FnOnce(TaskResult) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let task_id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let (result_tx, result_rx) = oneshot::channel();

        let envelope = TaskEnvelope {
            task_id,
            code: code.into(),
            runtime_config,
            on_complete: on_complete.map(|f| {
                Box::new(
                    move |result: TaskResult| -> Pin<Box<dyn Future<Output = ()> + Send>> {
                        Box::pin(f(result))
                    },
                ) as TaskCallback
            }),
            result_tx,
        };

        // try_send keeps submit() synchronous. If the channel is full,
        // fall back to an async send in a spawned task.
        if let Err(mpsc::error::TrySendError::Full(envelope)) = self.tx.try_send(envelope) {
            let tx = self.tx.clone();
            tokio::spawn(async move {
                let _ = tx.send(envelope).await;
            });
        }

        TaskHandle {
            task_id,
            rx: result_rx,
        }
    }

    /// Convenience: submit without a callback (just await the handle).
    pub fn submit_simple(
        &self,
        code: impl Into<String>,
        runtime_config: Option<RuntimeConfig>,
    ) -> TaskHandle {
        self.submit(
            code,
            runtime_config,
            None::<fn(TaskResult) -> std::future::Ready<()>>,
        )
    }

    // ── Status ───────────────────────────────────────────────

    /// Number of tasks currently executing or queued.
    pub fn in_flight(&self) -> u64 {
        self.in_flight.load(Ordering::Relaxed)
    }

    /// Number of free runtime slots on the underlying engine.
    pub fn available_runtimes(&self) -> usize {
        self.engine.available_runtimes()
    }

    // ── Shutdown ─────────────────────────────────────────────

    /// Gracefully shut down the runner.
    ///
    /// 1. Stops accepting new tasks (channel is closed).
    /// 2. Waits for all in-flight tasks to finish.
    pub async fn shutdown(self) {
        drop(self.tx);
        self.shutdown.notify_one();

        while self.in_flight.load(Ordering::Acquire) > 0 {
            self.all_done.notified().await;
        }
    }
}

// ─── Background loop ────────────────────────────────────────────────────────

async fn background_loop(
    mut rx: mpsc::Receiver<TaskEnvelope>,
    engine: Arc<JsEngine>,
    default_config: RuntimeConfig,
    _shutdown: Arc<Notify>,
    in_flight: Arc<AtomicU64>,
    all_done: Arc<Notify>,
) {
    while let Some(envelope) = rx.recv().await {
        in_flight.fetch_add(1, Ordering::AcqRel);

        let engine = engine.clone();
        let cfg = envelope
            .runtime_config
            .unwrap_or_else(|| default_config.clone());
        let in_flight = in_flight.clone();
        let all_done = all_done.clone();

        // JsRuntime (QuickJS) is !Send – each task must live entirely on
        // one thread.  We use a plain OS thread so that:
        //   1. The !Send JsRuntime never crosses thread boundaries.
        //   2. `block_on_async` in core APIs detects no tokio context and
        //      safely calls `handle.block_on()` (the handle stored in
        //      RuntimeState points to the caller's multi-threaded runtime).
        //   3. We don't consume tokio worker threads with blocking JS work.
        std::thread::spawn(move || {
            let start = std::time::Instant::now();

            // Wrap the entire execution in catch_unwind so that a panic
            // (JS engine crash, OOM, etc.) is converted to TaskOutcome::Error
            // instead of silently leaving the record in Running state forever.
            let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                // Use the caller's tokio handle to drive async work.
                // JsRuntime is !Send so we must create + use + drop it
                // entirely within this closure (single thread).
                engine.tokio_handle().block_on(async {
                    match engine.create_runtime(cfg).await {
                        Ok(runtime) => match runtime.eval(&envelope.code).await {
                            Ok(val) => TaskOutcome::Success(val),
                            Err(e) => TaskOutcome::Error(e.to_string()),
                        },
                        Err(e) => TaskOutcome::Error(format!("Runtime creation failed: {}", e)),
                    }
                })
            }));

            let outcome = match panic_result {
                Ok(outcome) => outcome,
                Err(panic_val) => {
                    // Extract a human-readable message from the panic payload.
                    let msg = if let Some(s) = panic_val.downcast_ref::<&str>() {
                        format!("JS runtime panicked: {}", s)
                    } else if let Some(s) = panic_val.downcast_ref::<String>() {
                        format!("JS runtime panicked: {}", s)
                    } else {
                        "JS runtime panicked: <unknown>".to_string()
                    };
                    TaskOutcome::Error(msg)
                }
            };

            let result = TaskResult {
                task_id: envelope.task_id,
                code: envelope.code,
                outcome,
                elapsed: start.elapsed(),
            };

            // Fire per-task callback (async, driven on the caller's tokio runtime)
            if let Some(cb) = envelope.on_complete {
                engine.tokio_handle().block_on(cb(result.clone()));
            }

            // Notify awaiter
            let _ = envelope.result_tx.send(result);

            // Decrement in-flight counter
            if in_flight.fetch_sub(1, Ordering::AcqRel) == 1 {
                all_done.notify_waiters();
            }
        });
    }

    // Channel closed – wait for remaining in-flight tasks
    while in_flight.load(Ordering::Acquire) > 0 {
        all_done.notified().await;
    }
}
