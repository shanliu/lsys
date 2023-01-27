use crate::{
    dao::{user::UserDataOption, RestAuthQueryDao},
    {JsonData, JsonResult},
};
use lsys_user::dao::auth::{SessionData, UserSession};
use lsys_user::model::{UserEmailStatus, UserMobileStatus};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct OauthDataOptionParam {
    pub auth: Option<bool>,
    pub user: Option<bool>,
    pub name: Option<bool>,
    pub info: Option<bool>,
    pub address: Option<bool>,
    pub external: Option<Vec<String>>,
    pub email: Option<Vec<i8>>,
    pub mobile: Option<Vec<i8>>,
}

pub async fn login_data_from_oauth(
    param: OauthDataOptionParam,
    req_dao: &RestAuthQueryDao,
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
    Ok(JsonData::data(json!({
        "user_data":json!({
            "user":user_data.0,
            "name":user_data.1,
            "info":user_data.2,
            "address":user_data.3,
            "email":user_data.4,
            "external":user_data.5,
            "mobile":user_data.6,
        }),
    })))
}
