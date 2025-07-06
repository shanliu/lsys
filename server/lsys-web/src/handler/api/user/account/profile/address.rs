use crate::common::JsonData;
use crate::dao::access::RbacAccessCheckEnv;
use crate::{
    common::{JsonResponse, JsonResult, UserAuthQueryDao},
    dao::{access::api::system::user::CheckUserAddressEdit, AddressData},
};
use lsys_access::dao::{AccessSession, AccessSessionData};
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct AddressAddParam {
    pub code: String,
    pub info: String,
    pub detail: String,
    pub name: String,
    pub mobile: String,
}

pub async fn address_add(
    param: &AddressAddParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserAddressEdit {
                res_user_id: auth_data.user_id(),
            },
        )
        .await?;

    let id = req_dao
        .web_dao
        .web_user
        .account
        .user_address_add(
            &AddressData {
                code: &param.code,
                info: &param.info,
                detail: &param.detail,
                name: &param.name,
                mobile: &param.mobile,
            },
            auth_data.session_body(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({ "id": id }))))
}

#[derive(Debug, Deserialize)]
pub struct AddressEditParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub address_id: u64,
    pub code: String,
    pub info: String,
    pub detail: String,
    pub name: String,
    pub mobile: String,
}
pub async fn address_edit(
    param: &AddressEditParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    let address = req_dao
        .web_dao
        .web_user
        .user_dao
        .account_dao
        .account_address
        .find_by_id(&param.address_id)
        .await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserAddressEdit {
                res_user_id: req_dao
                    .web_dao
                    .web_user
                    .account
                    .account_id_to_user(address.account_id)
                    .await?
                    .id,
            },
        )
        .await?;

    req_dao
        .web_dao
        .web_user
        .account
        .user_address_edit(
            &address,
            &AddressData {
                code: param.code.as_str(),
                info: param.info.as_str(),
                detail: param.detail.as_str(),
                name: param.name.as_str(),
                mobile: param.mobile.as_str(),
            },
            auth_data.session_body(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::default())
}

#[derive(Debug, Deserialize)]
pub struct AddressDeleteParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub address_id: u64,
}
pub async fn address_delete(
    param: &AddressDeleteParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let address = req_dao
        .web_dao
        .web_user
        .user_dao
        .account_dao
        .account_address
        .find_by_id(&param.address_id)
        .await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserAddressEdit {
                res_user_id: req_dao
                    .web_dao
                    .web_user
                    .account
                    .account_id_to_user(address.account_id)
                    .await?
                    .id,
            },
        )
        .await?;
    req_dao
        .web_dao
        .web_user
        .user_dao
        .account_dao
        .account_address
        .del_address(&address, None, auth_data.user_id(), Some(&req_dao.req_env))
        .await?;
    Ok(JsonResponse::default())
}

pub async fn address_list_data(req_dao: &UserAuthQueryDao) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    let data = req_dao
        .web_dao
        .web_user
        .account
        .user_address(auth_data.user_id())
        .await?;
    let data_list = data
        .iter()
        .map(|e| {
            let code_detail = match req_dao.web_dao.app_area.code_find(&e.address_code) {
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
    Ok(JsonResponse::data(JsonData::body(json!({
        "data": data_list,
        "total":data.len(),
    }))))
}
