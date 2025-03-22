use serde_json::json;

use crate::common::{JsonData, JsonResult};
use crate::{
    common::{LimitParam, UserAuthQueryDao},
    dao::access::api::system::CheckAdminUserManage,
};
use lsys_access::dao::AccessSession;

pub struct LoginHistoryParam {
    pub account_id: Option<u64>,
    pub login_type: Option<String>,
    pub login_account: Option<String>,
    pub login_ip: Option<String>,
    pub is_login: Option<i8>,
    pub count_num: Option<bool>,
    pub page: Option<LimitParam>,
}
pub async fn account_login_history(
    param: &LoginHistoryParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminUserManage {})
        .await?;
    let (res, next) = req_dao
        .web_dao
        .web_user
        .user_dao
        .account_dao
        .account_login_hostory
        .history_data(
            param.account_id,
            param.login_account.as_deref(),
            param.is_login,
            param.login_type.as_deref(),
            param.login_ip.as_deref(),
            param.page.as_ref().map(|e| e.into()).as_ref(),
        )
        .await?;
    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .web_user
                .user_dao
                .account_dao
                .account_login_hostory
                .history_count(
                    param.account_id,
                    None,
                    param.is_login,
                    param.login_type.as_deref(),
                    param.login_ip.as_deref(),
                )
                .await?,
        )
    } else {
        None
    };
    Ok(JsonData::data(json!({
        "data": res ,
        "next": next,
        "total":count,
    })))
}
