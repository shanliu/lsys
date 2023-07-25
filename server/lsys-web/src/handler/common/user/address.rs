use crate::{
    dao::RequestDao,
    handler::access::{AccessUserAddressEdit, AccessUserAddressView},
    {JsonData, JsonResult},
};
use lsys_user::{
    dao::auth::{SessionData, SessionTokenData, UserSession},
    model::{UserAddressModelRef, UserAddressStatus},
};
use serde::Deserialize;
use serde_json::json;
use sqlx_model::model_option_set;

#[derive(Debug, Deserialize)]
pub struct AddressAddParam {
    pub code: String,
    pub info: String,
    pub detail: String,
    pub name: String,
    pub mobile: String,
}

pub async fn user_address_add<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: AddressAddParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    let user = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user
        .find_by_id(&req_auth.user_data().user_id)
        .await?;

    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessUserAddressEdit {
                user_id: req_auth.user_data().user_id,
                res_user_id: req_auth.user_data().user_id,
            },
            None,
        )
        .await?;
    let area = req_dao.web_dao.area.code_related(&param.code)?;
    if area.is_empty() {
        return Ok(
            JsonData::message("your submit area code not find any data").set_code("bad_code")
        );
    }
    let country_code = "CHN".to_string();
    let adm = model_option_set!(UserAddressModelRef, {
        country_code:country_code,
        address_code: param.code,
        address_info: param.info,
        address_detail: param.detail,
        name: param.name,
        mobile: param.mobile,
        user_id:user.id,
    });
    let id = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user_address
        .add_address(&user, adm, None, Some(&req_dao.req_env))
        .await?;
    Ok(JsonData::data(json!({ "id": id })))
}

#[derive(Debug, Deserialize)]
pub struct AddressEditParam {
    pub address_id: u64,
    pub code: String,
    pub info: String,
    pub detail: String,
    pub name: String,
    pub mobile: String,
}
pub async fn user_address_edit<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: AddressEditParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    let user = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user
        .find_by_id(&req_auth.user_data().user_id)
        .await?;

    let addres = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user_address
        .find_by_id(&param.address_id)
        .await?;

    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessUserAddressEdit {
                user_id: req_auth.user_data().user_id,
                res_user_id: addres.user_id,
            },
            None,
        )
        .await?;
    let country_code = "CHN".to_string();

    let area = req_dao.web_dao.area.code_find(&param.code)?;
    if area.is_empty() {
        return Ok(
            JsonData::message("your submit area code not find any data").set_code("bad_code")
        );
    }
    let adm = model_option_set!(UserAddressModelRef, {
        country_code:country_code,
        address_code: param.code,
        address_info: param.info,
        address_detail: param.detail,
        name: param.name,
        mobile: param.mobile,
    });
    req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user_address
        .edit_address(&addres, &user, adm, None, Some(&req_dao.req_env))
        .await?;
    Ok(JsonData::default())
}

#[derive(Debug, Deserialize)]
pub struct AddressDeleteParam {
    pub address_id: u64,
}
pub async fn user_address_delete<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: AddressDeleteParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    let res = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user_address
        .find_by_id(&param.address_id)
        .await;

    match res {
        Ok(address) => {
            if UserAddressStatus::Enable.eq(address.status) {
                req_dao
                    .web_dao
                    .user
                    .rbac_dao
                    .rbac
                    .check(
                        &AccessUserAddressEdit {
                            user_id: req_auth.user_data().user_id,
                            res_user_id: address.user_id,
                        },
                        None,
                    )
                    .await?;

                req_dao
                    .web_dao
                    .user
                    .user_dao
                    .user_account
                    .user_address
                    .del_address(&address, None, Some(&req_dao.req_env))
                    .await?;
            }
        }
        Err(e) => {
            if !e.is_not_found() {
                return Err(e.into());
            }
        }
    }
    Ok(JsonData::default())
}

pub async fn user_address_list_data<
    't,
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessUserAddressView {
                user_id: req_auth.user_data().user_id,
                res_user_id: req_auth.user_data().user_id,
            },
            None,
        )
        .await?;
    let data = req_dao
        .web_dao
        .user
        .user_address(req_auth.user_data().user_id)
        .await?;
    let area = &req_dao.web_dao.area;
    let data_list = data
        .iter()
        .map(|e| {
            let code_detail = match area.code_find(&e.address_code) {
                Ok(e) => e
                    .into_iter()
                    .map(|es| {
                        json!({
                            "name":es.name,
                            "code":es.code,
                        })
                    })
                    .collect::<Vec<_>>(),
                Err(_) => vec![],
            };
            json!({
                "id":e.id,
                "country_code":e.country_code,
                "address_code":e.address_code,
                "address_info":e.address_info,
                "code_detail":code_detail,
                "address_detail":e.address_detail,
                "name":e.name,
                "mobile":e.mobile,
                "change_time":e.change_time,
            })
        })
        .collect::<Vec<_>>();
    Ok(JsonData::data(json!({
        "data": data_list,
        "total":data.len(),
    })))
}
