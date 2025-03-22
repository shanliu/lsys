use crate::common::{LimitParam, UserAuthQueryDao};
use lsys_access::dao::AccessSession;

use serde::Deserialize;
use serde_json::json;

use crate::common::{JsonData, JsonResult};

#[derive(Debug, Deserialize)]
pub struct LoginHistoryParam {
    pub login_type: Option<String>,
    pub login_account: Option<String>,
    pub login_ip: Option<String>,
    pub is_login: Option<i8>,
    pub page: Option<LimitParam>,
}

pub async fn login_history(
    param: &LoginHistoryParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let (data, next) = req_dao
        .web_dao
        .web_user
        .user_dao
        .account_dao
        .account_login_hostory
        .history_data(
            Some(auth_data.account_id()?),
            param.login_account.as_deref(),
            param.is_login,
            param.login_type.as_deref(),
            param.login_ip.as_deref(),
            param.page.as_ref().map(|e| e.into()).as_ref(),
        )
        .await?;
    let total = req_dao
        .web_dao
        .web_user
        .user_dao
        .account_dao
        .account_login_hostory
        .history_count(
            Some(auth_data.account_id()?),
            param.login_account.as_deref(),
            param.is_login,
            param.login_type.as_deref(),
            param.login_ip.as_deref(),
        )
        .await?;
    Ok(JsonData::data(json!({
        "data": data ,
        "next": next,
        "total":total,
    })))
}
