use crate::{
    common::{CaptchaParam, JsonData, JsonResult, RequestDao, UserAuthQueryDao},
    dao::access::api::user::{CheckUserMobileBase, CheckUserMobileEdit},
};
use lsys_access::dao::{AccessSession, AccessSessionData};

use lsys_user::model::AccountMobileStatus;
use serde::Deserialize;
use serde_json::json;
#[derive(Debug, Deserialize)]
pub struct MobileAddParam {
    pub area_code: String,
    pub mobile: String,
}
pub async fn mobile_add(
    param: &MobileAddParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckUserMobileBase {})
        .await?;
    let mobile_id = req_dao
        .web_dao
        .web_user
        .account
        .user_mobile_add(
            &param.area_code,
            &param.mobile,
            auth_data.session_body(),
            Some(&req_dao.req_env),
        )
        .await?;

    Ok(JsonData::data(json!({ "id": mobile_id })))
}

#[derive(Debug, Deserialize)]
pub struct MobileSendCodeParam {
    pub area_code: String,
    pub mobile: String,
    pub captcha: CaptchaParam,
}
pub async fn mobile_send_code(
    param: &MobileSendCodeParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
  
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckUserMobileBase {})
        .await?;
    req_dao
        .web_dao
        .web_user
        .account
        .user_mobile_send_code(
            &param.area_code,
            &param.mobile,
            &param.captcha,
            &auth_data,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::default())
}

#[derive(Debug, Deserialize)]
pub struct MobileConfirmParam {
    pub mobile_id: u64,
    pub code: String,
}
pub async fn mobile_confirm(
    param: &MobileConfirmParam,
    req_dao: &RequestDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, None, &CheckUserMobileBase {})
        .await?;
    let mobile = req_dao
        .web_dao
        .web_user
        .user_dao
        .account_dao
        .account_mobile
        .find_by_id(&param.mobile_id)
        .await?;
    req_dao
        .web_dao
        .web_user
        .user_dao
        .account_dao
        .account_mobile
        .confirm_mobile_from_code(&mobile, &param.code, 0, Some(&req_dao.req_env))
        .await?;

    Ok(JsonData::default())
}

#[derive(Debug, Deserialize)]
pub struct MobileDeleteParam {
    pub mobile_id: u64,
}
pub async fn mobile_delete(
    param: &MobileDeleteParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    let mobile = req_dao
        .web_dao
        .web_user
        .user_dao
        .account_dao
        .account_mobile
        .find_by_id(&param.mobile_id)
        .await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &req_dao.req_env,Some(&auth_data),
            &CheckUserMobileEdit {
                res_user_id: req_dao
                    .web_dao
                    .web_user
                    .account
                    .account_id_to_user(mobile.account_id)
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
        .account_mobile
        .del_mobile(&mobile, auth_data.user_id(), None, Some(&req_dao.req_env))
        .await?;

    Ok(JsonData::default())
}

#[derive(Debug, Deserialize)]
pub struct MobileListDataParam {
    pub status: Option<Vec<i8>>,
}
pub async fn mobile_list_data(
    param: &MobileListDataParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    let status = if let Some(ref e) = param.status {
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
    let data = req_dao
        .web_dao
        .web_user
        .account
        .user_mobile(auth_data.user_id(), status.as_deref())
        .await?;
    Ok(JsonData::data(json!({
        "data": data ,
        "total":data.len(),
    })))
}
