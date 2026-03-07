// REST 采集接口 — 对外调用

use crate::common::{JsonData, JsonResponse, JsonResult, RequestDao};
use crate::dao::access::rest::CheckRestApp;
use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::collector::WebFileCollector;
use lsys_app::model::AppModel;
use lsys_core::db::CursorPageSort;
use serde::Deserialize;
use serde_json::json;

// ==================== 参数定义 ====================

#[derive(Debug, Deserialize)]
pub struct TriggerParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub script_id: u64,
    #[serde(default)]
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct StatusParam {
    pub request_id: String,
}

#[derive(Debug, Deserialize)]
pub struct RecordsParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub script_id: u64,
    #[serde(default)]
    pub request_id: Option<String>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_i8")]
    pub status: Option<i8>,
    pub limit: Option<crate::common::LimitParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct RecordFilesParam {
    pub request_id: String,
    pub limit: Option<crate::common::LimitParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct RecordLogsParam {
    pub request_id: String,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u8")]
    pub level: Option<u8>,
    pub page: Option<crate::common::PageParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
}

// ==================== 处理函数 ====================

/// POST /rest/collector/trigger — 触发采集任务
pub async fn trigger(
    param: &TriggerParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    let app_user = req_dao
        .web_dao
        .web_access
        .access_dao
        .user
        .cache()
        .find_by_id(&app.user_id)
        .await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::user(&app_user, &req_dao.req_env),
            &CheckRestApp {},
        )
        .await?;

    // 从 RequestEnv 提取 request_id，若不存在则自动生成
    let request_id = WebFileCollector::resolve_request_id(&req_dao.req_env);

    let params = param.params.clone().unwrap_or(serde_json::json!({}));

    let (record_id, task_id, script_name) = req_dao
        .web_dao
        .web_files
        .collector
        .submit_task(param.script_id, app.user_id, app.id, &request_id, &params)
        .await?;

    Ok(JsonResponse::data(JsonData::body(json!({
        "request_id": request_id,
        "record_id": record_id.to_string(),
        "task_id": task_id.to_string(),
        "script_name": script_name,
    }))))
}

/// GET /rest/collector/status — 查询状态（按 request_id）
pub async fn status(
    param: &StatusParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    let app_user = req_dao
        .web_dao
        .web_access
        .access_dao
        .user
        .cache()
        .find_by_id(&app.user_id)
        .await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::user(&app_user, &req_dao.req_env),
            &CheckRestApp {},
        )
        .await?;

    let record = req_dao
        .web_dao
        .web_files
        .collector
        .find_record_by_request_id(&param.request_id)
        .await?;

    let record = match record {
        Some(r) => r,
        None => {
            return Ok(JsonResponse::data(JsonData::body(json!({
                "found": false,
            }))));
        }
    };

    Ok(JsonResponse::data(JsonData::body(json!({
        "found": true,
        "record": {
            "id": record.id,
            "request_id": record.request_id,
            "script_id": record.script_id,
            "status": record.status,
            "elapsed_ms": record.elapsed_ms,
            "error_message": record.error_message,
            "add_time": record.add_time,
            "start_time": record.start_time,
            "finish_time": record.finish_time,
        },
    }))))
}

/// GET /rest/collector/records — 记录列表
pub async fn records(
    param: &RecordsParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    let app_user = req_dao
        .web_dao
        .web_access
        .access_dao
        .user
        .cache()
        .find_by_id(&app.user_id)
        .await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::user(&app_user, &req_dao.req_env),
            &CheckRestApp {},
        )
        .await?;

    use crate::common::ToCursorPageParam;
    let page = param.limit.to_u64_cursor_page_param(CursorPageSort::Desc);

    let (record_list, page_data) = req_dao
        .web_dao
        .web_files
        .collector
        .list_records(
            param.script_id,
            param.request_id.as_deref(),
            param.status,
            &page,
        )
        .await?;

    let record_items: Vec<serde_json::Value> = record_list
        .iter()
        .map(|rec| {
            json!({
                "id": rec.id,
                "request_id": rec.request_id,
                "script_id": rec.script_id,
                "status": rec.status,
                "elapsed_ms": rec.elapsed_ms,
                "error_message": rec.error_message,
                "exec_params": rec.exec_params,
                "add_time": rec.add_time,
                "start_time": rec.start_time,
                "finish_time": rec.finish_time,
            })
        })
        .collect();

    let total = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .web_files
                .collector
                .count_records(param.script_id, param.request_id.as_deref(), param.status)
                .await?,
        )
    } else {
        None
    };

    Ok(JsonResponse::data(JsonData::body(json!({
        "data": record_items,
        "next_cursor": page_data.next_cursor,
        "prev_cursor": page_data.prev_cursor,
        "total": total,
    }))))
}

