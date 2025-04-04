use serde::Deserialize;
use serde_json::json;

use crate::{
    common::{CaptchaParam, JsonData, JsonResult, RequestDao},
    dao::{
        ResetPasswordFromEmailData, ResetPasswordFromMobileData,
        ResetPasswordSendCodeFromEmailData, ResetPasswordSendCodeFromMobileData,
    },
};

#[derive(Debug, Deserialize)]
pub struct ResetPasswordSendCodeFromMobileParam {
    pub mobile: String,
    pub area_code: String,
    pub captcha: CaptchaParam,
}
pub async fn user_reset_password_send_code_from_mobile(
    param: &ResetPasswordSendCodeFromMobileParam,
    req_dao: &RequestDao,
) -> JsonResult<JsonData> {
    let ttl = req_dao
        .web_dao
        .web_user
        .auth
        .user_reset_password_send_code_from_mobile(
            &ResetPasswordSendCodeFromMobileData {
                mobile: &param.mobile,
                area_code: &param.area_code,
                captcha: &param.captcha,
            },
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::data(json!({ "ttl": ttl })))
}
#[derive(Debug, Deserialize)]
pub struct ResetPasswordSendCodeFromEmailParam {
    pub email: String,
    pub captcha: CaptchaParam,
}
pub async fn user_reset_password_send_code_from_email(
    param: &ResetPasswordSendCodeFromEmailParam,
    req_dao: &RequestDao,
) -> JsonResult<JsonData> {
    let ttl = req_dao
        .web_dao
        .web_user
        .auth
        .user_reset_password_send_code_from_email(
            &ResetPasswordSendCodeFromEmailData {
                email: &param.email,
                captcha: &param.captcha,
            },
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::data(json!({ "ttl": ttl })))
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordFromEmailParam {
    pub email: String,
    pub code: String,
    pub new_password: String,
}
pub async fn user_reset_password_from_email(
    param: &ResetPasswordFromEmailParam,
    req_dao: &RequestDao,
) -> JsonResult<JsonData> {
    let id = req_dao
        .web_dao
        .web_user
        .auth
        .user_reset_password_from_email(
            &ResetPasswordFromEmailData {
                email: &param.email,
                code: &param.code,
                new_password: &param.new_password,
            },
            0,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::data(json!({ "id": id })))
}
#[derive(Debug, Deserialize)]
pub struct ResetPasswordFromMobileParam {
    pub area_code: String,
    pub mobile: String,
    pub code: String,
    pub new_password: String,
}
pub async fn user_reset_password_from_mobile(
    param: &ResetPasswordFromMobileParam,
    req_dao: &RequestDao,
) -> JsonResult<JsonData> {
    let id = req_dao
        .web_dao
        .web_user
        .auth
        .user_reset_password_from_mobile(
            &ResetPasswordFromMobileData {
                area_code: &param.area_code,
                mobile: &param.mobile,
                code: &param.code,
                new_password: &param.new_password,
            },
            0,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::data(json!({ "id": id })))
}
