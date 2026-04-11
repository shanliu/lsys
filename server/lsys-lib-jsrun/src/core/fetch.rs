//! `core.fetch` – HTTP request capability.
//!
//! Rust controls concurrency, timeouts, and host allow-list.

use std::collections::HashSet;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

use rquickjs::{Ctx, FromJs, Function, Object, Result as JsResult, Value};
use tokio::sync::Semaphore;

use super::block_on_async;
use crate::runtime::RuntimeState;

/// Check if an IP address is private/internal.
fn is_private_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || v4.is_broadcast()
                || v4.is_unspecified()
        }
        IpAddr::V6(v6) => v6.is_loopback() || v6.is_unspecified(),
    }
}

/// Check whether the host is allowed by the whitelist and not an internal IP.
fn validate_host(
    url: &str,
    allow_list: &Option<HashSet<String>>,
    deny_private: bool,
) -> std::result::Result<(), String> {
    let parsed = reqwest::Url::parse(url).map_err(|e| format!("Invalid URL '{}': {}", url, e))?;
    let host = parsed
        .host_str()
        .ok_or_else(|| "URL has no host".to_string())?;

    // whitelist check
    if let Some(list) = allow_list
        && !list.contains(host)
    {
        return Err(format!("Host '{}' is not in the allow-list", host));
    }

    // private IP check
    if deny_private
        && let Ok(ip) = host.parse::<IpAddr>()
        && is_private_ip(&ip)
    {
        return Err(format!("Access to private IP '{}' is denied", ip));
    }

    Ok(())
}

/// Shared state for the fetch closure.
#[derive(Clone)]
struct FetchState {
    http_client: reqwest::Client,
    semaphore: Arc<Semaphore>,
    allow_list: Option<HashSet<String>>,
    deny_private: bool,
    timeout: Duration,
}

/// Register `core.fetch` on the given object.
///
/// `core.fetch(url)` or `core.fetch({url, method, headers, body})`
/// Returns a JS string with JSON: `{status, body}`.
///
/// For simplicity this is implemented as a **synchronous blocking** call that
/// bridges back into the tokio runtime.  This avoids the complex
/// `Persistent` + spawn dance needed for true async promises in rquickjs,
/// keeping the codebase approachable while still respecting concurrency limits.
pub fn register_fetch<'js>(
    ctx: &Ctx<'js>,
    core: &Object<'js>,
    state: &RuntimeState,
) -> JsResult<()> {
    let fs = FetchState {
        http_client: state.engine.http_client.clone(),
        semaphore: state.engine.fetch_semaphore.clone(),
        allow_list: state.config.host_allow_list.clone(),
        deny_private: state.config.deny_private_ip,
        timeout: state.config.fetch_timeout,
    };
    let handle = state.engine.tokio_handle.clone();

    core.set(
        "fetch",
        Function::new(
            ctx.clone(),
            move |ctx: Ctx<'js>, input: Value<'js>| -> JsResult<Value<'js>> {
                // Parse arguments: string or {url, method, headers, body}
                let (url, method, headers, body) = parse_fetch_args(&ctx, input)?;

                // Validate host
                if let Err(msg) = validate_host(&url, &fs.allow_list, fs.deny_private) {
                    return Err(rquickjs::Error::new_from_js_message("fetch", "string", msg));
                }

                let fs = fs.clone();
                let result = block_on_async(&handle, async move {
                    // Log concurrency state before acquiring
                    let total = fs.semaphore.available_permits();
                    tracing::info!(
                        target: "jsrun::fetch",
                        available_permits = total,
                        url = %url,
                        "fetch: acquiring concurrency permit"
                    );

                    // Acquire concurrency permit
                    let _permit = fs.semaphore.acquire().await.map_err(|e| e.to_string())?;

                    tracing::info!(
                        target: "jsrun::fetch",
                        remaining_permits = fs.semaphore.available_permits(),
                        url = %url,
                        method = %method,
                        "fetch: permit acquired, sending request"
                    );

                    let mut req = match method.to_uppercase().as_str() {
                        "POST" => fs.http_client.post(&url),
                        "PUT" => fs.http_client.put(&url),
                        "DELETE" => fs.http_client.delete(&url),
                        "PATCH" => fs.http_client.patch(&url),
                        "HEAD" => fs.http_client.head(&url),
                        _ => fs.http_client.get(&url),
                    };

                    for (k, v) in &headers {
                        req = req.header(k.as_str(), v.as_str());
                    }
                    if let Some(b) = &body {
                        req = req.body(b.clone());
                    }
                    req = req.timeout(fs.timeout);

                    let resp = req.send().await.map_err(|e| e.to_string())?;
                    let status = resp.status().as_u16();
                    let text = resp.text().await.map_err(|e| e.to_string())?;

                    tracing::info!(
                        target: "jsrun::fetch",
                        status = status,
                        url = %url,
                        body_len = text.len(),
                        "fetch: response received, releasing permit"
                    );

                    Ok::<(u16, String), String>((status, text))
                });

                match result {
                    Ok((status, text)) => {
                        let obj = Object::new(ctx.clone())?;
                        obj.set("status", status as u32)?;
                        obj.set("body", text)?;
                        Ok(obj.into_value())
                    }
                    Err(msg) => Err(rquickjs::Error::new_from_js_message("fetch", "object", msg)),
                }
            },
        )?,
    )?;

    Ok(())
}

/// Type alias for fetch arguments: (url, method, headers, body).
type FetchArgs = (String, String, Vec<(String, String)>, Option<String>);

/// Parse fetch arguments from JS value.
fn parse_fetch_args<'js>(ctx: &Ctx<'js>, value: Value<'js>) -> JsResult<FetchArgs> {
    // If it's a string, treat as URL with GET
    if let Ok(url) = String::from_js(ctx, value.clone()) {
        return Ok((url, "GET".into(), vec![], None));
    }

    // Otherwise expect an object {url, method?, headers?, body?}
    let obj = Object::from_js(ctx, value).map_err(|_| {
        rquickjs::Error::new_from_js_message("fetch", "object", "expected string or object")
    })?;

    let url: String = obj.get("url")?;
    let method: String = obj
        .get::<_, Option<String>>("method")?
        .unwrap_or_else(|| "GET".into());
    let body: Option<String> = obj.get("body")?;

    let mut headers = Vec::new();
    if let Ok(Some(hdr_obj)) = obj.get::<_, Option<Object<'js>>>("headers") {
        for k in hdr_obj.keys::<String>().flatten() {
            if let Ok(v) = hdr_obj.get::<_, String>(&k as &str) {
                headers.push((k, v));
            }
        }
    }

    Ok((url, method, headers, body))
}
