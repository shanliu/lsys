//用户文件日志接口

use crate::common::{
    JsonData, JsonResponse, JsonResult, PageParam, RequestDao, ToOffsetPageParam, UserAuthQueryDao,
};
use crate::dao::WebDao;
use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::user::CheckUserFileView;
use lsys_access::dao::AccessSession;
use lsys_core::api_utils::JsonPageData;
use lsys_core::db::OffsetPageParam;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct FileLogsParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub file_ref_id: u64,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
    pub page: Option<PageParam>,
}

/// 文件日志列表
pub async fn file_logs(
    param: &FileLogsParam,
    auth_dao: &UserAuthQueryDao,
    web_dao: &WebDao,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    let auth_data = auth_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await?;

 
    let page: OffsetPageParam = param.page.to_offset_page_param();

    let data_dao = web_dao.web_file.file_dao.data_dao();

    let file_ref = web_dao
        .web_file.file_dao
        .cache()
        .find_file_ref_by_id(param.file_ref_id)
        .await?;

       web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserFileView {
                res_user_id: file_ref.user_id,
            },
        )
        .await?;

    let data = data_dao.list_logs_by_file_id(file_ref.file_id, &page).await?;

    let count = if param.count_num.unwrap_or(false) {
        Some(data_dao.count_logs_by_file_id(file_ref.file_id).await?)
    } else {
        None
    };

    // 批量获取用户信息
    let user_ids: Vec<u64> = data.iter().map(|item| item.user_id).collect();
    let user_data = web_dao
        .web_access
        .access_dao
        .user
        .cache()
        .find_users_by_ids(&user_ids)
        .await?;

    let items: Vec<serde_json::Value> = data
        .iter()
        .map(|item| {
            json!({
                "id": item.id,
                "file_chunk_id": item.file_chunk_id,
                "message": item.message,
                "user_data": user_data.get(item.user_id),
                "add_time": item.add_time,
            })
        })
        .collect();

    Ok(JsonResponse::data(JsonData::body(JsonPageData::total(
        items, count,
    ))))
}
