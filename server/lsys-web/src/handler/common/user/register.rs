use lsys_core::fluent_message;
use lsys_user::model::{UserEmailStatus, UserInfoModelRef, UserMobileStatus};
use serde::Deserialize;
use serde_json::json;
use sqlx_model::model_option_set;

use crate::{
    dao::{user::UserRegData, RequestAuthDao},
    {CaptchaParam, JsonData, JsonResult},
};
use lsys_user::dao::auth::{SessionData, SessionTokenData, UserSession};
#[derive(Deserialize, Default)]
pub struct RegFromNameParam {
    pub nikename: Option<String>,
    pub name: String,
    pub password: String,
}
pub async fn user_reg_from_name<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: RegFromNameParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let reg_ip = req_dao.req_env.request_ip.clone().unwrap_or_default();
    let info = model_option_set!(UserInfoModelRef,{
        reg_ip:  reg_ip,
    });
    let user = req_dao
        .web_dao
        .user
        .reg_user(
            UserRegData {
                nikename: param.nikename.unwrap_or_else(|| param.name.clone()),
                passwrod: Some(param.password),
                name: Some(param.name),
                email: None,
                mobile: None,
                external: None,
                info: Some(info),
            },
            Some(&req_dao.req_env),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::data(json!({
        "id":user.id,
    })))
}

#[derive(Debug, Deserialize)]
pub struct RegSendCodeFromMobileParam {
    pub mobile: String,
    pub area_code: String,
    pub captcha: CaptchaParam,
}
pub async fn user_reg_send_code_from_mobile<
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    param: RegSendCodeFromMobileParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let valid_code = req_dao
        .web_dao
        .captcha
        .valid_code(&crate::dao::CaptchaKey::RegSms);
    valid_code
        .check_code(&param.captcha.key, &param.captcha.code)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let mobile_res = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user_mobile
        .find_by_last_mobile(param.area_code.clone(), param.mobile.clone())
        .await;
    if let Ok(mobile) = mobile_res {
        if UserMobileStatus::Valid.eq(mobile.status) {
            return Ok(
                req_dao
                    .fluent_json_data(fluent_message!("reg-mobile-registered"))
                    .set_sub_code("mobile_is_reg"), // JsonData::message("this mobile is registered")
            );
        }
    }
    let data = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user_mobile
        .valid_code_set(
            &mut req_dao
                .web_dao
                .user
                .user_dao
                .user_account
                .user_mobile
                .valid_code_builder(),
            &param.area_code,
            &param.mobile,
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
pub struct RegSendCodeFromEmailParam {
    pub email: String,
    pub captcha: CaptchaParam,
}
pub async fn user_reg_send_code_from_email<
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    param: RegSendCodeFromEmailParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let valid_code = req_dao
        .web_dao
        .captcha
        .valid_code(&crate::dao::CaptchaKey::RegEmail);
    let email_res = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user_email
        .find_by_last_email(param.email.clone())
        .await;
    if let Ok(email) = email_res {
        if UserEmailStatus::Valid.eq(email.status) {
            return Ok(req_dao
                .fluent_json_data(fluent_message!("reg-mobile-registered"))
                .set_sub_code("mobile_is_reg"));
        }
    }
    valid_code
        .check_code(&param.captcha.key, &param.captcha.code)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let data = req_dao
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
            &0,
            &param.email,
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
        .clear_code(&valid_code, &param.email)
        .await;

    Ok(JsonData::data(json!({ "ttl": data.1 })))
}

#[derive(Deserialize)]
pub struct RegFromEmailParam {
    pub email: String,
    pub code: String,
    pub password: String,
    pub nikename: String,
}

pub async fn user_reg_from_email<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: RegFromEmailParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user_email
        .valid_code_check(&param.code, &0, &param.email)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let reg_ip = req_dao.req_env.request_ip.clone().unwrap_or_default();
    let info = model_option_set!(UserInfoModelRef,{
        reg_ip:reg_ip,
    });
    let user = req_dao
        .web_dao
        .user
        .reg_user(
            UserRegData {
                nikename: param.nikename.clone(),
                passwrod: Some(param.password),
                name: None,
                email: Some((param.email.clone(), UserEmailStatus::Valid)),
                mobile: None,
                external: None,
                info: Some(info),
            },
            Some(&req_dao.req_env),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let _ = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user_email
        .valid_code_clear(&0, &param.email)
        .await;
    Ok(JsonData::data(json!({
        "id":user.id,
    })))
}

#[derive(Deserialize)]
pub struct RegFromMobileParam {
    pub mobile: String,
    pub area_code: String,
    pub code: String,
    pub password: String,
    pub nikename: String,
}

pub async fn user_reg_from_mobile<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: RegFromMobileParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user_mobile
        .valid_code_check(&param.code, &param.area_code, &param.mobile)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let reg_op = req_dao.req_env.request_ip.clone().unwrap_or_default();
    let info = model_option_set!(UserInfoModelRef,{
        reg_ip:reg_op,
    });
    let user = req_dao
        .web_dao
        .user
        .reg_user(
            UserRegData {
                nikename: param.nikename.clone(),
                passwrod: Some(param.password),
                name: None,
                email: None,
                mobile: Some((
                    param.area_code.clone(),
                    param.mobile.clone(),
                    UserMobileStatus::Valid,
                )),
                external: None,
                info: Some(info),
            },
            Some(&req_dao.req_env),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let _ = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user_mobile
        .valid_code_clear(&param.area_code, &param.mobile)
        .await;
    Ok(JsonData::data(json!({
        "id":user.id,
    })))
}
