use axum::{http::{HeaderMap, StatusCode}, response::Response};
use serde_json::json;
use std::sync::Arc;

use crate::auth::JwtAuthorizeRequest;
use crate::model::{BarcodeCreateStatus, BarcodeParseStatus};
use crate::server::AppState;
use crate::utils::handler::{barcode_type_json, create_status_json, parse_status_json};
use crate::utils::handler::{json_fluent_err, json_ok};
use lsys_core::FluentBundle;

pub(crate) async fn handle_mapping(
    state: &Arc<AppState>,
    fluent: &Arc<FluentBundle>,
    headers: &HeaderMap,
) -> Response {
    let authz = state
        .upstream
        .jwt_authorize(
            headers,
            &JwtAuthorizeRequest {
                action: "mapping".to_string(),
                app_id: None,
                res_user_id: None,
            },
        )
        .await;
    if let Err(err) = authz {
        return json_fluent_err(fluent, StatusCode::BAD_GATEWAY, "service", err);
    }
    json_ok(json!({
        "barcode_type":vec![
            barcode_type_json(fluent, "aztec"),
            barcode_type_json(fluent, "qrcode"),
            barcode_type_json(fluent, "datamatrix"),
        ],
        "create_status":vec![
            create_status_json(fluent, BarcodeCreateStatus::EnablePrivate),
            create_status_json(fluent, BarcodeCreateStatus::EnablePublic),
        ],
        "parse_status":vec![
            parse_status_json(fluent, BarcodeParseStatus::Succ),
            parse_status_json(fluent, BarcodeParseStatus::Fail),
        ],
    }))
}
