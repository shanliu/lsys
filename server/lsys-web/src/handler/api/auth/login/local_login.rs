use std::borrow::Borrow;

use crate::{
    common::{CaptchaParam, JsonData, JsonResult, RequestDao, UserAuthQueryDao},
    dao::{access::common::CheckSystemLogin, ShowUserAuthData},
};

use lsys_user::dao::{
    login::{EmailCodeLogin, EmailLogin, MobileCodeLogin, MobileLogin, NameLogin},
    UserAuthToken,
};
use serde::Deserialize;
use serde_json::json;
#[derive(Deserialize)]
pub struct NameLoginParam {
    name: String,
    password: String,
    captcha: Option<CaptchaParam>,
}

pub async fn user_login_from_name(
    param: &NameLoginParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<(UserAuthToken, ShowUserAuthData)> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, None, &CheckSystemLogin {})
        .await?;
    req_dao
        .web_dao
        .web_user
        .auth
        .user_login(
            &NameLogin::new(
                req_dao.web_dao.web_user.user_dao.account_dao.clone(),
                &param.name,
                &param.password,
            ),
            param.captcha.as_ref(),
            &req_dao.user_session,
            Some(&req_dao.req_env),
        )
        .await
}

#[derive(Deserialize)]
pub struct EmailLoginParam {
    email: String,
    password: String,
    captcha: Option<CaptchaParam>,
}
pub async fn user_login_from_email(
    param: &EmailLoginParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<(UserAuthToken, ShowUserAuthData)> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, None, &CheckSystemLogin {})
        .await?;
    req_dao
        .web_dao
        .web_user
        .auth
        .user_login(
            &EmailLogin::new(
                req_dao.web_dao.web_user.user_dao.account_dao.clone(),
                &param.email,
                &param.password,
            ),
            param.captcha.as_ref(),
            &req_dao.user_session,
            Some(&req_dao.req_env),
        )
        .await
}

#[derive(Deserialize)]
pub struct EmailCodeLoginParam {
    email: String,
    code: String,
    captcha: Option<CaptchaParam>,
}

pub async fn user_login_from_email_code(
    param: &EmailCodeLoginParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<(UserAuthToken, ShowUserAuthData)> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, None, &CheckSystemLogin {})
        .await?;
    req_dao
        .web_dao
        .web_user
        .auth
        .user_login(
            &EmailCodeLogin::new(
                req_dao.web_dao.redis.clone(),
                req_dao.web_dao.web_user.user_dao.account_dao.clone(),
                &param.email,
                &param.code,
            ),
            param.captcha.as_ref(),
            &req_dao.user_session,
            Some(&req_dao.req_env),
        )
        .await
}

#[derive(Deserialize)]
pub struct MobileLoginParam {
    area_code: String,
    mobile: String,
    password: String,
    captcha: Option<CaptchaParam>,
}

pub async fn user_login_from_mobile(
    param: &MobileLoginParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<(UserAuthToken, ShowUserAuthData)> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, None, &CheckSystemLogin {})
        .await?;
    req_dao
        .web_dao
        .web_user
        .auth
        .user_login(
            &MobileLogin::new(
                req_dao.web_dao.web_user.user_dao.account_dao.clone(),
                &param.area_code,
                &param.mobile,
                &param.password,
            ),
            param.captcha.as_ref(),
            &req_dao.user_session,
            Some(&req_dao.req_env),
        )
        .await
}

#[derive(Deserialize)]
pub struct MobileCodeLoginParam {
    area_code: String,
    mobile: String,
    code: String,
    captcha: Option<CaptchaParam>,
}

pub async fn user_login_from_mobile_code(
    param: &MobileCodeLoginParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<(UserAuthToken, ShowUserAuthData)> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, None, &CheckSystemLogin {})
        .await?;
    req_dao
        .web_dao
        .web_user
        .auth
        .user_login(
            &MobileCodeLogin::new(
                req_dao.web_dao.redis.clone(),
                req_dao.web_dao.web_user.user_dao.account_dao.clone(),
                &param.area_code,
                &param.mobile,
                &param.code,
            ),
            param.captcha.as_ref(),
            &req_dao.user_session,
            Some(&req_dao.req_env),
        )
        .await
}

#[derive(Deserialize)]
pub struct MobileSendCodeLoginParam {
    area_code: String,
    mobile: String,
    captcha: CaptchaParam,
}

pub async fn user_login_mobile_send_code(
    param: &MobileSendCodeLoginParam,
    req_dao: &RequestDao,
) -> JsonResult<JsonData> {
    let valid_code = req_dao
        .web_dao
        .app_captcha
        .valid_code(&crate::dao::CaptchaKey::LoginSmsCode);
    valid_code
        .check_code(&param.captcha.borrow().into())
        .await?;
    let data = MobileCodeLogin::valid_code_set(
        req_dao.web_dao.redis.clone(),
        &mut EmailCodeLogin::valid_code_builder(),
        &param.area_code,
        &param.mobile,
    )
    .await?;
    req_dao
        .web_dao
        .app_sender
        .smser
        .send_valid_code(
            &param.area_code,
            &param.mobile,
            &data.0,
            &data.1,
            Some(&req_dao.req_env),
        )
        .await?;
    let _ = valid_code
        .clear_code(
            &param.captcha.key,
            &mut req_dao.web_dao.app_captcha.valid_code_builder(),
        )
        .await;
    Ok(JsonData::data(json!({ "ttl": data.1 })))
}

#[derive(Deserialize)]
pub struct EmailSendCodeLoginParam {
    email: String,
    captcha: CaptchaParam,
}

pub async fn user_login_email_send_code(
    param: &EmailSendCodeLoginParam,
    req_dao: &RequestDao,
) -> JsonResult<JsonData> {
    let valid_code = req_dao
        .web_dao
        .app_captcha
        .valid_code(&crate::dao::CaptchaKey::LoginEmailCode);
    valid_code
        .check_code(&param.captcha.borrow().into())
        .await?;
    let data = EmailCodeLogin::valid_code_set(
        req_dao.web_dao.redis.clone(),
        &mut EmailCodeLogin::valid_code_builder(),
        &param.email,
    )
    .await?;
    req_dao
        .web_dao
        .app_sender
        .mailer
        .send_valid_code(&param.email, &data.0, &data.1, Some(&req_dao.req_env))
        .await?;
    let _ = valid_code
        .clear_code(
            &param.captcha.key,
            &mut req_dao.web_dao.app_captcha.valid_code_builder(),
        )
        .await;
    Ok(JsonData::data(json!({ "ttl": data.1 })))
}
