use crate::{
    dao::{
        user::{ShowUserAuthData, UserDataOption},
        UserAuthQueryDao,
    },
    {JsonData, JsonResult},
};
use lsys_user::dao::auth::{SessionData, UserSession};
use lsys_user::model::{UserEmailStatus, UserMobileStatus};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct UserAuthDataOptionParam {
    pub reload_auth: Option<bool>,
    pub auth: Option<bool>,
    pub user: Option<bool>,
    pub name: Option<bool>,
    pub info: Option<bool>,
    pub address: Option<bool>,
    pub external: Option<Vec<String>>,
    pub email: Option<Vec<i8>>,
    pub mobile: Option<Vec<i8>>,
    pub password_timeout: Option<bool>, 
}

pub async fn login_data_from_user_auth(
    param: UserAuthDataOptionParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let email = if let Some(e) = param.email {
        let mut out = Vec::with_capacity(e.len());
        for tmp in e {
            match UserEmailStatus::try_from(tmp) {
                Ok(ts) => out.push(ts),
                Err(err) => return Err(JsonData::error(err)),
            };
        }
        Some(out)
    } else {
        None
    };
    let mobile = if let Some(e) = param.mobile {
        let mut out = Vec::with_capacity(e.len());
        for tmp in e {
            match UserMobileStatus::try_from(tmp) {
                Ok(ts) => out.push(ts),
                Err(err) => return Err(JsonData::error(err)),
            };
        }
        Some(out)
    } else {
        None
    };

    let data_option = UserDataOption {
        user: param.user.unwrap_or(false),
        name: param.name.unwrap_or(false),
        info: param.info.unwrap_or(false),
        address: param.address.unwrap_or(false),
        email: email.as_deref(),
        external: param.external.as_deref(),
        mobile: mobile.as_deref(),
    };
    let user_data = req_dao
        .web_dao
        .user
        .user_detail(auth_data.user_data().user_id, data_option)
        .await?;

    let passwrod_timeout = if param.password_timeout.unwrap_or(false) {
        req_dao
            .web_dao
            .user
            .user_dao
            .user_account
            .user_password
            .password_timeout(&auth_data.user_data().user_password_id)
            .await
            .unwrap_or(false)
    } else {
        false
    };

    let (token_str, auth_data) = if param.reload_auth.unwrap_or(false) {
        let mut session = req_dao.user_session.write().await;
        let _ = session.refresh_session(true).await;
        (
            Some(session.get_session_token().to_string()),
            Some(ShowUserAuthData::from(session.get_session_data().await?)),
        )
    } else if param.auth.unwrap_or(false) {
        (None, Some(ShowUserAuthData::from(auth_data)))
    } else {
        (None, None)
    };

    Ok(JsonData::data(json!({
        "auth_token":token_str,
        "auth_data": auth_data ,
        "user_data":json!({
            "user":user_data.0,
            "name":user_data.1,
            "info":user_data.2,
            "address":user_data.3,
            "email":user_data.4,
            "external":user_data.5,
            "mobile":user_data.6,
            "passwrod_timeut":passwrod_timeout
        }),
    })))
}
