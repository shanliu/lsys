use crate::common::{JsonData, ToCursorPageParam};
use crate::common::{JsonResponse, JsonResult, LimitParam, UserAuthQueryDao};
use crate::dao::access::api::system::admin::CheckAdminChangeLogsView;
use crate::dao::access::RbacAccessCheckEnv;
use lsys_access::dao::AccessSession;
use lsys_core::db::CursorPageSort;
use serde::Deserialize;
use serde_json::json;
#[derive(Debug, Deserialize)]
pub struct ChangeLogsListParam {
    pub log_type: Option<String>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u64")]
    pub add_user_id: Option<u64>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
    pub limit: Option<LimitParam>,
}

pub async fn change_logs_list(
    param: &ChangeLogsListParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminChangeLogsView {},
        )
        .await?;
    let (res, next_data) = req_dao
        .web_dao
        .web_user
        .change_logger_dao
        .list_data(
            param.log_type.as_deref(),
            param.add_user_id,
            &param.limit.to_u64_cursor_page_param(CursorPageSort::Desc),
        )
        .await?;

    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .web_user
                .change_logger_dao
                .list_count(param.log_type.as_deref(), param.add_user_id)
                .await?,
        )
    } else {
        None
    };

    Ok(JsonResponse::data(JsonData::body(json!({
        "data":  bind_vec_user_info_from_req!(req_dao, res, add_user_id,false) ,
        "next_cursor": next_data.next_cursor,
        "prev_cursor": next_data.prev_cursor,
        "total":count,
    }))))
}
