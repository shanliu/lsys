use crate::common::JsonData;
use crate::{
    common::{JsonResponse, JsonResult, LimitParam, UserAuthQueryDao},
    dao::{access::api::system::CheckAdminUserManage, AccountOptionData},
};
use lsys_access::dao::AccessSession;
use lsys_user::model::{
    AccountAddressModel, AccountEmailModel, AccountEmailStatus, AccountExternalModel,
    AccountIndexCat, AccountInfoModel, AccountMobileModel, AccountMobileStatus, AccountModel,
    AccountNameModel,
};
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct AccountSearchParam {
    pub key_word: Option<String>,
    #[serde(deserialize_with = "crate::common::deserialize_bool")]
    pub enable: bool,
    pub limit: Option<LimitParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub base: Option<bool>,
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
}
pub async fn account_search(
    param: &AccountSearchParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminUserManage {})
        .await?;

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
        .map(|e| e.iter().map(|s| s.as_str()).collect::<Vec<_>>());
    let data_option = AccountOptionData {
        user: true,
        name: param.name.unwrap_or(false),
        info: param.info.unwrap_or(false),
        address: param.address.unwrap_or(false),
        email: email.as_deref(),
        external: external.as_ref().map(|e| e.as_ref()),
        mobile: mobile.as_deref(),
    };

    let user = req_dao
        .web_dao
        .web_user
        .user_dao
        .account_dao
        .account
        .search(
            &param.key_word.to_owned().unwrap_or_default(),
            param.enable,
            param.limit.as_ref().map(|e| e.into()).as_ref(),
        )
        .await?;
    let next = user.1;

    let user_data = req_dao
        .web_dao
        .web_user
        .account
        .list_user(
            &user.0.iter().map(|e| e.account_id).collect::<Vec<_>>(),
            &data_option,
        )
        .await?;
    let mut user_data = user_data.into_iter().collect::<Vec<(
        u64,
        (
            Option<AccountModel>,
            Option<AccountNameModel>,
            Option<AccountInfoModel>,
            Option<Vec<AccountAddressModel>>,
            Option<Vec<AccountEmailModel>>,
            Option<Vec<AccountExternalModel>>,
            Option<Vec<AccountMobileModel>>,
        ),
    )>>();
    user_data.sort_by(|a, b| b.0.cmp(&a.0));

    let mut out = Vec::with_capacity(user_data.len());
    for (uid, udat) in user_data {
        let mut cat = vec![];
        for tmp in user.0.iter() {
            if tmp.account_id == uid {
                cat = tmp
                    .cats
                    .iter()
                    .filter_map(|e| {
                        let cn = match e.0 {
                            AccountIndexCat::Email => Some("email"),
                            AccountIndexCat::Mobile => Some("mobile"),
                            AccountIndexCat::AccountName => Some("username"),
                            AccountIndexCat::NikeName => Some("nikename"),
                            _ => None,
                        };
                        cn.map(|ce| {
                            json!({
                                "type":ce,
                                "val":e.1.to_owned()
                            })
                        })
                    })
                    .collect::<Vec<_>>();
                break;
            }
        }

        out.push(json!({
            "user":udat.0,
            "name":udat.1,
            "info":udat.2,
            "address":udat.3,
            "email":udat.4,
            "external":udat.5,
            "mobile":udat.6,
            "cat":cat
        }))
    }
    Ok(JsonResponse::data(JsonData::body(json!({
        "data": out,
        "next": next
    }))))
}

#[derive(Debug, Deserialize)]
pub struct AccountIdSearchParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub account_id: u64,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub base: Option<bool>,
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
}

pub async fn account_id_search(
    param: &AccountIdSearchParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminUserManage {})
        .await?;

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
        .map(|e| e.iter().map(|s| s.as_str()).collect::<Vec<_>>());
    let data_option = AccountOptionData {
        user: true,
        name: param.name.unwrap_or(false),
        info: param.info.unwrap_or(false),
        address: param.address.unwrap_or(false),
        email: email.as_deref(),
        external: external.as_ref().map(|e| e.as_ref()),
        mobile: mobile.as_deref(),
    };

    if param.account_id == 0 {
        return Ok(JsonResponse::data(JsonData::body(json!({
            "data": null,
        }))));
    }

    let user_data = req_dao
        .web_dao
        .web_user
        .account
        .list_user(&[param.account_id], &data_option)
        .await?;
    let udat = match user_data.get(&param.account_id) {
        Some(t) => t,
        None => {
            return Ok(JsonResponse::data(JsonData::body(json!({
                "data": null,
            }))))
        }
    };

    Ok(JsonResponse::data(JsonData::body(json!({
        "data":  json!({
            "user":udat.0,
            "name":udat.1,
            "info":udat.2,
            "address":udat.3,
            "email":udat.4,
            "external":udat.5,
            "mobile":udat.6,
        }),
    }))))
}
