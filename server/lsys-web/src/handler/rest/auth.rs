use std::collections::HashMap;

use crate::common::{JsonData, JsonError, JsonFluent};
use crate::dao::access::RbacAccessCheckEnv;
use crate::{
    common::{JsonResponse, JsonResult, RequestDao},
    dao::access::rest::CheckRestApp,
};
use lsys_access::dao::{AccessError, AccessLoginData};
use lsys_app::model::AppModel;
use lsys_core::{fluent_message, now_time};
use lsys_mfa::dao::MfaSubject;
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Debug, Deserialize)]
pub struct DoLoginParam {
    pub token_code: String,
    pub user_data: String,
    pub user_nickname: String,
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub expire_time: u64,
    pub device_name: Option<String>,
    pub user_account: Option<String>,
    pub login_ip: Option<String>,
    pub device_id: Option<String>,
    pub session_data: Option<HashMap<String, Value>>,
}
pub async fn do_login(
    param: &DoLoginParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    let app_user = req_dao
        .web_dao
        .web_access
        .access_dao
        .user
        .cache()
        .find_by_id(&app.user_id)
        .await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::user(&app_user, &req_dao.req_env),
            &CheckRestApp {},
        )
        .await?;
    req_dao
        .web_dao
        .web_app
        .app_dao
        .exter_login
        .inner_feature_exter_login_check(app)
        .await?;

    let session_data = param
        .session_data
        .as_ref()
        .map(|t| {
            t.iter()
                .map(|(k, v)| (k, v.to_string()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let seession_body = match req_dao
        .web_dao
        .web_user
        .user_dao
        .auth_code_dao
        .code_login(
            app.id,
            &param.token_code,
            &param.user_data,
            &param.user_nickname,
            &AccessLoginData {
                user_account: param.user_account.as_deref(),
                login_ip: param.login_ip.as_deref(),
                device_id: param.device_id.as_deref(),
                device_name: param.device_name.as_deref(),
                expire_time: now_time().unwrap_or_default() + param.expire_time,
                session_data: session_data
                    .iter()
                    .map(|e| (e.0.as_str(), e.1.as_str()))
                    .collect::<Vec<_>>(),
            },
        )
        .await
    {
        Ok(t) => t,
        Err(err) => match err {
            lsys_user::dao::AccountError::AccessError(err @ AccessError::LoginTokenDataExit(_)) => {
                return Err(JsonError::JsonResponse(
                    err.to_json_data(&req_dao.fluent),
                    fluent_message!("access-token-data-token-code-exits"),
                ))
            }
            err => Err(err)?,
        },
    };
    Ok(JsonResponse::data(JsonData::body(json!({
        "token_data": seession_body.token_data(),
        "user_id": seession_body.user_id(),
        "user_nickname": seession_body.user().user_nickname,
    }))))
}

#[derive(Debug, Deserialize)]
pub struct DoLogoutParam {
    token_data: String,
}
pub async fn do_logout(
    param: &DoLogoutParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    req_dao
        .web_dao
        .web_app
        .app_dao
        .exter_login
        .inner_feature_exter_login_check(app)
        .await?;
    req_dao
        .web_dao
        .web_user
        .user_dao
        .auth_code_dao
        .code_logout(app.id, &param.token_data)
        .await?;

    Ok(JsonResponse::default())
}

#[derive(Debug, Deserialize)]
pub struct LoginInfoParam {
    pub token_data: String,
}
pub async fn login_info(
    param: &LoginInfoParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    let app_user = req_dao
        .web_dao
        .web_access
        .access_dao
        .user
        .cache()
        .find_by_id(&app.user_id)
        .await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::user(&app_user, &req_dao.req_env),
            &CheckRestApp {},
        )
        .await?;
    req_dao
        .web_dao
        .web_app
        .app_dao
        .exter_login
        .inner_feature_exter_login_check(app)
        .await?;

    let session = req_dao
        .web_dao
        .web_user
        .user_dao
        .auth_code_dao
        .login_data(app.id, &param.token_data)
        .await?;

    Ok(JsonResponse::data(JsonData::body(json!({
        "session":session.session(),
        "user":session.user()
    }))))
}

/// 检查一批账号是否启用MFA验证
#[derive(Debug, Deserialize)]
pub struct MfaIsEnabledParam {
    /// 账号列表，格式为 Vec<user_data>
    pub accounts: Vec<String>,
}

pub async fn mfa_is_enabled(
    param: &MfaIsEnabledParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    let app_user = req_dao
        .web_dao
        .web_access
        .access_dao
        .user
        .cache()
        .find_by_id(&app.user_id)
        .await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::user(&app_user, &req_dao.req_env),
            &CheckRestApp {},
        )
        .await?;

    // 创建要检查的MfaSubject列表
    let subjects: Vec<MfaSubject> = param
        .accounts
        .iter()
        .map(|user_data| MfaSubject {
            app_id: app.id,
            user_data: user_data.clone(),
        })
        .collect();

    // 批量检查MFA是否启用
    let enabled_list = req_dao
        .web_dao
        .web_mfa
        .totp_dao
        .is_enabled_batch(&subjects)
        .await?;

    // 构建响应，将账号与启用状态对应
    let result: Vec<Value> = param
        .accounts
        .iter()
        .zip(enabled_list.iter())
        .map(|(account, enabled)| {
            json!({
                "account": account,
                "enabled": *enabled
            })
        })
        .collect();

    Ok(JsonResponse::data(JsonData::body(json!({
        "accounts": result
    }))))
}

/// 为指定账号生成MFA绑定
#[derive(Debug, Deserialize)]
pub struct MfaEnableParam {
    pub user_data: String,
    /// Base32 编码的密钥
    pub secret: String,
}

pub async fn mfa_enable(
    param: &MfaEnableParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    let app_user = req_dao
        .web_dao
        .web_access
        .access_dao
        .user
        .cache()
        .find_by_id(&app.user_id)
        .await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::user(&app_user, &req_dao.req_env),
            &CheckRestApp {},
        )
        .await?;

    let subject = MfaSubject {
        app_id: app.id,
        user_data: param.user_data.clone(),
    };

    // 启用新的TOTP密钥
    let record_id = req_dao
        .web_dao
        .web_mfa
        .totp_dao
        .enable_new_secret(&subject, &param.secret)
        .await?;

    Ok(JsonResponse::data(JsonData::body(json!({
        "record_id": record_id,
        "user_data": param.user_data,
    }))))
}

/// 对指定账号进行MFA验证
#[derive(Debug, Deserialize)]
pub struct MfaVerifyParam {
    pub user_data: String,
    /// TOTP验证码 (6-8位数字)
    pub code: String,
}

pub async fn mfa_verify(
    param: &MfaVerifyParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    let app_user = req_dao
        .web_dao
        .web_access
        .access_dao
        .user
        .cache()
        .find_by_id(&app.user_id)
        .await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::user(&app_user, &req_dao.req_env),
            &CheckRestApp {},
        )
        .await?;

    let subject = MfaSubject {
        app_id: app.id,
        user_data: param.user_data.clone(),
    };

    // 执行MFA验证
    req_dao
        .web_dao
        .web_mfa
        .totp_dao
        .verify_totp(&subject, &param.code)
        .await?;

    Ok(JsonResponse::data(JsonData::body(json!({
        "user_data": param.user_data,
    }))))
}
