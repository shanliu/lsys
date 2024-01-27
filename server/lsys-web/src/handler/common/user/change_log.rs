use crate::{
    dao::RequestAuthDao,
    handler::access::AccessAdminChangeLogsView,
    LimitParam, {JsonData, JsonResult},
};

use lsys_user::dao::auth::{SessionData, SessionTokenData, UserSession};
use serde::Deserialize;
use serde_json::json;
#[derive(Debug, Deserialize)]
pub struct ChangeLogsListParam {
    pub log_type: Option<String>,
    pub limit: Option<LimitParam>,
    pub user_id: Option<u64>,
    pub add_user_id: Option<u64>,
}

pub async fn change_logs_list<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: ChangeLogsListParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessAdminChangeLogsView {
                user_id: req_auth.user_data().user_id,
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let (res, next) = req_dao
        .web_dao
        .logger
        .list_data(
            &param.log_type,
            &param.user_id,
            &param.add_user_id,
            &Some(param.limit.unwrap_or_default().into()),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::data(json!({
        "data": res,
        "next": next
    })))
}
