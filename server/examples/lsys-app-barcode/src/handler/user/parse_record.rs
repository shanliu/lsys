use axum::{
    http::{HeaderMap, StatusCode},
    response::Response,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::auth::JwtAuthorizeRequest;
use crate::dao::BarcodeParseRecord;
use crate::handler::common::{PageParam, ToOffsetPageParam};
use crate::server::AppState;
use crate::utils::handler::{json_err, json_fluent_err, json_ok};
use lsys_core::{FluentBundle, RequestEnv};

#[derive(Debug, Deserialize)]
struct ParseRecordListParam {
    pub app_id: Option<u64>,
    pub barcode_type: Option<String>,
    pub count_num: Option<bool>,
    pub page: Option<PageParam>,
}

#[derive(Debug, Deserialize)]
struct ParseRecordDeleteParam {
    pub id: u64,
}

pub(crate) async fn handle_parse_record_list(
    state: &Arc<AppState>,
    fluent: &Arc<FluentBundle>,
    headers: &HeaderMap,
    payload: Value,
) -> Response {
    let param: ParseRecordListParam = match serde_json::from_value(payload) {
        Ok(p) => p,
        Err(err) => return json_err(StatusCode::BAD_REQUEST, "param", err.to_string()),
    };

    let authz = match state
        .upstream
        .jwt_authorize(
            headers,
            &JwtAuthorizeRequest {
                action: "parse_record_list".to_string(),
                app_id: None,
                res_user_id: None,
            },
        )
        .await
    {
        Ok(a) => a,
        Err(err) => return json_fluent_err(fluent, StatusCode::BAD_GATEWAY, "service", err),
    };

    let data = match state
        .barcode
        .list_parse_record(
            authz.user_id,
            param.app_id,
            param.barcode_type.as_deref(),
            &param.page.to_offset_page_param(),
        )
        .await
    {
        Ok(d) => d,
        Err(err) => return json_err(StatusCode::INTERNAL_SERVER_ERROR, "db", format!("{err:?}")),
    };

    let count = if param.count_num.unwrap_or(false) {
        match state
            .barcode
            .count_parse_record(authz.user_id, param.app_id, param.barcode_type.as_deref())
            .await
        {
            Ok(c) => Some(c),
            Err(err) => {
                return json_err(StatusCode::INTERNAL_SERVER_ERROR, "db", format!("{err:?}"))
            }
        }
    } else {
        None
    };

    let data = data
        .into_iter()
        .map(|tmp| match tmp {
            BarcodeParseRecord::Succ(t) => {
                json!({
                    "id":t.0.id,
                    "app_id":t.0.app_id,
                    "bar_type":t.0.barcode_type,
                    "status":1,
                    "text":t.1.text,
                    "error":"",
                    "hash":t.0.file_hash,
                    "create_time":t.0.create_time
                })
            }
            BarcodeParseRecord::Fail(t) => {
                json!({
                    "id":t.id,
                    "app_id":t.app_id,
                    "bar_type":t.barcode_type,
                    "text":"",
                    "status":0,
                    "error":t.record,
                    "hash":t.file_hash,
                    "create_time":t.create_time
                })
            }
        })
        .collect::<Vec<Value>>();

    json_ok(json!({"data": data, "total": count}))
}

pub(crate) async fn handle_parse_record_delete(
    state: &Arc<AppState>,
    fluent: &Arc<FluentBundle>,
    headers: &HeaderMap,
    env: &RequestEnv,
    payload: Value,
) -> Response {
    let param: ParseRecordDeleteParam = match serde_json::from_value(payload) {
        Ok(p) => p,
        Err(err) => return json_err(StatusCode::BAD_REQUEST, "param", err.to_string()),
    };

    let data = match state.barcode.find_by_parse_record_id(&param.id).await {
        Ok(d) => d,
        Err(err) => return json_err(StatusCode::NOT_FOUND, "record", format!("{err:?}")),
    };

    let authz = match state
        .upstream
        .jwt_authorize(
            headers,
            &JwtAuthorizeRequest {
                action: "parse_record_delete".to_string(),
                app_id: None,
                res_user_id: Some(data.user_id),
            },
        )
        .await
    {
        Ok(a) => a,
        Err(err) => return json_fluent_err(fluent, StatusCode::BAD_GATEWAY, "service", err),
    };

    if let Err(err) = state
        .barcode
        .delete_parse_record(authz.user_id, &data, Some(env))
        .await
    {
        return json_err(StatusCode::INTERNAL_SERVER_ERROR, "db", format!("{err:?}"));
    }
    json_ok(json!({}))
}
