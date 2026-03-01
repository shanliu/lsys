use axum::{
    http::{HeaderMap, StatusCode},
    response::Response,
};
use lsys_core::fluent_message;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::auth::JwtAuthorizeRequest;
use crate::handler::common::{PageParam, ToOffsetPageParam};
use crate::model::BarcodeCreateStatus;
use crate::server::AppState;
use crate::utils::handler::{json_err, json_fluent_err, json_ok};
use lsys_core::fluents::FluentBundle;
use lsys_core::utils::RequestEnv;

#[derive(Debug, Deserialize)]
struct CreateConfigAddParam {
    pub app_id: u64,
    pub barcode_type: String,
    pub status: i8,
    pub image_format: String,
    pub image_width: i32,
    pub image_height: i32,
    pub margin: i32,
    pub image_color: String,
    pub image_background: String,
}

#[derive(Debug, Deserialize)]
struct CreateConfigEditParam {
    pub id: u64,
    pub barcode_type: String,
    pub status: i8,
    pub image_format: String,
    pub image_width: i32,
    pub image_height: i32,
    pub margin: i32,
    pub image_color: String,
    pub image_background: String,
}

#[derive(Debug, Deserialize)]
struct CreateConfigDeleteParam {
    pub id: u64,
}

#[derive(Debug, Deserialize)]
struct CreateConfigListParam {
    pub id: Option<u64>,
    pub app_id: Option<u64>,
    pub barcode_type: Option<String>,
    pub count_num: Option<bool>,
    pub page: Option<PageParam>,
}

pub(crate) async fn handle_create_config_add(
    state: &Arc<AppState>,
    fluent: &Arc<FluentBundle>,
    headers: &HeaderMap,
    env: &RequestEnv,
    payload: Value,
) -> Response {
    let param: CreateConfigAddParam = match serde_json::from_value(payload) {
        Ok(p) => p,
        Err(err) => return json_err(StatusCode::BAD_REQUEST, "param", err.to_string()),
    };

    let authz = match state
        .upstream
        .jwt_authorize(
            headers,
            &JwtAuthorizeRequest {
                action: "create_config_add".to_string(),
                app_id: Some(param.app_id),
                res_user_id: None,
            },
        )
        .await
    {
        Ok(a) => a,
        Err(err) => return json_fluent_err(fluent, StatusCode::BAD_GATEWAY, "service", err),
    };

    let status = match BarcodeCreateStatus::try_from(param.status) {
        Ok(s) => s,
        Err(err) => {
            return json_err(
                StatusCode::BAD_REQUEST,
                "status",
                fluent_message!("barcode-add-status-error", err).default_format(),
            )
        }
    };

    let id = match state
        .barcode
        .add_create_config(
            authz.user_id,
            param.app_id,
            &status,
            &param.barcode_type,
            &param.image_format,
            param.image_width,
            param.image_height,
            param.margin,
            &param.image_color,
            &param.image_background,
            Some(env),
        )
        .await
    {
        Ok(id) => id,
        Err(err) => return json_err(StatusCode::INTERNAL_SERVER_ERROR, "db", format!("{err:?}")),
    };

    json_ok(json!({"id": id}))
}

pub(crate) async fn handle_create_config_edit(
    state: &Arc<AppState>,
    fluent: &Arc<FluentBundle>,
    headers: &HeaderMap,
    env: &RequestEnv,
    payload: Value,
) -> Response {
    let param: CreateConfigEditParam = match serde_json::from_value(payload) {
        Ok(p) => p,
        Err(err) => return json_err(StatusCode::BAD_REQUEST, "param", err.to_string()),
    };

    let data = match state.barcode.find_by_create_config_id(&param.id).await {
        Ok(d) => d,
        Err(err) => return json_err(StatusCode::NOT_FOUND, "code", format!("{err:?}")),
    };

    let authz = match state
        .upstream
        .jwt_authorize(
            headers,
            &JwtAuthorizeRequest {
                action: "create_config_edit".to_string(),
                app_id: Some(data.app_id),
                res_user_id: None,
            },
        )
        .await
    {
        Ok(a) => a,
        Err(err) => return json_fluent_err(fluent, StatusCode::BAD_GATEWAY, "service", err),
    };

    let status = match BarcodeCreateStatus::try_from(param.status) {
        Ok(s) => s,
        Err(err) => {
            return json_err(
                StatusCode::BAD_REQUEST,
                "status",
                fluent_message!("barcode-add-status-error", err).default_format(),
            )
        }
    };

    if let Err(err) = state
        .barcode
        .edit_create_config(
            &data,
            authz.user_id,
            &status,
            &param.barcode_type,
            &param.image_format,
            param.image_width,
            param.image_height,
            param.margin,
            &param.image_color,
            &param.image_background,
            Some(env),
        )
        .await
    {
        return json_err(StatusCode::INTERNAL_SERVER_ERROR, "db", format!("{err:?}"));
    }
    json_ok(json!({}))
}

pub(crate) async fn handle_create_config_delete(
    state: &Arc<AppState>,
    fluent: &Arc<FluentBundle>,
    headers: &HeaderMap,
    env: &RequestEnv,
    payload: Value,
) -> Response {
    let param: CreateConfigDeleteParam = match serde_json::from_value(payload) {
        Ok(p) => p,
        Err(err) => return json_err(StatusCode::BAD_REQUEST, "param", err.to_string()),
    };

    let data = match state.barcode.find_by_create_config_id(&param.id).await {
        Ok(d) => d,
        Err(err) => return json_err(StatusCode::NOT_FOUND, "code", format!("{err:?}")),
    };

    let authz = match state
        .upstream
        .jwt_authorize(
            headers,
            &JwtAuthorizeRequest {
                action: "create_config_delete".to_string(),
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
        .delete_create_config(authz.user_id, &data, Some(env))
        .await
    {
        return json_err(StatusCode::INTERNAL_SERVER_ERROR, "db", format!("{err:?}"));
    }
    json_ok(json!({}))
}

pub(crate) async fn handle_create_config_list(
    state: &Arc<AppState>,
    fluent: &Arc<FluentBundle>,
    headers: &HeaderMap,
    payload: Value,
) -> Response {
    let param: CreateConfigListParam = match serde_json::from_value(payload) {
        Ok(p) => p,
        Err(err) => return json_err(StatusCode::BAD_REQUEST, "param", err.to_string()),
    };

    let authz = match state
        .upstream
        .jwt_authorize(
            headers,
            &JwtAuthorizeRequest {
                action: "create_config_list".to_string(),
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
        .list_create_config(
            authz.user_id,
            param.id,
            param.app_id,
            param.barcode_type.as_deref(),
            &param.page.to_offset_page_param(),
        )
        .await
    {
        Ok(d) => d,
        Err(err) => return json_err(StatusCode::INTERNAL_SERVER_ERROR, "db", format!("{err:?}")),
    };

    let data = data
        .into_iter()
        .map(|e| {
            json!({
                "id":e.id,
                "barcode_type":e.barcode_type,
                "app_id":e.app_id,
                "change_time":e.change_time,
                "image_background":e.image_background,
                "image_color":e.image_color,
                "image_format":e.image_format,
                "image_height":e.image_height,
                "image_width":e.image_width,
                "margin":e.margin,
                "status":e.status,
                "url":"",
            })
        })
        .collect::<Vec<_>>();

    let count = if param.count_num.unwrap_or(false) {
        match state
            .barcode
            .count_create_config(
                authz.user_id,
                param.id,
                param.app_id,
                param.barcode_type.as_deref(),
            )
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

    json_ok(json!({"data": data, "total": count}))
}
