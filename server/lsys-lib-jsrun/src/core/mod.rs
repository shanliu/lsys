//! Core API module – Rust-implemented atomic capabilities.
//!
//! All interfaces are mounted under `runtime.core` in JavaScript.

pub mod cache;
pub mod fetch;
pub mod file;

use rquickjs::function::Opt;
use rquickjs::{Class, Ctx, Function, Object, Result, Value};

use self::cache::JsCache;
use self::file::JsFile;

/// Run an async future synchronously from a JS callback context.
///
/// Detects the calling context and picks the right strategy:
/// - **Inside a tokio runtime** (worker or blocking thread) →
///   `block_in_place` + `handle.block_on`
/// - **Plain OS thread** (no tokio context) → `handle.block_on` directly
///
/// The `handle` should come from [`RuntimeState::tokio_handle`] and must
/// point to a **multi-threaded** tokio runtime.
pub(crate) fn block_on_async<F, R>(handle: &tokio::runtime::Handle, fut: F) -> R
where
    F: std::future::Future<Output = R>,
{
    if tokio::runtime::Handle::try_current().is_ok() {
        // On a tokio-managed thread (worker or blocking pool) we must use
        // block_in_place so we don't block the runtime.
        tokio::task::block_in_place(|| handle.block_on(fut))
    } else {
        // Plain OS thread – block_on is safe, block_in_place would panic.
        handle.block_on(fut)
    }
}
use crate::runtime::{
    LogHandler, RuntimeState, LOG_LEVEL_DEBUG, LOG_LEVEL_ERROR, LOG_LEVEL_INFO, LOG_LEVEL_TRACE,
    LOG_LEVEL_WARN, MESSAGE_TYPE_GET_ENV, MESSAGE_TYPE_GET_PARAM,
};

