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
        .check(&AccessUserAddressEdit {
            user_id: req_auth.user_data().user_id,
            res_user_id: req_auth.user_data().user_id,
        })
        .await?;

    let adm = model_option_set!(UserAddressModelRef, {
        address_code: param.code,
        address_info: param.info,
        address_detail: param.detail,
        name: param.name,
        mobile: param.mobile,
    });
    let id = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user_address
        .add_address(&user, adm, None)
        .await?;
    Ok(JsonData::data(json!({ "id": id })))
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
                    .check(&AccessUserAddressEdit {
                        user_id: req_auth.user_data().user_id,
                        res_user_id: address.user_id,
                    })
                    .await?;

                req_dao
                    .web_dao
                    .user
                    .user_dao
                    .user_account
                    .user_address
                    .del_address(&address, None)
                    .await?;
            }
        }
        Err(e) => {
            if !e.is_not_found() {
                return Err(e.into());
            }
        }
    }
    Ok(JsonData::message("del address ok"))
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
        .check(&AccessUserAddressView {
            user_id: req_auth.user_data().user_id,
            res_user_id: req_auth.user_data().user_id,
        })
        .await?;
    let data = req_dao
        .web_dao
        .user
        .user_address(req_auth.user_data().user_id)
        .await?;
    Ok(JsonData::data(json!({
        "data": data ,
        "total":data.len(),
    })))
}
