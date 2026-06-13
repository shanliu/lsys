use crate::common::{JsonData, JsonResponse, JsonResult, RequestDao, UserAuthQueryDao};
use crate::dao::WebDao;
use crate::handler::api::user::app_collector::app_check_get;
use lsys_access::dao::AccessSession;
use lsys_core::api_utils::{PageCursorValue, PageTotalRowValue};
use lsys_core::db::{CursorPageSort, TotalParam};
use serde::Deserialize;
use serde_json::json;
#[derive(Debug, Deserialize)]
pub struct ScriptRecordsParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub script_id: u64,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_i8")]
    pub status: Option<i8>,
    pub limit: Option<crate::common::LimitParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
    /// 是否附加文件信息
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub attr_file: Option<bool>,
    /// 是否附加文件的 local 属性
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub attr_file_local: Option<bool>,
    /// 是否附加文件的 oss 属性
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub attr_file_oss: Option<bool>,
    /// 是否附加文件的 tag 属性
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub attr_file_tag: Option<bool>,
}

pub async fn script_records(
    param: &ScriptRecordsParam,
    req_dao: &RequestDao,
    auth_dao: &UserAuthQueryDao,
    web_dao: &WebDao,
) -> JsonResult<JsonResponse> {
    let auth_data = auth_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await?;
    app_check_get(param.app_id, false, &auth_data, req_dao, web_dao).await?;

    // 先查询脚本信息
    let script = web_dao
        .web_collector.collector
        .find_script_by_id(param.script_id)
        .await?
        .ok_or_else(|| {
            crate::common::JsonError::Message(
                lsys_core::fluent_message!("collector-script-not-found",
                    {"script_id": param.script_id}
                ),
            )
        })?;

    use crate::common::ToCursorPageParam;
    let page = param.limit.to_u64_cursor_page_param(CursorPageSort::Desc);

    let attr = lsys_file_manager::dao::collector::CollectorRecordListAttr {
        attr_file: param.attr_file,
        attr_file_local: param.attr_file_local,
        attr_file_oss: param.attr_file_oss,
        attr_file_tag: param.attr_file_tag,
    };

    let (record_list, page_data) = web_dao
        .web_collector.collector
        .list_records(&script, None, param.status, &page, &attr)
        .await?;

    let total = if param.count_num.unwrap_or(false) {
        Some(
            web_dao
                .web_collector.collector
                .count_records(&script, None, param.status, &TotalParam::default())
                .await
                .map(PageTotalRowValue::from)?,
        )
    } else {
        None
    };

    let record_items: Vec<serde_json::Value> = record_list
        .iter()
        .map(|item| {
            json!({
                "id": item.record.id,
                "request_id": item.record.request_id,
                "script_id": item.record.script_id,
                "user_id": item.record.add_user_id,
                "app_id": item.record.app_id,
                "task_id": item.record.task_id,
                "exec_params": item.record.exec_params,
                "status": item.record.status,
                "elapsed_ms": item.record.elapsed_ms,
                "error_message": item.record.error_message,
                "add_time": item.record.add_time,
                "start_time": item.record.start_time,
                "finish_time": item.record.finish_time,
                "file": item.file,
                "has_more_files": item.has_more_files,
            })
        })
        .collect();

    let cursor = PageCursorValue::from(&page_data);

    let mut response_body = json!({
        "data": record_items,
        "cursor": cursor,
    });

    if let Some(total) = total {
        response_body["total"] = json!(total);
    }

    // 添加属性参数到响应中
    if param.attr_file.is_some() {
        response_body["attr_file"] = json!(param.attr_file);
    }
    if param.attr_file_local.is_some() {
        response_body["attr_file_local"] = json!(param.attr_file_local);
    }
    if param.attr_file_oss.is_some() {
        response_body["attr_file_oss"] = json!(param.attr_file_oss);
    }
    if param.attr_file_tag.is_some() {
        response_body["attr_file_tag"] = json!(param.attr_file_tag);
    }

    Ok(JsonResponse::data(JsonData::body(response_body)))
}
