use crate::{
    dao::RequestAuthDao,
    handler::access::{AccessSystemEmailConfirm, AccessUserEmailEdit, AccessUserEmailView},
    {CaptchaParam, JsonData, JsonResult},
};
use lsys_core::fluent_message;
use lsys_user::dao::auth::{SessionData, SessionTokenData, UserSession};
use lsys_user::model::UserEmailStatus;
use serde::Deserialize;
use serde_json::json;
#[derive(Debug, Deserialize)]
pub struct EmailAddParam {
    pub email: String,
}
pub async fn user_email_add<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: EmailAddParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let user = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user
        .find_by_id(&req_auth.user_data().user_id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessUserEmailEdit {
                user_id: req_auth.user_data().user_id,
                res_user_id: req_auth.user_data().user_id,
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let status = lsys_user::model::UserEmailStatus::Init;
    let email_id = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user_email
        .add_email(&user, param.email, status, None, Some(&req_dao.req_env))
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::data(json!({ "id": email_id })))
}

#[derive(Debug, Deserialize)]
pub struct EmailSendCodeParam {
    pub email: String,
    pub captcha: CaptchaParam,
}
pub async fn user_email_send_code<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: EmailSendCodeParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let valid_code = req_dao
        .web_dao
        .captcha
        .valid_code(&crate::dao::CaptchaKey::AddEmailCode);
    valid_code
        .check_code(&param.captcha.key, &param.captcha.code)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let email_res = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user_email
        .find_by_last_email(param.email)
        .await;
    let email = match email_res {
        Ok(email) => {
            if UserEmailStatus::Valid.eq(email.status) {
                if email.user_id != req_auth.user_data().user_id {
                    return Ok(
                        req_dao
                            .fluent_json_data(fluent_message!("mail-bind-other-user",{
                                "other_user_id":email.user_id,
                               // "user_id":req_auth.user_data().user_id
                            }))
                            .set_code(500)
                            .set_sub_code("mail-exits"),
                        // JsonData::message(format!("other user bind[{}]",)),
                    );
                } else {
                    return Ok(
                        req_dao
                            .fluent_json_data(fluent_message!("mail-is-confirm"))
                            .set_code(500)
                            .set_sub_code("mail-is-confirm"), // JsonData::message("the email is confirm")
                    );
                }
            }
            email
        }
        Err(err) => {
            if !err.is_not_found() {
                return Err(req_dao.fluent_json_data(err));
            } else {
                return Ok(req_dao.fluent_json_data(err));
            }
        }
    };

    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessUserEmailEdit {
                user_id: req_auth.user_data().user_id,
                res_user_id: email.user_id,
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let res = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user_email
        .valid_code_set(
            &mut req_dao
                .web_dao
                .user
                .user_dao
                .user_account
                .user_email
                .valid_code_builder(),
            &email.user_id,
            &email.email,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .web_dao
        .sender_mailer
        .send_valid_code(&email.email, &res.0, &res.1, Some(&req_dao.req_env))
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::default())
}

#[derive(Debug, Deserialize)]
pub struct EmailConfirmParam {
    pub email_id: u64,
    pub code: String,
}
pub async fn user_email_confirm<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: EmailConfirmParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let email = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user_email
        .find_by_id(&param.email_id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    if UserEmailStatus::Delete.eq(email.status) {
        return Ok(req_dao.fluent_json_data(fluent_message!("email-bad-status")));
    }
    if UserEmailStatus::Init.eq(email.status) {
        req_dao
            .web_dao
            .user
            .rbac_dao
            .rbac
            .check(&AccessSystemEmailConfirm {}, None)
            .await
            .map_err(|e| req_dao.fluent_json_data(e))?;
        req_dao
            .web_dao
            .user
            .user_dao
            .user_account
            .user_email
            .confirm_email_from_code(&email, &param.code, Some(&req_dao.req_env))
            .await
            .map_err(|e| req_dao.fluent_json_data(e))?;
        // Ok(JsonData::message("email confirm success"))
    } else {
        // Ok(JsonData::message("email is confirm"))
    }
    Ok(JsonData::default())
}

#[derive(Debug, Deserialize)]
pub struct EmailDeleteParam {
    pub email_id: u64,
}
pub async fn user_email_delete<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: EmailDeleteParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let res = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user_email
        .find_by_id(&param.email_id)
        .await;

    match res {
        Ok(email) => {
            if UserEmailStatus::Init.eq(email.status) || UserEmailStatus::Valid.eq(email.status) {
                req_dao
                    .web_dao
                    .user
                    .rbac_dao
                    .rbac
                    .check(
                        &AccessUserEmailEdit {
                            user_id: req_auth.user_data().user_id,
                            res_user_id: email.user_id,
                        },
                        None,
                    )
                    .await
                    .map_err(|e| req_dao.fluent_json_data(e))?;
                req_dao
                    .web_dao
                    .user
                    .user_dao
                    .user_account
                    .user_email
                    .del_email(&email, None, Some(&req_dao.req_env))
                    .await
                    .map_err(|e| req_dao.fluent_json_data(e))?;
            }
        }
        Err(e) => {
            if !e.is_not_found() {
                return Err(req_dao.fluent_json_data(e));
            }
        }
    }
    Ok(JsonData::default())
}

#[derive(Debug, Deserialize)]
pub struct EmailListDataParam {
    pub status: Option<Vec<i8>>,
}
pub async fn user_email_list_data<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: EmailListDataParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessUserEmailView {
                user_id: req_auth.user_data().user_id,
                res_user_id: req_auth.user_data().user_id,
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let status = if let Some(e) = param.status {
        let mut out = Vec::with_capacity(e.len());
        for tmp in e {
            match UserEmailStatus::try_from(tmp) {
                Ok(ts) => out.push(ts),
                Err(err) => return Err(req_dao.fluent_json_data(err)),
            };
        }
        Some(out)
    } else {
        None
    };
    let data = req_dao
        .web_dao
        .user
        .user_email(req_auth.user_data().user_id, status.as_deref())
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::data(json!({
        "data": data ,
        "total":data.len(),
    })))
}
