use serde_json::json;

use crate::{
    common::{LimitParam, UserAuthQueryDao},
    dao::access::api::system::CheckAdminUserManage,
};

use crate::common::{JsonData, JsonResult};

pub struct LoginHistoryParam {
    pub account_id: Option<u64>,
    pub login_type: Option<String>,
    pub login_account: Option<String>,
    pub login_ip: Option<String>,
    pub is_login: Option<i8>,
    pub count_num: Option<bool>,
    pub page: Option<LimitParam>,
}
pub async fn user_login_history(
    param: &LoginHistoryParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.access_env().await?, &CheckAdminUserManage {}, None)
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
