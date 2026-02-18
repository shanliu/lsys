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
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub auth: Option<bool>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub user: Option<bool>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub name: Option<bool>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub info: Option<bool>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub address: Option<bool>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub email: Option<bool>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub mobile: Option<bool>,
}

pub async fn account_data_from_oauth(
    param: &AccountOptionDataParam,
    req_dao: &RestAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    if auth_data.user().app_id > 0 {
        //oauth 服务处理,有2种传递数据方式
        //1. 外部站点先把数据存储在session中,通过这里获取
        //2. 外部站点提供接口对外获取前,通过接口检查lsys系统中是否对应的scope授权是否存在
        // let scope_data = req_dao
        //     .web_dao
        //     .web_access
        //     .access_dao
        //     .auth
        //     .session_get_vec_data(
        //         &auth_data,
        //         &req_dao
        //             .web_dao
        //             .web_app
        //             .app_dao
        //             .oauth_client
        //             .get_session_scope_data(&auth_data)
        //             .await?
        //             .iter()
        //             .map(|s| s.as_str())
        //             .collect::<Vec<&str>>(),
        //     )
        //     .await?;
        Ok(JsonResponse::data(JsonData::body(json!({
             "auth_data":auth_data.user(),
        }))))
    } else {
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

        let user_data = if !check_scope.is_empty() {
            req_dao
                .web_dao
                .web_app
                .app_dao
                .oauth_client
                .check_session_scope_data(&auth_data, &check_scope)
                .await?;
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
            json!({
                "account":user_data.0,
                "name":user_data.1,
                "info":user_data.2,
                "address":user_data.3,
                "email":user_data.4,
                "mobile":user_data.6,
            })
        } else {
            json!({})
        };
        Ok(JsonResponse::data(JsonData::body(json!({
            "auth_data":auth_data.user(),
            "user_data":user_data
        }))))
    }
}
