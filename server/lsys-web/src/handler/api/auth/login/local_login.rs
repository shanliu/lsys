use std::borrow::Borrow;

use crate::dao::access::RbacAccessCheckEnv;
use crate::{
    common::{CaptchaParam, JsonData, JsonResponse, JsonResult, RequestDao, UserAuthQueryDao},
    dao::{ShowUserAuthData, WebDao, access::api::system::auth::CheckSystemLogin},
};
use lsys_access::dao::{AccessSession, SessionBody};
use lsys_core::fluents::IntoFluentMessage;
use lsys_user::dao::UserAuthToken;
use lsys_user::dao::login::{EmailCodeLogin, EmailLogin, MobileCodeLogin, MobileLogin, NameLogin};
use serde::Deserialize;
use serde_json::json;
use tracing::warn;

pub async fn user_login_finish(
    session_body: SessionBody,
    req_dao: &RequestDao,
    auth_dao: &UserAuthQueryDao,
    web_dao: &WebDao,
) -> JsonResult<(UserAuthToken, ShowUserAuthData)> {
    web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&session_body, &req_dao.req_env),
            &CheckSystemLogin {},
        )
        .await?;

    let user_token = UserAuthToken::new(
        session_body.session().user_app_id,
        session_body.token_data(),
        session_body.user_id(),
        session_body.session().expire_time,
    );
    auth_dao
        .user_session
        .write()
        .await
        .set_session_token(user_token.to_owned());
    let auth_data = auth_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await?;
    Ok((
        user_token,
        web_dao
            .web_user
            .auth
            .create_show_account_auth_data(&auth_data)
            .await?,
    ))
}

#[derive(Deserialize)]
pub struct NameLoginParam {
    name: String,
    password: String,
    captcha: Option<CaptchaParam>,
}

pub async fn user_login_from_name(
    param: &NameLoginParam,
    req_dao: &RequestDao,
    auth_dao: &UserAuthQueryDao,
    web_dao: &WebDao,
) -> JsonResult<(UserAuthToken, ShowUserAuthData)> {
    let session_body = web_dao
        .web_user
        .auth
        .user_login(
            &NameLogin::new(
                web_dao.web_user.user_dao.account_dao.clone(),
                &param.name,
                &param.password,
            )
            .await?,
            param.captcha.as_ref(),
            Some(&req_dao.req_env),
        )
        .await?;
    user_login_finish(session_body, req_dao, auth_dao, web_dao).await
}

#[derive(Deserialize)]
pub struct EmailLoginParam {
    email: String,
    password: String,
    captcha: Option<CaptchaParam>,
}
pub async fn user_login_from_email(
    param: &EmailLoginParam,
    req_dao: &RequestDao,
    auth_dao: &UserAuthQueryDao,
    web_dao: &WebDao,
) -> JsonResult<(UserAuthToken, ShowUserAuthData)> {
    let session_body = web_dao
        .web_user
        .auth
        .user_login(
            &EmailLogin::new(
                web_dao.web_user.user_dao.account_dao.clone(),
                &param.email,
                &param.password,
            )
            .await?,
            param.captcha.as_ref(),
            Some(&req_dao.req_env),
        )
        .await?;
    user_login_finish(session_body, req_dao, auth_dao, web_dao).await
}

#[derive(Deserialize)]
pub struct EmailCodeLoginParam {
    email: String,
    code: String,
    captcha: Option<CaptchaParam>,
}

