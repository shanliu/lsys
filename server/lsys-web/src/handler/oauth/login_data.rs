use crate::{
    dao::{user::UserDataOption, RestAuthQueryDao},
    handler::access::{
        AccessOauthUserAddress, AccessOauthUserEmail, AccessOauthUserInfo, AccessOauthUserMobile,
    },
    {JsonData, JsonResult},
};
use lsys_app::model::AppsModel;
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
    pub email: Option<bool>,
    pub mobile: Option<bool>,
}

pub async fn login_data_from_oauth(
    param: OauthDataOptionParam,
    app: &AppsModel,
    req_dao: &RestAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    if param.info.unwrap_or(false) || param.name.unwrap_or(false) {
        req_dao
            .web_dao
            .user
            .rbac_dao
            .rbac
            .check(
                &AccessOauthUserInfo {
                    app_id: app.id,
                    user_id: app.user_id,
                },
                None,
            )
            .await?;
        auth_data.check_scope("user_info")?;
    }
    if param.address.unwrap_or(false) {
        req_dao
            .web_dao
            .user
            .rbac_dao
            .rbac
            .check(
                &AccessOauthUserAddress {
                    app_id: app.id,
                    user_id: app.user_id,
                },
                None,
            )
            .await?;
        auth_data.check_scope("user_address")?;
    }
    if param.email.unwrap_or(false) {
        req_dao
            .web_dao
            .user
            .rbac_dao
            .rbac
            .check(
                &AccessOauthUserEmail {
                    app_id: app.id,
                    user_id: app.user_id,
                },
                None,
            )
            .await?;
        auth_data.check_scope("user_email")?;
    }
    if param.mobile.unwrap_or(false) {
        req_dao
            .web_dao
            .user
            .rbac_dao
            .rbac
            .check(
                &AccessOauthUserMobile {
                    app_id: app.id,
                    user_id: app.user_id,
                },
                None,
            )
            .await?;
        auth_data.check_scope("user_mobile")?;
    }

    let email: Option<Vec<UserEmailStatus>> = if param.email.unwrap_or(false) {
        Some(vec![UserEmailStatus::Valid])
    } else {
        None
    };
    let mobile = if param.mobile.unwrap_or(false) {
        Some(vec![UserMobileStatus::Valid])
    } else {
        None
    };
    let data_option = UserDataOption {
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
            "mobile":user_data.6,
        }),
    })))
}
