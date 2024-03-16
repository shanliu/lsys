use lsys_core::fluent_message;
use serde::Deserialize;
use serde_json::json;

use crate::{
    dao::RequestAuthDao,
    handler::access::{AccessSystemReSetPassword, AccessUserSetPassword},
    {CaptchaParam, JsonData, JsonResult},
};
use lsys_user::dao::auth::{SessionData, SessionTokenData, UserSession};
#[derive(Debug, Deserialize)]
pub struct SetPasswordParam {
    pub old_password: Option<String>,
    pub new_password: String,
}
pub async fn user_set_password<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: &SetPasswordParam,
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
            &AccessUserSetPassword {
                user_id: req_auth.user_data().user_id,
                res_user_id: req_auth.user_data().user_id,
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let user_password = &req_dao.web_dao.user.user_dao.user_account.user_password;
    if user.password_id > 0 {
        if let Some(ref old_passwrod) = param.old_password {
            let check = user_password
                .check_password(&user, old_passwrod)
                .await
                .map_err(|e| req_dao.fluent_json_data(e))?;
            if !check {
                return Ok(req_dao
                    .fluent_json_data(fluent_message!("user-old-passwrod-bad"))
                    .set_sub_code("bad_passwrod"));
                //   return Ok(JsonData::message("old password is wrong").set_sub_code("bad_passwrod"));
            }
        } else {
            return Ok(req_dao
                .fluent_json_data(fluent_message!("user-old-passwrod-empty"))
                .set_sub_code("need_old_passwrod"));

            // return Ok(JsonData::message("your need submit old password")
            //     .set_sub_code(""));
        }
    }
    let pid = user_password
        .set_passwrod(&user, param.new_password.clone(), None)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::data(json!({ "id": pid })))
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordSendCodeFromMobileParam {
    pub mobile: String,
    pub area_code: String,
    pub captcha: CaptchaParam,
}
pub async fn user_reset_password_send_code_from_mobile<
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    param: ResetPasswordSendCodeFromMobileParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let mobile = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user_mobile
        .find_by_last_mobile(param.area_code.clone(), param.mobile.clone())
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    mobile
        .is_enable()
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let valid_code = req_dao
        .web_dao
        .captcha
        .valid_code(&crate::dao::CaptchaKey::ResetPasswordSms);

    valid_code
        .check_code(&param.captcha.key, &param.captcha.code)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let data = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user_password
        .valid_code_set(
            &mut req_dao
                .web_dao
                .user
                .user_dao
                .user_account
                .user_password
                .valid_code_builder(),
            &mobile.user_id,
            &format!("mobile-{}-{}", param.area_code, param.mobile),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .web_dao
        .sender_smser
        .send_valid_code(
            &param.area_code,
            &param.mobile,
            &data.0,
            &data.1,
            Some(&req_dao.req_env),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .web_dao
        .captcha
        .clear_code(&valid_code, &param.captcha.key)
        .await;
    Ok(JsonData::data(json!({ "ttl": data.1 })))
}
#[derive(Debug, Deserialize)]
pub struct ResetPasswordSendCodeFromEmailParam {
    pub email: String,
    pub captcha: CaptchaParam,
}
pub async fn user_reset_password_send_code_from_email<
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    param: ResetPasswordSendCodeFromEmailParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let user_email = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user_email
        .find_by_last_email(param.email.clone())
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    user_email
        .is_enable()
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let valid_code = req_dao
        .web_dao
        .captcha
        .valid_code(&crate::dao::CaptchaKey::ResetPasswordMail);
    valid_code
        .check_code(&param.captcha.key, &param.captcha.code)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let data = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user_password
        .valid_code_set(
            &mut req_dao
                .web_dao
                .user
                .user_dao
                .user_account
                .user_password
                .valid_code_builder(),
            &user_email.user_id,
            &format!("mail-{}", param.email),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .web_dao
        .sender_mailer
        .send_valid_code(&param.email, &data.0, &data.1, Some(&req_dao.req_env))
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .web_dao
        .captcha
        .clear_code(&valid_code, &param.captcha.key)
        .await;
    Ok(JsonData::data(json!({ "ttl": data.1 })))
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordFromEmailParam {
    pub email: String,
    pub code: String,
    pub new_password: String,
}
pub async fn user_reset_password_from_email<
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    param: &ResetPasswordFromEmailParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let email = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user_email
        .find_by_last_email(param.email.clone())
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    email.is_enable().map_err(|e| req_dao.fluent_json_data(e))?;
    reset_password(
        &email.user_id,
        &param.new_password,
        &format!("mail-{}", param.email),
        &param.code,
        req_dao,
    )
    .await
}
#[derive(Debug, Deserialize)]
pub struct ResetPasswordFromMobileParam {
    pub area_code: String,
    pub mobile: String,
    pub code: String,
    pub new_password: String,
}
pub async fn user_reset_password_from_mobile<
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    param: &ResetPasswordFromMobileParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let mobile = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user_mobile
        .find_by_last_mobile(param.area_code.clone(), param.mobile.clone())
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    mobile
        .is_enable()
        .map_err(|e| req_dao.fluent_json_data(e))?;
    reset_password(
        &mobile.user_id,
        &param.new_password,
        &format!("mobile-{}-{}", param.area_code, param.mobile),
        &param.code,
        req_dao,
    )
    .await
}

async fn reset_password<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    user_id: &u64,
    new_password: &str,
    from_type: &String,
    code: &String,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(&AccessSystemReSetPassword {}, None)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let user = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user
        .find_by_id(user_id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    user.is_enable().map_err(|e| req_dao.fluent_json_data(e))?;

    let pid = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user_password
        .set_passwrod_from_code(&user, new_password.to_owned(), from_type, code, None)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::data(json!({ "id": pid })))
}
