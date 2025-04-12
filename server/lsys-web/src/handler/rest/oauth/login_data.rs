use crate::common::JsonData;
use crate::{
    common::{JsonResponse, JsonResult, RestAuthQueryDao},
    dao::AccountOptionData,
};
use lsys_access::dao::AccessSession;
use lsys_user::model::{AccountEmailStatus, AccountMobileStatus};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct AccountOptionDataParam {
    pub auth: Option<bool>,
    pub user: Option<bool>,
    pub name: Option<bool>,
    pub info: Option<bool>,
    pub address: Option<bool>,
    pub email: Option<bool>,
    pub mobile: Option<bool>,
}

pub async fn account_data_from_oauth(
    param: &AccountOptionDataParam,
    req_dao: &RestAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    let account_id = auth_data.account_id()?;
    let mut check_scope = vec![];
    if param.info.unwrap_or(false) || param.name.unwrap_or(false) {
        check_scope.push("user_info");
    }
    if param.address.unwrap_or(false) {
        check_scope.push("user_address");
    }
    if param.email.unwrap_or(false) {
        check_scope.push("user_email");
    }
    if param.mobile.unwrap_or(false) {
        check_scope.push("user_mobile");
    }

    if !check_scope.is_empty() {
        req_dao
            .web_dao
            .web_app
            .app_dao
            .oauth_client
            .check_session_scope_data(&auth_data, &check_scope)
            .await?;
    }

    let email: Option<Vec<AccountEmailStatus>> = if param.email.unwrap_or(false) {
        Some(vec![AccountEmailStatus::Valid])
    } else {
        None
    };
    let mobile = if param.mobile.unwrap_or(false) {
        Some(vec![AccountMobileStatus::Valid])
    } else {
        None
    };
    let data_option = AccountOptionData {
        user: param.user.unwrap_or(false),
        name: param.name.unwrap_or(false),
        info: param.info.unwrap_or(false),
        address: param.address.unwrap_or(false),
        email: email.as_deref(),
        external: None,
        mobile: mobile.as_deref(),
    };
    let user_data = req_dao
        .web_dao
        .web_user
        .account
        .user_detail(account_id, &data_option)
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({
        "user_data":json!({
            "user":user_data.0,
            "name":user_data.1,
            "info":user_data.2,
            "address":user_data.3,
            "email":user_data.4,
            "mobile":user_data.6,
        })
    }))))
}
