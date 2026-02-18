mod create;
mod parse;
mod utils;

use axum::{
    body::Body,
    extract::{FromRequest, Multipart, RawQuery, State},
    http::{Request, StatusCode},
    response::Response,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::server::AppState;
use crate::utils::handler::{get_fluent, json_err};

#[derive(Debug, Deserialize)]
pub(crate) struct RestGet {
    pub method: Option<String>,
    pub payload: Option<String>,
}

pub async fn rest_barcode(
    State(state): State<Arc<AppState>>,
    RawQuery(raw_query): RawQuery,
    req: Request<Body>,
) -> Response {
    let raw_query = raw_query.unwrap_or_default();
    let headers = req.headers().clone();
    let fluent = get_fluent(&state.fluent, &headers);

    let rfc = serde_urlencoded::from_str::<RestGet>(&raw_query).unwrap_or(RestGet {
        method: None,
        payload: None,
    });
    let method = rfc.method.clone().unwrap_or_default();

    match method.as_str() {
        "parse" => {
            let multipart = match Multipart::from_request(req, &state).await {
                Ok(m) => m,
                Err(err) => return json_err(StatusCode::BAD_REQUEST, "multipart", err.to_string()),
            };
            parse::handle_parse(&state, &fluent, &headers, &raw_query, &rfc, multipart).await
        }
        "create" => create::handle_create(&state, &fluent, &headers, &raw_query, req).await,
        _ => json_err(
            StatusCode::NOT_FOUND,
            "method",
            format!("method not found:{method}"),
        ),
    }
}