/// Register all core APIs onto the given `core` object inside the JS context.
pub fn register_core_api<'js>(
    ctx: &Ctx<'js>,
    core: &Object<'js>,
    state: &RuntimeState,
) -> Result<()> {
    // ── core.fetch ──────────────────────────────────────────────
    fetch::register_fetch(ctx, core, state)?;

    // ── core.File class ─────────────────────────────────────────
    Class::<JsFile>::define(core)?;

    // ── core.log(level, msg) ────────────────────────────────────
    {
        let handler: LogHandler = state.log_handler.clone();
        let namespace = state.config.namespace.clone();
        let handle = state.engine.tokio_handle.clone();
        core.set(
            "log",
            Function::new(ctx.clone(), move |level: u32, msg: String| {
                let handler = handler.clone();
                let ns = namespace.clone();
                block_on_async(&handle, async move {
                    handler(ns, level, msg).await;
                });
            })?,
        )?;
    }

    // ── core.LogLevel – well-known level constants ──────────────
    let log_level = Object::new(ctx.clone())?;
    log_level.set("TRACE", LOG_LEVEL_TRACE)?;
    log_level.set("DEBUG", LOG_LEVEL_DEBUG)?;
    log_level.set("INFO", LOG_LEVEL_INFO)?;
    log_level.set("WARN", LOG_LEVEL_WARN)?;
    log_level.set("ERROR", LOG_LEVEL_ERROR)?;
    core.set("LogLevel", log_level)?;

    // ── core.localTime(ms?) ──────────────────────────────────
    // Unified date/time primitive combining now + timezone + date-parts.
    //
    // Returns { now, year, month, day, hours, minutes, seconds, ms,
    //           weekday, offset, offsetName }
    //
    //   now        – current UTC epoch milliseconds (always real-time)
    //   year‥ms    – local-timezone date parts for the given `ms`
    //                (or current time when omitted / NaN)
    //   offset     – UTC offset in minutes (e.g. +480 for UTC+8)
    //   offsetName – human-readable offset (e.g. "+08:00")
    core.set(
        "localTime",
        Function::new(
            ctx.clone(),
            |ctx: Ctx<'js>, ms: Opt<f64>| -> Result<Value<'js>> {
                use chrono::{Datelike, TimeZone, Timelike};

                let now_ms = ::std::time::SystemTime::now()
                    .duration_since(::std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as f64;

                let dt = match ms.0 {
                    Some(m) if m.is_finite() => {
                        let secs = (m / 1000.0).floor() as i64;
                        let nsecs = ((m % 1000.0) * 1_000_000.0) as u32;
                        chrono::Local
                            .timestamp_opt(secs, nsecs)
                            .single()
                            .unwrap_or_else(chrono::Local::now)
                    }
                    _ => chrono::Local::now(),
                };

                let obj = Object::new(ctx.clone())?;
                obj.set("now", now_ms)?;
                obj.set("year", dt.year())?;
                obj.set("month", dt.month() as i32)?; // 1-12
                obj.set("day", dt.day() as i32)?; // 1-31
                obj.set("hours", dt.hour() as i32)?; // 0-23
                obj.set("minutes", dt.minute() as i32)?; // 0-59
                obj.set("seconds", dt.second() as i32)?; // 0-59
                obj.set("ms", (dt.nanosecond() / 1_000_000) as i32)?;
                obj.set("weekday", dt.weekday().num_days_from_sunday() as i32)?; // 0=Sun
                obj.set("offset", dt.offset().local_minus_utc() / 60)?;
                obj.set("offsetName", dt.offset().to_string())?;
                Ok(obj.into_value())
            },
        )?,
    )?;

    // ── core.Cache ──────────────────────────────────────────────
    let cache = state.engine.cache.clone();
    let cache_obj = JsCache::new(cache, state.config.namespace.clone());
    let cache_instance = Class::instance(ctx.clone(), cache_obj)?;
    core.set("Cache", cache_instance)?;

    // ── core.sleep(ms) ──────────────────────────────────────────
    {
        let handle = state.engine.tokio_handle.clone();
        core.set(
            "sleep",
            Function::new(ctx.clone(), move |ms: f64| {
                let duration = ::std::time::Duration::from_millis(ms.max(0.0) as u64);
                block_on_async(&handle, async move {
                    tokio::time::sleep(duration).await;
                });
            })?,
        )?;
    }

    // ── core.message – Web MessagePort style bridge ─────────────
    register_message(ctx, core, state)?;

    Ok(())
}

/// Register `core.message` object with `postMessage(type, data)` and `onMessage`.
///
/// Modelled after the Web MessagePort / BroadcastChannel API:
///   - `core.message.postMessage(type, data)` → calls Rust-side `MessageHandler`
///   - `core.message.GET_PARAM` → well-known message type identifier
///
/// The handler is async on the Rust side; JS sees a synchronous bridge
/// via `block_in_place` + `block_on`.
fn register_message<'js>(ctx: &Ctx<'js>, core: &Object<'js>, state: &RuntimeState) -> Result<()> {
    let handler = state.message_handler.clone();
    let namespace = state.config.namespace.clone();
    let handle = state.engine.tokio_handle.clone();
    let message_obj = Object::new(ctx.clone())?;

    // ── Well-known type constants (registered from Rust) ────────
    message_obj.set("GET_PARAM", MESSAGE_TYPE_GET_PARAM)?;
    message_obj.set("GET_ENV", MESSAGE_TYPE_GET_ENV)?;

    // ── postMessage(type, data) ─────────────────────────────────
    message_obj.set(
        "postMessage",
        Function::new(
            ctx.clone(),
            move |ctx: Ctx<'js>, msg_type: String, data: Value<'js>| -> Result<Value<'js>> {
                // JS value → JSON string → serde_json::Value
                let json_str = ctx
                    .json_stringify(data)?
                    .map(|s| s.to_string())
                    .transpose()?
                    .unwrap_or_else(|| "null".to_string());

                let data_val: serde_json::Value =
                    serde_json::from_str(&json_str).unwrap_or(serde_json::Value::Null);

                tracing::debug!(
                    target: "jsrun::message",
                    msg_type = %msg_type,
                    namespace = ?namespace,
                    "core.message.postMessage"
                );

                // Call the async Rust-side handler, blocking the JS thread
                let handler = handler.clone();
                let ns = namespace.clone();
                let result =
                    block_on_async(
                        &handle,
                        async move { handler(ns, msg_type, data_val).await },
                    );

                // serde_json::Value → JSON string → JS value
                let result_str = serde_json::to_string(&result).unwrap_or_else(|_| "null".into());
                ctx.eval(format!("({})", result_str))
            },
        )?,
    )?;

    core.set("message", message_obj)?;
    Ok(())
}
