use crate::common::JsonData;
use crate::dao::access::RbacAccessCheckEnv;
use crate::{
    common::{CaptchaParam, JsonResponse, JsonResult, RequestDao, UserAuthQueryDao},
    dao::access::api::system::user::CheckUserEmailEdit,
};
use lsys_access::dao::{AccessSession, AccessSessionData};
use lsys_user::model::AccountEmailStatus;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct EmailAddParam {
    pub email: String,
}
pub async fn email_add(
    param: &EmailAddParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::sys_user(auth_data.user_id(), &req_dao.req_env),
            &CheckUserEmailEdit {
                res_user_id: auth_data.user_id(),
            },
        )
        .await?;

    let id = req_dao
        .web_dao
        .web_user
        .account
        .user_email_add(
            &param.email,
            auth_data.session_body(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({
        "id":id
    }))))
}
#[derive(Debug, Deserialize)]
pub struct EmailSendCodeParam {
    pub email: String,
    pub captcha: CaptchaParam,
}
pub async fn email_send_code(
    param: &EmailSendCodeParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::sys_user(auth_data.user_id(), &req_dao.req_env),
            &CheckUserEmailEdit {
                res_user_id: auth_data.user_id(),
            },
        )
        .await?;

    req_dao
        .web_dao
        .web_user
        .account
        .user_email_send_code(
            &param.email,
            &param.captcha,
            &auth_data,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::default())
}
#[derive(Debug, Deserialize)]
pub struct EmailConfirmParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub email_id: u64,
    pub code: String,
}
pub async fn email_confirm(
    param: &EmailConfirmParam,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    let email = req_dao
        .web_dao
        .web_user
        .user_dao
        .account_dao
        .account_email
        .find_by_id(&param.email_id)
        .await?;
    let uid = req_dao
        .web_dao
        .web_user
        .account
        .account_id_to_user(email.account_id)
        .await?
        .id;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::sys_user(uid, &req_dao.req_env),
            &CheckUserEmailEdit { res_user_id: uid },
        )
        .await?;
    req_dao
        .web_dao
        .web_user
        .user_dao
        .account_dao
        .account_email
        .confirm_email_from_code(&email, &param.code, 0, Some(&req_dao.req_env))
        .await?;

    Ok(JsonResponse::default())
}
#[derive(Debug, Deserialize)]
pub struct EmailDeleteParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub email_id: u64,
}
pub async fn email_delete(
    param: &EmailDeleteParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let email = req_dao
        .web_dao
        .web_user
        .user_dao
        .account_dao
        .account_email
        .find_by_id(&param.email_id)
        .await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserEmailEdit {
                res_user_id: req_dao
                    .web_dao
                    .web_user
                    .account
                    .account_id_to_user(email.account_id)
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
        .account_email
        .del_email(&email, auth_data.user_id(), None, Some(&req_dao.req_env))
        .await?;
    Ok(JsonResponse::default())
}
#[derive(Debug, Deserialize)]
pub struct EmailListDataParam {
    #[serde(default, deserialize_with = "crate::common::deserialize_option_vec_i8")]
    pub status: Option<Vec<i8>>,
}
pub async fn email_list_data(
    param: &EmailListDataParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let status = if let Some(ref e) = param.status {
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
    let data = req_dao
        .web_dao
        .web_user
        .account
        .user_email(auth_data.user_id(), status.as_deref())
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({
        "data": data ,
        "total":data.len(),
    }))))
}
