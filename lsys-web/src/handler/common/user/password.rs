use serde::Deserialize;
use serde_json::json;

use crate::{
    dao::RequestDao,
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
        .access
        .check(
            req_auth.user_data().user_id,
            &[],
            &res_data!(UserSetPassword(req_auth.user_data().user_id)),
        )
        .await?;
    let user_password = &req_dao.web_dao.user.user_dao.user_account.user_password;
    if user.password_id > 0 {
        if let Some(ref old_passwrod) = param.old_password {
            let check = user_password.check_password(&user, old_passwrod).await?;
            if !check {
                return Ok(
                    JsonData::message_error("old password is wrong").set_code("bad_passwrod")
                );
            }
        } else {
            return Ok(JsonData::message_error("your need submit old password").set_code(400));
        }
    }
    let pid = user_password
        .set_passwrod(&user, param.new_password.clone(), None)
        .await?;
    Ok(JsonData::data(json!({ "id": pid })))
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordSendCodeFromMobileParam {
    pub mobile: String,
    pub area_code: String,
    pub captcha: CaptchaParam,
}
pub async fn user_reset_password_send_code_from_mobile<
    't,
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    param: ResetPasswordSendCodeFromMobileParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let mobile = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user_mobile
        .find_by_last_mobile(param.area_code.clone(), param.mobile.clone())
        .await?;
    mobile.is_enable()?;
    let valid_code = req_dao
        .web_dao
        .captcha
        .valid_code(&crate::dao::CaptchaKey::ResetPasswordSms);

    valid_code
        .check_code(&param.captcha.key, &param.captcha.code)
        .await?;
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
        .await?;
    req_dao
        .web_dao
        .smser
        .send_valid_code(&param.area_code, &param.mobile, &data.0, &data.1)
        .await?;
    req_dao
        .web_dao
        .captcha
        .clear_code(&valid_code, &param.captcha.key)
        .await;
    Ok(JsonData::message("reset sms is send").set_data(json!({ "ttl": data.1 })))
}
#[derive(Debug, Deserialize)]
pub struct ResetPasswordSendCodeFromEmailParam {
    pub email: String,
    pub captcha: CaptchaParam,
}
pub async fn user_reset_password_send_code_from_email<
    't,
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    param: ResetPasswordSendCodeFromEmailParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let user_email = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user_email
        .find_by_last_email(param.email.clone())
        .await?;
    user_email.is_enable()?;
    let valid_code = req_dao
        .web_dao
        .captcha
        .valid_code(&crate::dao::CaptchaKey::ResetPasswordMail);
    valid_code
        .check_code(&param.captcha.key, &param.captcha.code)
        .await?;
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
        .await?;
    req_dao
        .web_dao
        .mailer
        .send_valid_code(&param.email, &data.0, &data.1)
        .await?;
    req_dao
        .web_dao
        .captcha
        .clear_code(&valid_code, &param.captcha.key)
        .await;
    Ok(JsonData::message("reset mail is send").set_data(json!({ "ttl": data.1 })))
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordFromEmailParam {
    pub email: String,
    pub code: String,
    pub new_password: String,
}
pub async fn user_reset_password_from_email<
    't,
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    param: &ResetPasswordFromEmailParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let email = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user_email
        .find_by_last_email(param.email.clone())
        .await?;
    email.is_enable()?;
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
    't,
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    param: &ResetPasswordFromMobileParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let mobile = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user_mobile
        .find_by_last_mobile(param.area_code.clone(), param.mobile.clone())
        .await?;
    mobile.is_enable()?;
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
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .access
        .check(0, &[], &res_data!(SystemReSetPassword))
        .await?;
    let user = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user
        .find_by_id(user_id)
        .await?;
    user.is_enable()?;

    let pid = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user_password
        .set_passwrod_from_code(&user, new_password.to_owned(), from_type, code, None)
        .await?;
    Ok(JsonData::data(json!({ "id": pid })))
}
