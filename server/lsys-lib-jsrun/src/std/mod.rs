//! Standard library – JavaScript implementations injected at startup.
//!
//! Each JS module is embedded via `include_str!` and evaluated during
//! runtime initialization.  The results are mounted on `runtime.std`.

use rquickjs::{Ctx, Result as JsResult};

/// The console implementation (log, error, warn, info, time, timeEnd).
pub const CONSOLE_JS: &str = include_str!("js/console.js");

/// The module loader / import implementation.
pub const IMPORT_JS: &str = include_str!("js/import.js");

/// Crypto helpers (md5, sha1, sha256, hmacSha256).
pub const CRYPTO_JS: &str = include_str!("js/crypto.js");

/// Encoding helpers (base64, url encode/decode).
pub const ENCODING_JS: &str = include_str!("js/encoding.js");

/// Parameter & environment variable bridge (getParam, getEnv).
pub const PARAMS_JS: &str = include_str!("js/params.js");

/// Utility helpers (sleep, getRandomValues, randomHex, fetch wrapper).
pub const UTIL_JS: &str = include_str!("js/util.js");

/// Cache wrapper (get, set, has, remove, getJSON, setJSON).
pub const CACHE_JS: &str = include_str!("js/cache.js");

/// File I/O wrapper (File class with static helpers).
pub const FILE_JS: &str = include_str!("js/file.js");

/// Date implementation (模拟 Web JS Date 对象).
pub const DATE_JS: &str = include_str!("js/date.js");

/// Inject all std scripts into the context.
///
/// Expects `runtime.core` to already be available on the global object.
pub fn inject_std<'js>(ctx: &Ctx<'js>) -> JsResult<()> {
    // Create `runtime.std` namespace
    ctx.eval::<(), _>(
        r#"
        if (typeof runtime === 'undefined') {
            globalThis.runtime = {};
        }
        if (!runtime.std) {
            runtime.std = {};
        }
    "#,
    )?;

    // Inject each standard module
    ctx.eval::<(), _>(CONSOLE_JS)?;
    ctx.eval::<(), _>(CRYPTO_JS)?;
    ctx.eval::<(), _>(ENCODING_JS)?;
    ctx.eval::<(), _>(PARAMS_JS)?;
    ctx.eval::<(), _>(UTIL_JS)?;
    ctx.eval::<(), _>(CACHE_JS)?;
    ctx.eval::<(), _>(FILE_JS)?;
    ctx.eval::<(), _>(DATE_JS)?;
    ctx.eval::<(), _>(IMPORT_JS)?;

    Ok(())
}
