use axum::{
    body::Body,
    http::{HeaderMap, Request, StatusCode},
    response::Response,
};
use base64::Engine;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

use crate::auth::RestAuthorizeResponse;
use crate::model::BarcodeCreateStatus;
use crate::server::AppState;
use crate::utils::handler::{json_err, json_fluent_err, json_ok};
use lsys_core::fluents::FluentBundle;

use super::RestGet;

#[derive(Debug, Deserialize)]
pub struct CodeParam {
    pub contents: String,
    pub code_id: u64,
}

pub(crate) async fn handle_create(
    state: &Arc<AppState>,
    fluent: &Arc<FluentBundle>,
    headers: &HeaderMap,
    raw_query: &str,
    req: Request<Body>,
) -> Response {
    let bytes = match axum::body::to_bytes(req.into_body(), 1024 * 1024).await {
        Ok(b) => b,
        Err(err) => return json_err(StatusCode::BAD_REQUEST, "body", err.to_string()),
    };
    let param: CodeParam = match serde_json::from_slice(&bytes) {
        Ok(p) => p,
        Err(_) => {
            // fallback to payload query
            match serde_urlencoded::from_str::<RestGet>(raw_query)
                .ok()
                .and_then(|r| r.payload)
                .and_then(|p| serde_json::from_str::<CodeParam>(&p).ok())
            {
                Some(p) => p,
                None => return json_err(StatusCode::BAD_REQUEST, "param", "missing code param"),
            }
        }
    };

    let authz: RestAuthorizeResponse = match state.upstream.rest_authorize(headers, raw_query).await
    {
        Ok(r) => r,
        Err(err) => return json_fluent_err(fluent, StatusCode::BAD_GATEWAY, "service", err),
    };

    let code = match state
        .barcode
        .cache()
        .dao
        .find_by_create_config_id(&param.code_id)
        .await
    {
        Ok(c) => c,
        Err(err) => return json_err(StatusCode::NOT_FOUND, "code", format!("{err:?}")),
    };

    if BarcodeCreateStatus::Delete.eq(code.status) {
        return json_err(StatusCode::NOT_FOUND, "code", "code deleted");
    }

    // upstream already checked feature via rest_authz; but guard app_id mismatch
    if code.app_id != authz.app_id {
        // keep compatible: forbid cross-app usage
        return json_err(StatusCode::FORBIDDEN, "app", "code.app_id mismatch");
    }

    let data = match state
        .barcode
        .barcode_show(&param.contents, &code, true)
        .await
    {
        Ok(d) => d,
        Err(err) => {
            return json_err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "render",
                format!("{err:?}"),
            );
        }
    };

    let base64 = base64::engine::general_purpose::STANDARD.encode(data.1);
    json_ok(json!({
        "data": base64,
        "type": data.0.to_mime_type(),
    }))
}
