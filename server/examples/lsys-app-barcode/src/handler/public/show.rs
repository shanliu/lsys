use axum::{
    extract::{Path as AxumPath, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
};
use base64::Engine;
use std::sync::Arc;

use crate::model::BarcodeCreateStatus;
use crate::server::AppState;
use crate::utils::handler::{get_fluent, json_err, json_fluent_err};

pub async fn public_show(
    State(state): State<Arc<AppState>>,
    AxumPath((content_type, code_id, content_data)): AxumPath<(String, u64, String)>,
    headers: HeaderMap,
) -> Response {
    let fluent = get_fluent(&state.fluent, &headers);
    let data = match content_type.as_str() {
        "base64" => match base64::engine::general_purpose::STANDARD.decode(&content_data) {
            Ok(v) => String::from_utf8_lossy(&v).to_string(),
            Err(err) => return json_err(StatusCode::BAD_REQUEST, "base64", err.to_string()),
        },
        "text" => content_data,
        _ => return json_err(StatusCode::BAD_REQUEST, "type", "show-barcode-bad-type"),
    };

    let code = match state
        .barcode
        .cache()
        .find_by_create_config_id(&code_id)
        .await
    {
        Ok(c) => c,
        Err(err) => return json_err(StatusCode::NOT_FOUND, "code", format!("{err:?}")),
    };

    // ask upstream if barcode feature is enabled for the app
    match state.upstream.app_feature_barcode(code.app_id).await {
        Ok(feature) => {
            if !feature.enabled {
                return json_err(
                    StatusCode::FORBIDDEN,
                    "feature",
                    "barcode feature is disabled for this app",
                );
            }
        }
        Err(err) => return json_fluent_err(&fluent, StatusCode::BAD_GATEWAY, "service", err),
    };

    if !BarcodeCreateStatus::EnablePublic.eq(code.status) {
        return json_err(StatusCode::FORBIDDEN, "auth", "barcode-bad-auth-error");
    }

    match state.barcode.barcode_show(&data, &code, true).await {
        Ok(img) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, img.0.to_mime_type().to_string())],
            img.1,
        )
            .into_response(),
        Err(err) => json_err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "render",
            format!("{err:?}"),
        ),
    }
}
