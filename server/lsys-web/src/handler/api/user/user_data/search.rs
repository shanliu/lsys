use lsys_access::dao::{AccessSession, AccessSessionData, AccessSessionToken};
use lsys_user::model::AccountIndexCat;
use serde::Deserialize;
use serde_json::json;

use crate::common::{JsonData, JsonResult, LimitParam, RequestAuthDao};

#[derive(Debug, Deserialize)]
pub struct UserSearchParam {
    pub key_word: Option<String>,
    pub enable: bool,
    pub limit: Option<LimitParam>,
}
pub async fn user_search<T: AccessSessionToken, D: AccessSessionData, S: AccessSession<T, D>>(
    param: &UserSearchParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
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
        .user_dao
        .account_dao
        .account
        .cache()
        .find_by_ids(&user.0.iter().map(|e| e.account_id).collect::<Vec<_>>())
        .await?;
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
            "id":udat.id,
            "nickname":udat.nickname,
            "confirm_time":udat.confirm_time,
            "cat":cat,
            "status":udat.status,
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
}

pub async fn user_id_search<T: AccessSessionToken, D: AccessSessionData, S: AccessSession<T, D>>(
    param: &UserIdSearchParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let udat = req_dao
        .web_dao
        .web_user
        .user_dao
        .account_dao
        .account
        .cache()
        .find_by_id(&param.user_id)
        .await?;
    Ok(JsonData::data(json!({
        "data": json!({
            "id":udat.id,
            "nickname":udat.nickname,
            "confirm_time":udat.confirm_time,
            "status":udat.status,
        }),
    })))
}
