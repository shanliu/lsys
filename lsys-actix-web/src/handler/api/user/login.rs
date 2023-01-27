use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};

use actix_web::web::Data;
use actix_web::{get, post};

use lsys_web::dao::WebDao;
use lsys_web::handler::api::login::user_login_email_send_code;
use lsys_web::handler::api::login::user_login_from_email;
use lsys_web::handler::api::login::user_login_from_email_code;
use lsys_web::handler::api::login::user_login_from_mobile;
use lsys_web::handler::api::login::user_login_from_mobile_code;
use lsys_web::handler::api::login::user_login_from_name;
use lsys_web::handler::api::login::user_login_mobile_send_code;
use lsys_web::handler::api::login::user_oauth_callback;
use lsys_web::handler::api::login::user_oauth_login;
use lsys_web::handler::api::login::EmailCodeLoginParam;
use lsys_web::handler::api::login::EmailLoginParam;
use lsys_web::handler::api::login::EmailSendCodeLoginParam;
use lsys_web::handler::api::login::MobileCodeLoginParam;
use lsys_web::handler::api::login::MobileLoginParam;
use lsys_web::handler::api::login::MobileSendCodeLoginParam;
use lsys_web::handler::api::login::NameLoginParam;
use lsys_web::handler::api::login::UserAuthDataOptionParam;
use lsys_web::handler::api::login::{login_data_from_user_auth, user_oauth};
use lsys_web::handler::api::user::user_login_history;
use lsys_web::handler::api::user::user_logout;
use lsys_web::handler::api::user::LoginHistoryParam;
use lsys_web::JsonData;

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

#[derive(Debug, Deserialize)]
pub struct OauthLoginParam {
    pub login_type: String,
    pub login_callback: String,
    pub login_state: String,
}

#[post("/oauth_login")]
pub async fn oauth_login(
    rest: JsonQuery,
    app_dao: Data<WebDao>,
) -> ResponseJsonResult<ResponseJson> {
    let login_param = rest.param::<OauthLoginParam>()?;
    let res = match login_param.login_type.as_str() {
        "wechat" => {
            user_oauth_login::<WechatLogin, _, _, _>(
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
pub struct OauthCheckParam {
    pub login_type: String,
    pub login_state: String,
}

#[post("/oauth_qrcode_check")]
pub async fn oauth_qrcode_check(
    rest: JsonQuery,
    auth_dao: UserAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    let login_param = rest.param::<OauthCheckParam>()?;
    let res = match login_param.login_type.as_str() {
        "wechat" => {
            let wechat =
                user_oauth::<WechatLogin, WechatLoginParam, _, _>("wechat", &auth_dao.web_dao)
                    .await?;
            let (reload, login_data) = wechat
                .get_state_login_data(&auth_dao.web_dao.user, &login_param.login_state)
                .await?;
            if let Some(ldat) = login_data {
                user_oauth_callback::<WechatLogin, WechatLoginParam, _, _>(
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
pub struct OauthDoLoginParam {
    pub login_type: String,
    pub login_data: String,
}

#[post("/oauth_qrcode_do_login")]
pub async fn oauth_qrcode_do_login(
    rest: JsonQuery,
    app_dao: Data<WebDao>,
) -> ResponseJsonResult<ResponseJson> {
    let login_param = rest.param::<OauthDoLoginParam>()?;
    let res = match login_param.login_type.as_str() {
        "wechat" => {
            let wechat =
                user_oauth::<WechatLogin, WechatLoginParam, _, _>("wechat", &app_dao).await?;
            wechat
                .set_state_login_data(&app_dao.user, "wechat", &login_param.login_data)
                .await
        }
        name => handler_not_found!(name),
    };
    Ok(res?.into())
}

#[derive(Debug, Deserialize)]
pub struct OauthCallbackParam {
    pub login_type: String,
    pub code: String,
    pub callback_state: String,
}

#[post("/oauth_callback")]
pub async fn oauth_callback<'t>(
    rest: JsonQuery,
    auth_dao: UserAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    let login_param = rest.param::<OauthCallbackParam>()?;
    let res = match login_param.login_type.as_str() {
        "wechat" => {
            let param = WechatCallbackParam {
                code: login_param.code,
                state: login_param.callback_state,
            };
            user_oauth_callback::<WechatLogin, WechatLoginParam, _, _>("wechat", &auth_dao, &param)
                .await
        }
        name => handler_not_found!(name),
    };
    Ok(res?.into())
}