pub async fn user_login_from_email_code(
    param: &EmailCodeLoginParam,
    req_dao: &RequestDao,
    auth_dao: &UserAuthQueryDao,
    web_dao: &WebDao,
) -> JsonResult<(UserAuthToken, ShowUserAuthData)> {
    let session_body = web_dao
        .web_user
        .auth
        .user_login(
            &EmailCodeLogin::new(
                web_dao.redis.clone(),
                web_dao.web_user.user_dao.account_dao.clone(),
                &param.email,
                &param.code,
            )
            .await?,
            param.captcha.as_ref(),
            Some(&req_dao.req_env),
        )
        .await?;
    user_login_finish(session_body, req_dao, auth_dao, web_dao).await
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
    req_dao: &RequestDao,
    auth_dao: &UserAuthQueryDao,
    web_dao: &WebDao,
) -> JsonResult<(UserAuthToken, ShowUserAuthData)> {
    let session_body = web_dao
        .web_user
        .auth
        .user_login(
            &MobileLogin::new(
                web_dao.web_user.user_dao.account_dao.clone(),
                &param.area_code,
                &param.mobile,
                &param.password,
            )
            .await?,
            param.captcha.as_ref(),
            Some(&req_dao.req_env),
        )
        .await?;
    user_login_finish(session_body, req_dao, auth_dao, web_dao).await
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
    req_dao: &RequestDao,
    auth_dao: &UserAuthQueryDao,
    web_dao: &WebDao,
) -> JsonResult<(UserAuthToken, ShowUserAuthData)> {
    let session_body = web_dao
        .web_user
        .auth
        .user_login(
            &MobileCodeLogin::new(
                web_dao.redis.clone(),
                web_dao.web_user.user_dao.account_dao.clone(),
                &param.area_code,
                &param.mobile,
                &param.code,
            )
            .await?,
            param.captcha.as_ref(),
            Some(&req_dao.req_env),
        )
        .await?;
    user_login_finish(session_body, req_dao, auth_dao, web_dao).await
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
    web_dao: &WebDao,
) -> JsonResult<JsonResponse> {
    let valid_code = web_dao
        .app_captcha
        .valid_code(&crate::dao::CaptchaKey::LoginSmsCode);
    valid_code
        .check_code(&param.captcha.borrow().into())
        .await?;
    let data = MobileCodeLogin::valid_code_set(
        web_dao.redis.clone(),
        &mut EmailCodeLogin::valid_code_builder(),
        &param.area_code,
        &param.mobile,
    )
    .await?;
    web_dao
        .app_sender
        .smser
        .send_valid_code(
            "valid_code_login_mobile",
            &param.area_code,
            &param.mobile,
            &data.0,
            &data.1,
            Some(&req_dao.req_env),
        )
        .await?;
    if let Err(e) = valid_code
        .destroy_code(
            &param.captcha.key,
            &mut web_dao.app_captcha.valid_code_builder(),
        )
        .await
    {
        warn!(
            "user_login_mobile_send_code: destroy_code failed: {}",
            e.to_fluent_message().default_format()
        );
    }
    Ok(JsonResponse::data(JsonData::body(json!({ "ttl": data.1 }))))
}

#[derive(Deserialize)]
pub struct EmailSendCodeLoginParam {
    email: String,
    captcha: CaptchaParam,
}

pub async fn user_login_email_send_code(
    param: &EmailSendCodeLoginParam,
    req_dao: &RequestDao,
    web_dao: &WebDao,
) -> JsonResult<JsonResponse> {
    let valid_code = web_dao
        .app_captcha
        .valid_code(&crate::dao::CaptchaKey::LoginEmailCode);
    valid_code
        .check_code(&param.captcha.borrow().into())
        .await?;
    let data = EmailCodeLogin::valid_code_set(
        web_dao.redis.clone(),
        &mut EmailCodeLogin::valid_code_builder(),
        &param.email,
    )
    .await?;
    web_dao
        .app_sender
        .mailer
        .send_valid_code(
            "valid_code_login_email",
            &param.email,
            &data.0,
            &data.1,
            Some(&req_dao.req_env),
        )
        .await?;
    if let Err(e) = valid_code
        .destroy_code(
            &param.captcha.key,
            &mut web_dao.app_captcha.valid_code_builder(),
        )
        .await
    {
        warn!(
            "user_login_email_send_code: destroy_code failed: {}",
            e.to_fluent_message().default_format()
        );
    }
    Ok(JsonResponse::data(JsonData::body(json!({ "ttl": data.1 }))))
}
