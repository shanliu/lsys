// 系统级采集脚本管理接口（app_id=0，管理员操作）

use crate::common::{JsonData, JsonResponse, JsonResult, UserAuthQueryDao};
use crate::dao::access::api::system::admin::CheckAdminFileManage;
use crate::dao::access::RbacAccessCheckEnv;
use crate::model::{CollectorRecordStatus, CollectorScriptStatus};
use lsys_access::dao::AccessSession;
use lsys_core::db::CursorPageSort;
use serde::Deserialize;
use serde_json::json;

// ==================== 参数定义 ====================

#[derive(Debug, Deserialize)]
pub struct ScriptListParam {
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u64")]
    pub user_id: Option<u64>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_i8")]
    pub status: Option<i8>,
    pub page: Option<crate::common::PageParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ScriptAddParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub user_id: u64,
    pub name: String,
    pub script_code: String,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u32")]
    pub timeout_secs: Option<u32>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u64")]
    pub memory_limit: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct ScriptEditParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub script_id: u64,
    pub name: String,
    pub script_code: String,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u32")]
    pub timeout_secs: Option<u32>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u64")]
    pub memory_limit: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct ScriptStatusParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub script_id: u64,
    #[serde(deserialize_with = "crate::common::deserialize_i8")]
    pub status: i8,
}

#[derive(Debug, Deserialize)]
pub struct ScriptDeleteParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub script_id: u64,
}

#[derive(Debug, Deserialize)]
pub struct ScriptRecordsParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub script_id: u64,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_i8")]
    pub status: Option<i8>,
    pub limit: Option<crate::common::LimitParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ScriptFilesParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub script_id: u64,
    pub limit: Option<crate::common::LimitParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ScriptLogsParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub script_id: u64,
    #[serde(default)]
    pub request_id: Option<String>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u8")]
    pub level: Option<u8>,
    pub limit: Option<crate::common::LimitParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitTaskParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub script_id: u64,
    #[serde(default)]
    pub request_id: Option<String>,
    #[serde(default)]
    pub params: Option<serde_json::Value>,
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
} // ==================== 处理函数 ====================

/// POST /api/system/collector/mapping — 采集字典映射
pub async fn mapping_data(req_dao: &UserAuthQueryDao) -> JsonResult<JsonResponse> {
    Ok(JsonResponse::data(JsonData::body(json!({
        "script_status": vec![
            status_json_format!(req_dao, CollectorScriptStatus::Enable),
            status_json_format!(req_dao, CollectorScriptStatus::Disable),
        ],
        "record_status": vec![
            status_json_format!(req_dao, CollectorRecordStatus::Pending),
            status_json_format!(req_dao, CollectorRecordStatus::Running),
            status_json_format!(req_dao, CollectorRecordStatus::Success),
            status_json_format!(req_dao, CollectorRecordStatus::Failed),
            status_json_format!(req_dao, CollectorRecordStatus::Timeout),
        ],
    }))))
}

/// GET /api/system/collector/scripts — 系统脚本列表 (app_id=0)
pub async fn scripts(
    param: &ScriptListParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminFileManage {},
        )
        .await?;

    use crate::common::ToOffsetPageParam;
    let page = param.page.to_offset_page_param();

    let data = req_dao
        .web_dao
        .web_files
        .collector
        .list_scripts(0, param.status, &page)
        .await?;

    let total = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .web_files
                .collector
                .count_scripts(0, param.status)
                .await?,
        )
    } else {
        None
    };

    let items: Vec<serde_json::Value> = data
        .iter()
        .map(|s| {
            json!({
                "id": s.id,
                "user_id": s.user_id,
                "app_id": s.app_id,
                "name": s.name,
                "script_md5": s.script_md5,
                "timeout_secs": s.timeout_secs,
                "memory_limit": s.memory_limit,
                "status": s.status,
                "add_time": s.add_time,
                "change_time": s.change_time,
            })
        })
        .collect();

    Ok(JsonResponse::data(JsonData::body(json!({
        "data": items,
        "total": total,
    }))))
}

/// POST /api/system/collector/script_add — 创建系统脚本 (app_id=0)
pub async fn script_add(
    param: &ScriptAddParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminFileManage {},
        )
        .await?;

    let script_id = req_dao
        .web_dao
        .web_files
        .collector
        .script_add(
            param.user_id,
            0, // 系统级 app_id=0
            &param.name,
            &param.script_code,
            param.timeout_secs,
            param.memory_limit,
            Some(&req_dao.req_env),
        )
        .await?;

    Ok(JsonResponse::data(JsonData::body(json!({
        "id": script_id.to_string(),
    }))))
}

/// POST /api/system/collector/script_edit — 更新系统脚本
pub async fn script_edit(
    param: &ScriptEditParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminFileManage {},
        )
        .await?;

    let affected = req_dao
        .web_dao
        .web_files
        .collector
        .script_edit(
            param.script_id,
            &param.name,
            &param.script_code,
            param.timeout_secs,
            param.memory_limit,
            Some(&req_dao.req_env),
        )
        .await?;

    Ok(JsonResponse::data(JsonData::body(json!({
        "affected": affected,
    }))))
}

