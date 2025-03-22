use serde::Deserialize;
use serde_json::json;

use crate::{
    common::{CaptchaParam, JsonData, JsonResult, RequestDao},
    dao::{
        access::common::CheckSystemRegister, RegFromEmailData, RegFromMobileData, RegFromNameData,
        RegSendCodeFromEmailData, RegSendCodeFromMobileData,
    },
};

#[derive(Debug, Deserialize)]
pub struct RegFromNameParam {
    pub nikename: Option<String>,
    pub name: String,
    pub password: String,
}

pub async fn user_reg_from_name(
    param: &RegFromNameParam,
    req_dao: &RequestDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env,None,&CheckSystemRegister {}, )
        .await?;
    let user = req_dao
        .web_dao
        .web_user
        .auth
        .user_reg_from_name(
            &RegFromNameData {
                nikename: param.nikename.as_deref(),
                name: &param.name,
                password: &param.password,
            },
            0,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::data(json!({
        "id":user,
    })))
}
#[derive(Debug, Deserialize)]
pub struct RegSendCodeFromMobileParam {
    pub mobile: String,
    pub area_code: String,
    pub captcha: CaptchaParam,
}

pub async fn user_reg_send_code_from_mobile(
    param: &RegSendCodeFromMobileParam,
    req_dao: &RequestDao,
) -> JsonResult<JsonData> {
    let ttl = req_dao
        .web_dao
        .web_user
        .auth
        .user_reg_send_code_from_mobile(
            &RegSendCodeFromMobileData {
                mobile: &param.mobile,
                area_code: &param.area_code,
                captcha: &param.captcha,
            },
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::data(json!({ "ttl": ttl})))
}
#[derive(Debug, Deserialize)]
pub struct RegSendCodeFromEmailParam {
    pub email: String,
    pub captcha: CaptchaParam,
}
pub async fn user_reg_send_code_from_email(
    param: &RegSendCodeFromEmailParam,
    req_dao: &RequestDao,
) -> JsonResult<JsonData> {
    let ttl = req_dao
        .web_dao
        .web_user
        .auth
        .user_reg_send_code_from_email(
            &RegSendCodeFromEmailData {
                email: &param.email,
                captcha: &param.captcha,
            },
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::data(json!({ "ttl": ttl})))
}

#[derive(Deserialize)]
pub struct RegFromEmailParam {
    pub email: String,
    pub code: String,
    pub password: String,
    pub nikename: String,
}

pub async fn user_reg_from_email(
    param: &RegFromEmailParam,
    req_dao: &RequestDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env,None,&CheckSystemRegister {})
        .await?;
    let id = req_dao
        .web_dao
        .web_user
        .auth
        .user_reg_from_email(
            &RegFromEmailData {
                email: &param.email,
                code: &param.code,
                password: &param.password,
                nikename: &param.nikename,
            },
            0,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::data(json!({ "id": id})))
}

#[derive(Deserialize)]
pub struct RegFromMobileParam {
    pub mobile: String,
    pub area_code: String,
    pub code: String,
    pub password: String,
    pub nikename: String,
}

pub async fn user_reg_from_mobile(
    param: &RegFromMobileParam,
    req_dao: &RequestDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env,None,&CheckSystemRegister {})
        .await?;
    let id = req_dao
        .web_dao
        .web_user
        .auth
        .user_reg_from_mobile(
            &RegFromMobileData {
                mobile: &param.mobile,
                area_code: &param.area_code,
                code: &param.code,
                password: &param.password,
                nikename: &param.nikename,
            },
            0,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::data(json!({ "id": id})))
}
