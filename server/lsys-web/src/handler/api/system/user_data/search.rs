use crate::{
    common::{JsonData, JsonResult, LimitParam, UserAuthQueryDao},
    dao::{access::api::system::CheckAdminUserManage, AccountOptionData},
};

use lsys_user::model::{
    AccountAddressModel, AccountEmailModel, AccountEmailStatus, AccountExternalModel,
    AccountIndexCat, AccountInfoModel, AccountMobileModel, AccountMobileStatus, AccountModel,
    AccountNameModel,
};
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct UserSearchParam {
    pub key_word: Option<String>,
    pub enable: bool,
    pub limit: Option<LimitParam>,
    pub base: Option<bool>,
    pub name: Option<bool>,
    pub info: Option<bool>,
    pub address: Option<bool>,
    pub external: Option<Vec<String>>,
    pub email: Option<Vec<i8>>,
    pub mobile: Option<Vec<i8>>,
}
pub async fn user_search(
    param: &UserSearchParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.access_env().await?, &CheckAdminUserManage {}, None)
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
    Ok(JsonData::data(json!({
        "data": out,
        "next": next
    })))
}

#[derive(Debug, Deserialize)]
pub struct UserIdSearchParam {
    pub user_id: u64,
    pub base: Option<bool>,
    pub name: Option<bool>,
    pub info: Option<bool>,
    pub address: Option<bool>,
    pub external: Option<Vec<String>>,
    pub email: Option<Vec<i8>>,
    pub mobile: Option<Vec<i8>>,
}

pub async fn user_id_search(
    param: &UserIdSearchParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.access_env().await?, &CheckAdminUserManage {}, None)
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

    if param.user_id == 0 {
        return Ok(JsonData::data(json!({
            "data": null,
        })));
    }

    let user_data = req_dao
        .web_dao
        .web_user
        .account
        .list_user(&[param.user_id], &data_option)
        .await?;
    let udat = match user_data.get(&param.user_id) {
        Some(t) => t,
        None => {
            return Ok(JsonData::data(json!({
                "data": null,
            })))
        }
    };

    Ok(JsonData::data(json!({
        "data":  json!({
            "user":udat.0,
            "name":udat.1,
            "info":udat.2,
            "address":udat.3,
            "email":udat.4,
            "external":udat.5,
            "mobile":udat.6,
        }),
    })))
}