/// GET /rest/collector/record_files — 记录关联文件列表
pub async fn record_files(
    param: &RecordFilesParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    let app_user = req_dao
        .web_dao
        .web_access
        .access_dao
        .user
        .cache()
        .find_by_id(&app.user_id)
        .await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::user(&app_user, &req_dao.req_env),
            &CheckRestApp {},
        )
        .await?;

    let record = req_dao
        .web_dao
        .web_files
        .collector
        .find_record_by_request_id(&param.request_id)
        .await?
        .ok_or_else(|| {
            crate::common::JsonError::Message(lsys_core::fluent_message!(
                "collector-record-not-found"
            ))
        })?;

    use crate::common::ToCursorPageParam;
    let page = param.limit.to_u64_cursor_page_param(CursorPageSort::Desc);

    let (files, page_data) = req_dao
        .web_dao
        .web_files
        .collector
        .list_record_files(&record, &page, Some(app.id))
        .await?;

    let items: Vec<serde_json::Value> = files
        .iter()
        .map(|item| {
            let tags: Vec<serde_json::Value> = item
                .tags
                .iter()
                .map(|tag| {
                    json!({
                        "tag_name": tag.tag_name,
                        "add_time": tag.add_time,
                    })
                })
                .collect();
            json!({
                "file_id": item.file_id,
                "file_name": item.file_name,
                "file_md5": item.file_md5,
                "file_size": item.file_size,
                "storage_type": item.storage_type,
                "content_type": item.content_type,
                "file_url": item.file_url,
                "add_time": item.add_time,
                "tags": tags,
            })
        })
        .collect();

    let total = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .web_files
                .collector
                .count_record_files(&record, Some(app.id))
                .await?,
        )
    } else {
        None
    };

    Ok(JsonResponse::data(JsonData::body(json!({
        "data": items,
        "next_cursor": page_data.next_cursor,
        "prev_cursor": page_data.prev_cursor,
        "total": total,
    }))))
}

/// GET /rest/collector/record_logs — 记录关联日志列表
pub async fn record_logs(
    param: &RecordLogsParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    let app_user = req_dao
        .web_dao
        .web_access
        .access_dao
        .user
        .cache()
        .find_by_id(&app.user_id)
        .await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::user(&app_user, &req_dao.req_env),
            &CheckRestApp {},
        )
        .await?;

    let record = req_dao
        .web_dao
        .web_files
        .collector
        .find_record_by_request_id(&param.request_id)
        .await?
        .ok_or_else(|| {
            crate::common::JsonError::Message(lsys_core::fluent_message!(
                "collector-record-not-found"
            ))
        })?;

    use crate::common::ToOffsetPageParam;
    let page = param.page.to_offset_page_param();

    let logs = req_dao
        .web_dao
        .web_files
        .collector
        .list_record_logs(&record, param.level, &page)
        .await?;

    let items: Vec<serde_json::Value> = logs
        .iter()
        .map(|log| {
            json!({
                "id": log.id,
                "request_id": log.request_id,
                "script_id": log.script_id,
                "user_id": log.user_id,
                "app_id": log.app_id,
                "level": log.level,
                "message": log.message,
                "add_time": log.add_time,
            })
        })
        .collect();

    let total = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .web_files
                .collector
                .count_record_logs(&record, param.level)
                .await?,
        )
    } else {
        None
    };

    Ok(JsonResponse::data(JsonData::body(json!({
        "data": items,
        "total": total,
    }))))
}
