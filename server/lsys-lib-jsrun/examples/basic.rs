//! Basic example showing how to create a JsEngine and spawn
//! multiple isolated JsRuntime instances that share cache and fetch resources.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use lsys_lib_jsrun::{
    EngineConfig, JsEngine, LogHandler, MessageHandler, RuntimeConfig, LOG_LEVEL_DEBUG,
    LOG_LEVEL_ERROR, LOG_LEVEL_INFO, LOG_LEVEL_TRACE, LOG_LEVEL_WARN, MESSAGE_TYPE_GET_ENV,
    MESSAGE_TYPE_GET_PARAM,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialise tracing so we can see core.log / fetch logs
    tracing_subscriber::fmt::init();

    // ── Prepare a message handler (simulates task parameter storage) ──
    let params: Arc<Mutex<HashMap<String, serde_json::Value>>> = Arc::new(Mutex::new({
        let mut m = HashMap::new();
        m.insert(
            "greeting".into(),
            serde_json::Value::String("你好世界".into()),
        );
        m
    }));

    let message_handler: MessageHandler = {
        let params = params.clone();
        Arc::new(
            move |namespace: Option<String>, msg_type: String, data: serde_json::Value| {
                let params = params.clone();
                Box::pin(async move {
                    if let Some(ref ns) = namespace {
                        println!("  [message from namespace: {}]", ns);
                    }
                    match msg_type.as_str() {
                        // Well-known type: get_param - 返回整个 params 对象
                        MESSAGE_TYPE_GET_PARAM => {
                            let store = params.lock().unwrap();
                            let obj: serde_json::Map<String, serde_json::Value> = store
                                .iter()
                                .map(|(k, v)| (k.clone(), v.clone()))
                                .collect();
                            serde_json::Value::Object(obj)
                        }
                        // Well-known type: get_env
                        MESSAGE_TYPE_GET_ENV => {
                            let name = data
                                .get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or_default();
                            match std::env::var(name) {
                                Ok(val) => serde_json::json!({ "found": true, "value": val }),
                                Err(_) => serde_json::json!({ "found": false }),
                            }
                        }
                        // Custom: set_param
                        "set_param" => {
                            let name = data
                                .get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or_default()
                                .to_string();
                            let value = data.get("value").cloned().unwrap_or_default();
                            params.lock().unwrap().insert(name, value);
                            serde_json::json!({ "ok": true })
                        }
                        // Unknown: echo back
                        _ => data,
                    }
                })
            },
        )
    };

    // ── Prepare a log handler (demonstrates async log routing) ──
    let log_handler: LogHandler = Arc::new(|namespace, level, msg| {
        Box::pin(async move {
            let level_str = match level {
                LOG_LEVEL_TRACE => "TRACE",
                LOG_LEVEL_DEBUG => "DEBUG",
                LOG_LEVEL_INFO => "INFO",
                LOG_LEVEL_WARN => "WARN",
                LOG_LEVEL_ERROR => "ERROR",
                _ => "UNKNOWN",
            };
            println!("  📝 [{}] [ns={:?}] {}", level_str, namespace, msg);
        })
    });

    // ── 1. Create the engine ────────────────────────────────
    let engine_config = EngineConfig {
        max_runtimes: 4,
        ..Default::default()
    };
    let engine = JsEngine::new(engine_config)?;

    println!("Available runtime slots: {}", engine.available_runtimes());

    // ── 2. Create first runtime ─────────────────────────────
    let rt_config = RuntimeConfig {
        message_handler: Some(message_handler),
        log_handler: Some(log_handler),
        deny_private_ip: false, // allow localhost for demo
        ..Default::default()
    };
    let rt = engine.create_runtime(rt_config).await?;

    println!(
        "Available runtime slots after rt1: {}",
        engine.available_runtimes()
    );

    // ── 3. Simple expression ────────────────────────────────
    let result = rt.eval("1 + 2").await?;
    println!("1 + 2 = {}", result);

    // ── 4. Use console (std layer) ──────────────────────────
    rt.eval(r#"runtime.std.console.log("Hello from JavaScript!")"#)
        .await?;

    // ── 5. Use core.localTime() ────────────────────────────
    let ts = rt.eval("runtime.core.localTime().now").await?;
    println!("Current timestamp: {}", ts);

    // ── 6. Use crypto helpers (Web Crypto API style) ─────
    let md5 = rt.eval(r#"runtime.std.crypto.md5("hello world")"#).await?;
    println!("runtime.std.crypto.md5('hello world') = {}", md5);

    let sha = rt
        .eval(r#"runtime.std.crypto.sha256("hello world")"#)
        .await?;
    println!("runtime.std.crypto.sha256('hello world') = {}", sha);

    // ── 7. core.message.postMessage – direct call (echo) ────
    let echo = rt
        .eval(r#"JSON.stringify(runtime.core.message.postMessage("echo", { hello: "world" }))"#)
        .await?;
    println!("core.message echo: {}", echo);

    // ── 8. std.getParams – reads entire params object from Rust-side ───────
    let params_obj = rt
        .eval(r#"JSON.stringify(runtime.std.getParams())"#)
        .await?;
    println!("Params from task: {}", params_obj);

    let greeting = rt
        .eval(r#"runtime.std.getParams().greeting || "default""#)
        .await?;
    println!("Greeting from task params: {}", greeting);

    // ── 9. core.message – set a param from JS side ──────────
    rt.eval(r#"runtime.core.message.postMessage("set_param", { name: "color", value: "blue" })"#)
        .await?;
    // verify in Rust
    let color = params.lock().unwrap().get("color").cloned();
    println!("Rust sees color = {:?}", color);

    // ── 10. Cache ───────────────────────────────────────────
    rt.eval(r#"runtime.core.Cache.set("key1", "value1", 300000)"#)
        .await?;
    let cached = rt.eval(r#"runtime.core.Cache.get("key1")"#).await?;
    println!("Cached value: {}", cached);

    // ── 11. runtime.std.fetch (Web Fetch API style) ──────
    let fetch_demo = rt
        .eval(
            r#"var resp = runtime.std.fetch("https://httpbin.org/get");
            JSON.stringify({ status: resp.status, ok: resp.ok })"#,
        )
        .await?;
    println!("fetch result: {}", fetch_demo);

    // ── 13. Create a SECOND runtime – demonstrates isolation + sharing ──
    let rt2_config = RuntimeConfig {
        deny_private_ip: false,
        ..Default::default()
    };
    let rt2 = engine.create_runtime(rt2_config).await?;

    println!(
        "\nAvailable runtime slots after rt2: {}",
        engine.available_runtimes()
    );

    let result2 = rt2.eval("100 + 200").await?;
    println!("Runtime 2: 100 + 200 = {}", result2);

    // Cache is SHARED between runtimes
    let from_rt2 = rt2.eval(r#"runtime.core.Cache.get("key1")"#).await?;
    println!(
        "Runtime 2 reads key1 set by Runtime 1 from shared cache: {}",
        from_rt2
    );

    // But JS state is ISOLATED
    let isolated = rt2.eval("typeof resp").await?;
    println!("Runtime 2 cannot see rt1's `resp` variable: {}", isolated);

    // ── 14. Drop rt2 – frees a concurrency slot ─────────────
    drop(rt2);
    println!(
        "\nAvailable runtime slots after dropping rt2: {}",
        engine.available_runtimes()
    );

    println!("\n✅ All demos completed successfully!");
    Ok(())
}
