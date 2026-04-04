//! # lsys-jsrun
//!
//! A sandboxed JavaScript runtime library built on QuickJS (via rquickjs).
//!
//! ## Architecture
//!
//! The runtime is organized in three layers:
//!
//! - **core** – Rust-implemented atomic APIs exposed as `runtime.core.*`
//! - **std**  – JavaScript-implemented standard library exposed as `runtime.std.*`
//! - **user** – User scripts that may call both `core` and `std`
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use lsys_lib_jsrun::{JsEngine, EngineConfig, RuntimeConfig};
//!
//! #[tokio::main]
//! async fn main() {
//!     // 1. Create engine (cache, fetch pool, message handler …)
//!     let engine = JsEngine::new(EngineConfig::default()).unwrap();
//!
//!     // 2. Spawn isolated runtimes from the engine
//!     let rt1 = engine.create_runtime(RuntimeConfig::default()).await.unwrap();
//!     let rt2 = engine.create_runtime(RuntimeConfig::default()).await.unwrap();
//!
//!     // 3. Evaluate JS independently – cache is shared, heaps are isolated
//!     let r1 = rt1.eval("1 + 2").await.unwrap();
//!     let r2 = rt2.eval("3 + 4").await.unwrap();
//!     assert_eq!(r1, "3");
//!     assert_eq!(r2, "7");
//! }
//! ```

pub mod core;
pub mod runner;
pub mod runtime;
pub mod std;
pub mod utils;

pub use runner::{JsTaskRunner, TaskCallback, TaskHandle, TaskOutcome, TaskResult};
pub use runtime::{
    EngineConfig, FileLocalSyncHandler, JsEngine, JsRuntime, LogHandler,
    MessageHandler, RuntimeConfig, LOG_LEVEL_DEBUG, LOG_LEVEL_ERROR, LOG_LEVEL_INFO,
    LOG_LEVEL_TRACE, LOG_LEVEL_WARN, MESSAGE_TYPE_GET_ENV, MESSAGE_TYPE_GET_PARAM,
};
pub use utils::{check_js_syntax, JsSyntaxError};
