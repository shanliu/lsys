use crate::common::UserAuthQueryDao;
use crate::dao::{AccountOptionData, UserAuthDataOptionData};
use crate::{common::JsonResult, dao::ShowUserAuthData};
use lsys_user::dao::UserAuthData;
use lsys_user::model::{
    AccountAddressModel, AccountEmailModel, AccountEmailStatus, AccountExternalModel,
    AccountInfoModel, AccountMobileModel, AccountMobileStatus, AccountModel, AccountNameModel,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct UserAuthDataOptionParam {
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub reload_auth: Option<bool>,
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
    pub external: Option<Vec<String>>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_vec_i8")]
    pub email: Option<Vec<i8>>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_vec_i8")]
    pub mobile: Option<Vec<i8>>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub password_timeout: Option<bool>,
}

pub async fn login_data_from_user_auth(
    param: &UserAuthDataOptionParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<(
    UserAuthData,
    Option<ShowUserAuthData>,
    (
        Option<AccountModel>,
        Option<AccountNameModel>,
        Option<AccountInfoModel>,
        Option<Vec<AccountAddressModel>>,
        Option<Vec<AccountEmailModel>>,
        Option<Vec<AccountExternalModel>>,
        Option<Vec<AccountMobileModel>>,
    ),
    bool,
)> {
    let email = if let Some(ref e) = param.email {
        let mut out = Vec::with_capacity(e.len());
        for tmp in e {
            match AccountEmailStatus::try_from(*tmp) {
                Ok(ts) => out.push(ts),
                Err(err) => return Err(err.into()),
            };
        }
        Some(out)
    } else {
        None
    };
    let mobile = if let Some(ref e) = param.mobile {
        let mut out = Vec::with_capacity(e.len());
        for tmp in e {
            match AccountMobileStatus::try_from(*tmp) {
                Ok(ts) => out.push(ts),
                Err(err) => return Err(err.into()),
            };
        }
        Some(out)
    } else {
        None
    };
    let external = param
        .external
        .as_ref()
        .map(|e| e.iter().map(|e| e.as_str()).collect::<Vec<_>>());

    let (auth_data, out_auth_data, passwrod_timeout) = req_dao
        .web_dao
        .web_user
        .auth
        .login_data_from_user_auth(
            &req_dao.user_session,
            &UserAuthDataOptionData {
                reload_auth: param.reload_auth,
                auth: param.auth,
                password_timeout: param.password_timeout,
            },
        )
        .await?;
    let user_data = req_dao
        .web_dao
        .web_user
        .account
        .user_detail(
            auth_data.account_id()?,
            &AccountOptionData {
                user: param.user.unwrap_or(false),
                name: param.name.unwrap_or(false),
                info: param.info.unwrap_or(false),
                address: param.address.unwrap_or(false),
                email: email.as_deref(),
                external: external.as_deref(),
                mobile: mobile.as_deref(),
            },
        )
        .await?;
    Ok((auth_data, out_auth_data, user_data, passwrod_timeout))
}
