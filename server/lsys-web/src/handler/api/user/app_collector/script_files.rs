use crate::common::{JsonData, JsonResponse, JsonResult, RequestDao, UserAuthQueryDao};
use crate::dao::WebDao;
use crate::handler::api::user::app_collector::app_check_get;
use lsys_access::dao::AccessSession;
use lsys_core::api_utils::{JsonPageData, PageCursorValue, PageTotalRowValue};
use lsys_core::db::{CursorPageSort, TotalParam};
use serde::Deserialize;
use serde_json::json;
#[derive(Debug, Deserialize)]
pub struct ScriptFilesParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub script_id: u64,
    pub limit: Option<crate::common::LimitParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
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

pub async fn script_files(
    param: &ScriptFilesParam,
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

    use crate::common::ToCursorPageParam;
    let page = param.limit.to_u64_cursor_page_param(CursorPageSort::Desc);

    let script = web_dao
        .web_collector.collector
        .find_script_by_id(param.script_id)
        .await?
        .ok_or_else(|| {
            crate::common::JsonError::Message(lsys_core::fluent_message!(
                "collector-script-not-found"
            ))
        })?;

    let (files, page_data) = web_dao
        .web_collector.collector
        .list_script_files(&script, &page, Some(param.app_id))
        .await?;

    let items: Vec<serde_json::Value> = files
        .iter()
        .map(|item| {
            json!({
                "file_id": item.file_id,
                "file_name": item.file_name,
                "file_md5": item.file_md5,
                "file_size": item.file_size,
                "storage_type": item.storage_type,
                "content_type": item.content_type,
                "file_key": item.file_key,
                "add_time": item.add_time,
            })
        })
        .collect();

    let total = if param.count_num.unwrap_or(false) {
        Some(
            web_dao
                .web_collector.collector
                .count_script_files(&script, Some(param.app_id), &TotalParam::default())
                .await
                .map(PageTotalRowValue::from)?,
        )
    } else {
        None
    };

    let cursor = PageCursorValue::from(&page_data);
    Ok(JsonResponse::data(JsonData::body(JsonPageData::cursor(
        items, cursor, total,
    ))))
}
