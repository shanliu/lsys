use axum::{
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use lsys_core::api_utils::{JsonData, JsonResponse};
use lsys_core::fluents::{FluentBundle, FluentMessage, FluentMgr, IntoFluentMessage};
use lsys_core::utils::RequestEnv;
use serde_json::Value;
use std::sync::Arc;

// ---- response helpers ----

use serde_json::json;

use crate::model::{BarcodeCreateStatus, BarcodeParseStatus};

pub fn create_status_json(fluent: &FluentBundle, status: BarcodeCreateStatus) -> Value {
    json!({
        "key": status as i8,
        "val": fluent.format_message(&status.fluent()),
    })
}

pub fn parse_status_json(fluent: &FluentBundle, status: BarcodeParseStatus) -> Value {
    json!({
        "key": status as i8,
        "val": fluent.format_message(&status.fluent()),
    })
}

pub fn barcode_type_json(fluent: &FluentBundle, barcode_type: &str) -> Value {
    let msg = FluentMessage {
        id: format!("var-{}", barcode_type),
        crate_name: env!("CARGO_PKG_NAME").to_string(),
        data: vec![],
    };
    json!({
        "key": barcode_type,
        "val": fluent.format_message(&msg),
    })
}

pub fn json_ok(body: Value) -> Response {
    (
        StatusCode::OK,
        axum::Json(JsonResponse::data(JsonData::body(body)).to_value()),
    )
        .into_response()
}

pub fn json_err(status: StatusCode, sub_code: &str, msg: impl ToString) -> Response {
    (
        status,
        axum::Json(
            JsonResponse::data(JsonData::error().set_sub_code(sub_code))
                .set_message(msg)
                .to_value(),
        ),
    )
        .into_response()
}

pub fn json_fluent_err(
    fluent: &FluentBundle,
    status: StatusCode,
    sub_code: &str,
    err: impl IntoFluentMessage,
) -> Response {
    let msg = fluent.format_message(&err.to_fluent_message());
    json_err(status, sub_code, msg)
}

pub fn lang_from_headers(headers: &HeaderMap) -> Option<String> {
    headers
        .get(axum::http::header::ACCEPT_LANGUAGE)
        .and_then(|t| t.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s))
        .map(|s| s.replace('-', "_"))
}

pub fn get_fluent(fluent_mgr: &FluentMgr, headers: &HeaderMap) -> Arc<FluentBundle> {
    let lang = lang_from_headers(headers);
    fluent_mgr.locale(lang.as_deref())
}

pub fn env_from_headers(headers: &HeaderMap) -> Result<RequestEnv, String> {
    let lang = lang_from_headers(headers).unwrap_or_else(|| "zh_CN".to_string());

    let user_agent = headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|e| e.to_str().ok());
    let request_id = headers.get("X-Request-ID").and_then(|e| e.to_str().ok());
    let device_id = headers.get("X-Device-ID").and_then(|e| e.to_str().ok());

    // try first ip from x-forwarded-for
    let request_ip = headers
        .get("X-Forwarded-For")
        .and_then(|e| e.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty());

    RequestEnv::new(Some(&lang), request_ip, request_id, user_agent, device_id)
        .map_err(|e| e.to_fluent_message().default_format())
}
