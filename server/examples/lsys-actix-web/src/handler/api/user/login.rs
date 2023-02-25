use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};

use actix_web::web::Data;
use actix_web::{get, post};

use lsys_web::dao::WebDao;
use lsys_web::handler::api::login::login_data_from_user_auth;
use lsys_web::handler::api::login::user_login_from_email;
use lsys_web::handler::api::login::user_login_from_email_code;
use lsys_web::handler::api::login::user_login_from_mobile;
use lsys_web::handler::api::login::user_login_from_mobile_code;
use lsys_web::handler::api::login::user_login_from_name;
use lsys_web::handler::api::login::user_login_mobile_send_code;
use lsys_web::handler::api::login::EmailCodeLoginParam;
use lsys_web::handler::api::login::EmailLoginParam;
use lsys_web::handler::api::login::EmailSendCodeLoginParam;
use lsys_web::handler::api::login::MobileCodeLoginParam;
use lsys_web::handler::api::login::MobileLoginParam;
use lsys_web::handler::api::login::MobileSendCodeLoginParam;
use lsys_web::handler::api::login::NameLoginParam;
use lsys_web::handler::api::login::UserAuthDataOptionParam;
use lsys_web::handler::api::login::{user_external_login_callback, user_login_email_send_code};
use lsys_web::handler::api::user::user_login_history;
use lsys_web::handler::api::user::user_logout;
use lsys_web::handler::api::user::LoginHistoryParam;
use lsys_web::JsonData;

use lsys_web::handler::oauth::user::user_external_login_url;
use lsys_web_module_oauth::module::{WechatCallbackParam, WechatLogin, WechatLoginParam};
use serde::Deserialize;
use serde_json::json;

#[post("/login/{type}")]
pub(crate) async fn login<'t>(
    path: actix_web::web::Path<(String,)>,
    rest: JsonQuery,
    auth_dao: UserAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    let res = match path.0.to_string().as_str() {
        "name" => user_login_from_name(rest.param::<NameLoginParam>()?, &auth_dao).await,
        "sms" => user_login_from_mobile(rest.param::<MobileLoginParam>()?, &auth_dao).await,
        "email" => user_login_from_email(rest.param::<EmailLoginParam>()?, &auth_dao).await,
        "sms-send-code" => {
            user_login_mobile_send_code(rest.param::<MobileSendCodeLoginParam>()?, &auth_dao).await
        }
        "sms-code" => {
            user_login_from_mobile_code(rest.param::<MobileCodeLoginParam>()?, &auth_dao).await
        }
        "email-send-code" => {
            user_login_email_send_code(rest.param::<EmailSendCodeLoginParam>()?, &auth_dao).await
        }
        "email-code" => {
            user_login_from_email_code(rest.param::<EmailCodeLoginParam>()?, &auth_dao).await
        }
        name => handler_not_found!(name),
    };
    Ok(res?.into())
}

#[post("/login_data")]
pub(crate) async fn user_data<'t>(
    jwt: JwtQuery,
    auth_dao: UserAuthQuery,
    rest: JsonQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    Ok(
        login_data_from_user_auth(rest.param::<UserAuthDataOptionParam>()?, &auth_dao)
            .await?
            .into(),
    )
}

#[post("/login_history")]
pub async fn login_history<'t>(
    auth_dao: UserAuthQuery,
    rest: JsonQuery,
    jwt: JwtQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    let res = user_login_history(rest.param::<LoginHistoryParam>()?, &auth_dao).await;
    Ok(res?.into())
}

#[get("/logout")]
pub async fn logout<'t>(
    jwt: JwtQuery,
    auth_dao: UserAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    Ok(user_logout(&auth_dao).await?.into())
}

//------------------------外部OAUTH登录------------------------

#[derive(Debug, Deserialize)]
pub struct ExternalLoginParam {
    pub login_type: String,
    pub login_callback: String,
    pub login_state: String,
}
//获取外部登录URL地址
#[post("/external_login_url")]
pub async fn external_login_url(
    rest: JsonQuery,
    app_dao: Data<WebDao>,
) -> ResponseJsonResult<ResponseJson> {
    let login_param = rest.param::<ExternalLoginParam>()?;
    let res = match login_param.login_type.as_str() {
        "wechat" => {
            user_external_login_url::<WechatLogin, _, _, _>(
                "wechat",
                &app_dao,
                &WechatLoginParam {
                    state: login_param.login_state,
                    callback_url: login_param.login_callback,
                },
            )
            .await
        }
        name => handler_not_found!(name),
    };
    Ok(res?.into())
}

#[derive(Debug, Deserialize)]
pub struct ExternalLoginStateCheckParam {
    pub login_type: String,
    pub login_state: String,
}
//扫码登录检测是否已经完成登录
#[post("/external_state_check")]
pub async fn external_state_check(
    rest: JsonQuery,
    auth_dao: UserAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    let login_param = rest.param::<ExternalLoginStateCheckParam>()?;
    let res = match login_param.login_type.as_str() {
        "wechat" => {
            let wechat = &auth_dao
                .web_dao
                .user
                .user_external_oauth::<WechatLogin, WechatLoginParam, _, _>("wechat")
                .await
                .map_err(JsonData::from)?;
            let (reload, login_data) = wechat
                .state_check(&auth_dao.web_dao.user, &login_param.login_state)
                .await?;
            if let Some(ldat) = login_data {
                user_external_login_callback::<WechatLogin, WechatLoginParam, _, _>(
                    "wechat", &auth_dao, &ldat,
                )
                .await
            } else {
                Ok(JsonData::data(json!({ "reload": reload })))
            }
        }
        name => handler_not_found!(name),
    };
    Ok(res?.into())
}

#[derive(Debug, Deserialize)]
pub struct ExternalLoginStateCallbackParam {
    pub login_type: String,
    pub code: String,
    pub callback_state: String,
}
//APP端完成扫码登录后
//请求此回调地址完成登录操作
#[post("/external_state_callback")]
pub async fn external_state_callback(
    rest: JsonQuery,
    app_dao: Data<WebDao>,
) -> ResponseJsonResult<ResponseJson> {
    let login_param = rest.param::<ExternalLoginStateCallbackParam>()?;
    let res = match login_param.login_type.as_str() {
        "wechat" => {
            let wechat = &app_dao
                .user
                .user_external_oauth::<WechatLogin, WechatLoginParam, _, _>("wechat")
                .await
                .map_err(JsonData::from)?;
            wechat
                .state_callback(
                    &app_dao.user,
                    &WechatCallbackParam {
                        code: login_param.code,
                        state: login_param.callback_state,
                    },
                )
                .await
        }
        name => handler_not_found!(name),
    };
    Ok(res?.into())
}

//外部登录完成回调地址,不包含扫码登录,目前没用到
#[derive(Debug, Deserialize)]
pub struct ExternalCallbackParam {
    pub login_type: String,
    pub code: String,
    pub callback_state: String,
}

#[post("/external_login_callback")]
pub async fn external_login_callback<'t>(
    rest: JsonQuery,
    _auth_dao: UserAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    let login_param = rest.param::<ExternalCallbackParam>()?;
    let res = match login_param.login_type.as_str() {
        "qq" => {
            Ok(JsonData::message("未实现"))
            // user_external_login_callback::<WechatLogin, WechatLoginParam, _, _>(
            //     "wechat", &auth_dao, &param,
            // )
            // .await
        }
        name => handler_not_found!(name),
    };
    Ok(res?.into())
}
