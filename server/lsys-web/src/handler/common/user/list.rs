use crate::{
    dao::{user::UserDataOption, RequestDao},
    handler::access::{AccessAdminUserBase, AccessAdminUserFull},
    LimitParam, {JsonData, JsonResult},
};
use lsys_user::{
    dao::auth::{SessionData, SessionTokenData, UserSession},
    model::{UserEmailStatus, UserIndexCat, UserMobileStatus},
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
pub async fn user_search<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: UserSearchParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
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
        user: true,
        name: param.name.unwrap_or(false),
        info: param.info.unwrap_or(false),
        address: param.address.unwrap_or(false),
        email: email.as_deref(),
        external: param.external.as_deref(),
        mobile: mobile.as_deref(),
    };

    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    let is_full = param.base.unwrap_or(false)
        || data_option.name
        || data_option.info
        || data_option.address
        || data_option.email.is_some()
        || data_option.external.is_some()
        || data_option.mobile.is_some();
    if is_full {
        req_dao
            .web_dao
            .user
            .rbac_dao
            .rbac
            .check(
                &AccessAdminUserFull {
                    user_id: req_auth.user_data().user_id,
                },
                None,
            )
            .await?;
    } else {
        req_dao
            .web_dao
            .user
            .rbac_dao
            .rbac
            .check(
                &AccessAdminUserBase {
                    user_id: req_auth.user_data().user_id,
                },
                None,
            )
            .await?;
    }

    let user = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user
        .search_user(
            &param.key_word.unwrap_or_default(),
            param.enable,
            &Some(param.limit.unwrap_or_default().into()),
        )
        .await?;
    let next = user.1;

    let user_data = req_dao
        .web_dao
        .user
        .list_user(
            &user.0.iter().map(|e| e.user_id).collect::<Vec<_>>(),
            &data_option,
        )
        .await?;

    let mut out = Vec::with_capacity(user_data.len());
    for (uid, udat) in user_data {
        let mut cat = vec![];
        for tmp in user.0.iter() {
            if tmp.user_id == uid {
                cat = tmp
                    .cats
                    .iter()
                    .filter_map(|e| {
                        let cn = match e.0 {
                            UserIndexCat::Email => Some("email"),
                            UserIndexCat::Mobile => Some("mobile"),
                            UserIndexCat::UserName => Some("username"),
                            UserIndexCat::NikeName => Some("nikename"),
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

        if param.base.unwrap_or(false) {
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
        } else {
            out.push(json!({
                "user":udat.0.map(|e| {
                    json!({
                        "id":e.id,
                        "add_time":e.add_time,
                        "status":e.status,
                        "nickname":e.nickname,
                    })
                }),
                "cat":cat
            }))
        }
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

pub async fn user_id_search<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: UserIdSearchParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
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
        user: true,
        name: param.name.unwrap_or(false),
        info: param.info.unwrap_or(false),
        address: param.address.unwrap_or(false),
        email: email.as_deref(),
        external: param.external.as_deref(),
        mobile: mobile.as_deref(),
    };

    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    let is_full = param.base.unwrap_or(false)
        || data_option.name
        || data_option.info
        || data_option.address
        || data_option.email.is_some()
        || data_option.external.is_some()
        || data_option.mobile.is_some();
    if is_full {
        req_dao
            .web_dao
            .user
            .rbac_dao
            .rbac
            .check(
                &AccessAdminUserFull {
                    user_id: req_auth.user_data().user_id,
                },
                None,
            )
            .await?;
    } else {
        req_dao
            .web_dao
            .user
            .rbac_dao
            .rbac
            .check(
                &AccessAdminUserBase {
                    user_id: req_auth.user_data().user_id,
                },
                None,
            )
            .await?;
    }

    if param.user_id == 0 {
        return Ok(JsonData::data(json!({
            "data": null,
        })));
    }

    let user_data = req_dao
        .web_dao
        .user
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
    let out = if param.base.unwrap_or(false) {
        json!({
            "user":udat.0,
            "name":udat.1,
            "info":udat.2,
            "address":udat.3,
            "email":udat.4,
            "external":udat.5,
            "mobile":udat.6,
        })
    } else {
        json!({
            "user":udat.0.as_ref().map(| e| {
                json!({
                    "id":e.id,
                    "add_time":e.add_time,
                    "status":e.status,
                    "nickname":e.nickname,
                })
            }),
        })
    };
    Ok(JsonData::data(json!({
        "data": out,
    })))
}
