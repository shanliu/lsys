use crate::{
    common::{JsonData, JsonResult, LimitParam, UserAuthQueryDao},
    dao::access::api::system::CheckAdminChangeLogsView,
};
use lsys_access::dao::AccessSession;
use serde::Deserialize;
use serde_json::json;
#[derive(Debug, Deserialize)]
pub struct ChangeLogsListParam {
    pub log_type: Option<String>,
    pub add_user_id: Option<u64>,
    pub count_num: Option<bool>,
    pub limit: Option<LimitParam>,
}

pub async fn change_logs_list(
    param: &ChangeLogsListParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &req_dao.req_env,
            Some(&auth_data),
            &CheckAdminChangeLogsView {},
        )
        .await?;
    let (res, next) = req_dao
        .web_dao
        .web_user
        .change_logger_dao
        .list_data(
            param.log_type.as_deref(),
            param.add_user_id,
            param.limit.as_ref().map(|e| e.into()).as_ref(),
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

    Ok(JsonData::data(json!({
        "data": res,
        "next": next,
        "total":count,
    })))
}
