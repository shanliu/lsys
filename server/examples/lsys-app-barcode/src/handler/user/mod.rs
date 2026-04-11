mod create_config;
mod mapping;
mod parse_record;

use axum::{
    Json,
    extract::{Path as AxumPath, State},
    http::{HeaderMap, StatusCode},
    response::Response,
};
use serde_json::Value;
use std::sync::Arc;

use crate::server::AppState;
use crate::utils::handler::{env_from_headers, get_fluent, json_err};

pub async fn user_app_barcode(
    State(state): State<Arc<AppState>>,
    AxumPath(op): AxumPath<String>,
    headers: HeaderMap,
    Json(payload): Json<Value>,
) -> Response {
    let fluent = get_fluent(&state.fluent, &headers);
    let env = match env_from_headers(&headers) {
        Ok(e) => e,
        Err(err) => return json_err(StatusCode::BAD_REQUEST, "env", err),
    };

    match op.as_str() {
        "mapping" => mapping::handle_mapping(&state, &fluent, &headers).await,
        "create_config_add" => {
            create_config::handle_create_config_add(&state, &fluent, &headers, &env, payload).await
        }
        "create_config_edit" => {
            create_config::handle_create_config_edit(&state, &fluent, &headers, &env, payload).await
        }
        "create_config_delete" => {
            create_config::handle_create_config_delete(&state, &fluent, &headers, &env, payload)
                .await
        }
        "create_config_list" => {
            create_config::handle_create_config_list(&state, &fluent, &headers, payload).await
        }
        "parse_record_list" => {
            parse_record::handle_parse_record_list(&state, &fluent, &headers, payload).await
        }
        "parse_record_delete" => {
            parse_record::handle_parse_record_delete(&state, &fluent, &headers, &env, payload).await
        }
        name => json_err(StatusCode::NOT_FOUND, "op", format!("not found:{name}")),
    }
}
