use axum::{
    extract::Multipart,
    http::{HeaderMap, StatusCode},
    response::Response,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::auth::RestAuthorizeResponse;
use crate::dao::{BarcodeParseRecord, ParseParam as BarcodeParseParam};
use crate::server::AppState;
use crate::utils::handler::{env_from_headers, json_err, json_fluent_err, json_ok};
use lsys_core::FluentBundle;

use super::utils::upload_field;
use super::RestGet;

#[derive(Debug, Deserialize)]
pub struct ParseParam {
    #[serde(default)]
    pub try_harder: Option<bool>,
    #[serde(default)]
    pub decode_multi: Option<bool>,
    pub barcode_types: Option<Vec<String>>,
    pub other: Option<String>,
    #[serde(default)]
    pub pure_barcode: Option<bool>,
    pub character_set: Option<String>,
    pub allowed_lengths: Option<Vec<u32>>,
    #[serde(default)]
    pub assume_code_39_check_digit: Option<bool>,
    #[serde(default)]
    pub assume_gs1: Option<bool>,
    #[serde(default)]
    pub return_codabar_start_end: Option<bool>,
    pub allowed_ean_extensions: Option<Vec<u32>>,
    #[serde(default)]
    pub also_inverted: Option<bool>,
}

pub(crate) async fn handle_parse(
    state: &Arc<AppState>,
    fluent: &Arc<FluentBundle>,
    headers: &HeaderMap,
    raw_query: &str,
    rfc: &RestGet,
    mut multipart: Multipart,
) -> Response {
    let parse_param = match &rfc.payload {
        Some(payload) if !payload.is_empty() => {
            match serde_json::from_str::<Value>(payload)
                .ok()
                .and_then(|v| serde_json::from_value::<ParseParam>(v).ok())
            {
                Some(p) => p,
                None => return json_err(StatusCode::BAD_REQUEST, "param", "bad payload"),
            }
        }
        _ => ParseParam {
            try_harder: None,
            decode_multi: None,
            barcode_types: None,
            other: None,
            pure_barcode: None,
            character_set: None,
            allowed_lengths: None,
            assume_code_39_check_digit: None,
            assume_gs1: None,
            return_codabar_start_end: None,
            allowed_ean_extensions: None,
            also_inverted: None,
        },
    };

    let authz: RestAuthorizeResponse = match state.upstream.rest_authorize(headers, raw_query).await
    {
        Ok(r) => r,
        Err(err) => return json_fluent_err(fluent, StatusCode::BAD_GATEWAY, "service", err),
    };

    let env = match env_from_headers(headers) {
        Ok(e) => e,
        Err(err) => return json_err(StatusCode::BAD_REQUEST, "env", err),
    };

    let mut out = vec![];
    while let Some(field) = match multipart.next_field().await {
        Ok(v) => v,
        Err(err) => return json_err(StatusCode::BAD_REQUEST, "multipart", err.to_string()),
    } {
        match upload_field(field).await {
            Ok((_tmp_dir, file_path, ext)) => {
                let res = state
                    .barcode
                    .parse(
                        authz.app_user_id,
                        authz.app_id,
                        &file_path,
                        &ext,
                        &BarcodeParseParam {
                            try_harder: parse_param.try_harder,
                            decode_multi: parse_param.decode_multi,
                            barcode_types: parse_param
                                .barcode_types
                                .as_ref()
                                .map(|e| e.iter().map(|e| e.as_str()).collect::<Vec<_>>()),
                            other: parse_param.other.as_deref(),
                            pure_barcode: parse_param.pure_barcode,
                            character_set: parse_param.character_set.as_deref(),
                            allowed_lengths: parse_param.allowed_lengths.as_deref(),
                            assume_code_39_check_digit: parse_param.assume_code_39_check_digit,
                            assume_gs1: parse_param.assume_gs1,
                            return_codabar_start_end: parse_param.return_codabar_start_end,
                            allowed_ean_extensions: parse_param.allowed_ean_extensions.as_deref(),
                            also_inverted: parse_param.also_inverted,
                        },
                        Some(&env),
                    )
                    .await;

                match res {
                    Ok(tmp) => match tmp {
                        BarcodeParseRecord::Succ((t, record)) => {
                            out.push(json!({
                                "status":"1",
                                "data":{
                                    "type":t.barcode_type,
                                    "text":record.text,
                                    "position":record.position,
                                    "hash":t.file_hash,
                                }
                            }));
                        }
                        BarcodeParseRecord::Fail(t) => {
                            out.push(json!({
                                "status":"0",
                                "msg": format!("barcode-parse-error:{}", t.record),
                            }));
                        }
                    },
                    Err(err) => {
                        out.push(json!({
                            "status":"0",
                            "msg": format!("{err:?}"),
                        }));
                    }
                }
            }
            Err(err) => {
                out.push(json!({
                    "status":"0",
                    "msg":err,
                }));
            }
        }
    }

    json_ok(json!({"record": out}))
}
