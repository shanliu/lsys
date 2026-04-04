use crate::common::{JsonData, ToCursorPageParam};
use crate::common::{LimitParam, UserAuthQueryDao};
use lsys_access::dao::AccessSession;
use lsys_core::api_utils::{JsonPageData, PageCursorValue, PageTotalRowValue};
use lsys_core::db::{CursorPageSort, TotalParam};
use serde::Deserialize;

use crate::common::{JsonResponse, JsonResult};

#[derive(Debug, Deserialize)]
pub struct LoginHistoryParam {
    pub login_type: Option<String>,
    pub login_account: Option<String>,
    pub login_ip: Option<String>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_i8")]
    pub is_login: Option<i8>,
    pub limit: Option<LimitParam>,
    pub count_num: Option<bool>,
}

pub async fn login_history(
    param: &LoginHistoryParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let (data, next_data) = req_dao
        .web_dao
        .web_user
        .user_dao
        .account_dao
        .account_login_history
        .history_data(
            Some(auth_data.account_id()?),
            param.login_account.as_deref(),
            param.is_login,
            param.login_type.as_deref(),
            param.login_ip.as_deref(),
            &param.limit.to_u64_cursor_page_param(CursorPageSort::Desc),
        )
        .await?;
    let total = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .web_user
                .user_dao
                .account_dao
                .account_login_history
                .history_count(
                    Some(auth_data.account_id()?),
                    param.login_account.as_deref(),
                    param.is_login,
                    param.login_type.as_deref(),
                    param.login_ip.as_deref(),
                    &TotalParam::default(),
                )
                .await
                .map(PageTotalRowValue::from)?,
        )
    } else {
        None
    };
    let cursor = PageCursorValue::from(&next_data);
    Ok(JsonResponse::data(JsonData::body(
        JsonPageData::cursor(data, cursor, total),
    )))
}