/// POST /api/system/collector/script_status — 启用/禁用脚本
pub async fn script_status(
    param: &ScriptStatusParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminFileManage {},
        )
        .await?;

    let status = match param.status {
        1 => CollectorScriptStatus::Enable,
        2 => CollectorScriptStatus::Disable,
        _ => {
            return Ok(JsonResponse::data(JsonData::error())
                .set_message("invalid status, must be 1 (enable) or 2 (disable)"));
        }
    };

    let affected = req_dao
        .web_dao
        .web_files
        .collector
        .script_change_status(param.script_id, status, Some(&req_dao.req_env))
        .await?;

    Ok(JsonResponse::data(JsonData::body(json!({
        "affected": affected,
    }))))
}

/// POST /api/system/collector/script_del — 删除系统脚本
pub async fn script_del(
    param: &ScriptDeleteParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminFileManage {},
        )
        .await?;

    let affected = req_dao
        .web_dao
        .web_files
        .collector
        .script_delete(param.script_id, Some(&req_dao.req_env))
        .await?;

    Ok(JsonResponse::data(JsonData::body(json!({
        "affected": affected,
    }))))
}

/// GET /api/system/collector/script_records — 按脚本查记录
pub async fn script_records(
    param: &ScriptRecordsParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminFileManage {},
        )
        .await?;

    use crate::common::ToCursorPageParam;
    let page = param.limit.to_u64_cursor_page_param(CursorPageSort::Desc);

    let (record_list, page_data) = req_dao
        .web_dao
        .web_files
        .collector
        .list_records(param.script_id, None, param.status, &page)
        .await?;

    let record_items: Vec<serde_json::Value> = record_list
        .iter()
        .map(|rec| {
            json!({
                "id": rec.id,
                "request_id": rec.request_id,
                "script_id": rec.script_id,
                "user_id": rec.user_id,
                "app_id": rec.app_id,
                "task_id": rec.task_id,
                "exec_params": rec.exec_params,
                "status": rec.status,
                "elapsed_ms": rec.elapsed_ms,
                "error_message": rec.error_message,
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
                .count_records(param.script_id, None, param.status)
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

/// GET /api/system/collector/script_files — 按脚本查全部文件
pub async fn script_files(
    param: &ScriptFilesParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminFileManage {},
        )
        .await?;

    use crate::common::ToCursorPageParam;
    let page = param.limit.to_u64_cursor_page_param(CursorPageSort::Desc);

    let script = req_dao
        .web_dao
        .web_files
        .collector
        .find_script_by_id(param.script_id)
        .await?
        .ok_or_else(|| {
            crate::common::JsonError::Message(lsys_core::fluent_message!(
                "collector-script-not-found"
            ))
        })?;

    let (files, page_data) = req_dao
        .web_dao
        .web_files
        .collector
        .list_script_files(&script, &page, None)
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
                .count_script_files(&script, None)
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

/// GET /api/system/collector/script_logs — 查询采集日志
pub async fn script_logs(
    param: &ScriptLogsParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminFileManage {},
        )
        .await?;

    use crate::common::ToCursorPageParam;
    let page = param.limit.to_u64_cursor_page_param(CursorPageSort::Desc);

    let (log_list, page_data) = req_dao
        .web_dao
        .web_files
        .collector
        .list_logs(
            param.script_id,
            param.request_id.as_deref(),
            param.level,
            &page,
        )
        .await?;

    let items: Vec<serde_json::Value> = log_list
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
                .count_logs(param.script_id, param.request_id.as_deref(), param.level)
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

/// POST /api/system/collector/submit_task — 提交采集任务（管理员测试脚本）
pub async fn submit_task(
    param: &SubmitTaskParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminFileManage {},
        )
        .await?;

    let user_id = auth_data.user_id();

    let request_id = match &param.request_id {
        Some(rid) if !rid.trim().is_empty() => rid.trim().to_string(),
        _ => crate::dao::collector::WebFileCollector::resolve_request_id(&req_dao.req_env),
    };

    let params = param.params.clone().unwrap_or(serde_json::json!({}));

    let (record_id, task_id, script_name) = req_dao
        .web_dao
        .web_files
        .collector
        .submit_task(param.script_id, user_id, 0, &request_id, &params)
        .await?;

    Ok(JsonResponse::data(JsonData::body(json!({
        "request_id": request_id,
        "record_id": record_id.to_string(),
        "task_id": task_id.to_string(),
        "script_name": script_name,
    }))))
}
/// GET /api/system/collector/record_files — 记录关联文件列表
pub async fn record_files(
    param: &RecordFilesParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminFileManage {},
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
        .list_record_files(&record, &page, None)
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
                .count_record_files(&record, None)
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

/// GET /api/system/collector/record_logs — 记录关联日志列表
pub async fn record_logs(
    param: &RecordLogsParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminFileManage {},
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
