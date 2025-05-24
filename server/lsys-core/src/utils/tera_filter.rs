use chrono::{TimeZone, Utc};
use std::collections::HashMap;
use tera::{Result, Value};

pub fn tera_second_format(value: &Value, args: &HashMap<String, Value>) -> Result<Value> {
    match value.as_u64() {
        Some(seconds) => {
            let y = args.get("y").and_then(|e| e.as_str()).unwrap_or_default();
            let m = args.get("m").and_then(|e| e.as_str()).unwrap_or_default();
            let d = args.get("d").and_then(|e| e.as_str()).unwrap_or_default();
            let h = args.get("h").and_then(|e| e.as_str()).unwrap_or_default();
            let i = args.get("i").and_then(|e| e.as_str()).unwrap_or_default();
            let s = args.get("s").and_then(|e| e.as_str()).unwrap_or_default();
            let year = seconds / 3600 / 24 / 365;
            let mon = seconds / 3600 / 24 / 12;
            let day = seconds / 3600 / 24;
            let hours = seconds / 3600;
            let remaining = seconds % 3600;
            let minutes = remaining / 60;
            let secs = remaining % 60;
            let mut parts = Vec::new();
            if year > 0 {
                parts.push(format!("{}{}", year, y));
            }
            if mon > 0 {
                parts.push(format!("{}{}", mon, m));
            }
            if day > 0 {
                parts.push(format!("{}{}", day, d));
            }
            if hours > 0 {
                parts.push(format!("{}{}", hours, h));
            }
            if minutes > 0 {
                parts.push(format!("{}{}", minutes, i));
            }
            if parts.is_empty() || secs > 0 {
                parts.push(format!("{}{}", secs, s));
            }
            Ok(Value::String(parts.join("")))
        }
        None => Ok(value.clone()),
    }
}

pub fn tera_time_format(value: &Value, _: &HashMap<String, Value>) -> Result<Value> {
    match value.as_u64() {
        Some(timestamp) => {
            let dt = Utc.timestamp_opt(timestamp as i64, 0).single();
            let formatted = dt.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string());
            Ok(Value::String(
                formatted.unwrap_or_else(|| format!("time:{}", timestamp)),
            ))
        }
        None => Ok(value.clone()),
    }
}
