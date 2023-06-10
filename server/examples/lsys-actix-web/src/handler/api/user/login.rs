use crate::common::handler::{
    JsonQuery, JwtClaims, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};

use actix_web::web::Data;
use actix_web::{get, post};

use jsonwebtoken::{encode, EncodingKey, Header};
use lsys_user::dao::auth::UserAuthTokenData;
use lsys_web::dao::user::ShowUserAuthData;
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
use lsys_web::{JsonData, JsonResult};

use lsys_web::handler::oauth::user_external_login_url;
use lsys_web_module_oauth::module::{WechatCallbackParam, WechatLogin, WechatLoginParam};
use serde::Deserialize;
use serde_json::json;

async fn jwt_login_data(
    auth_dao: &UserAuthQuery,
    token: UserAuthTokenData,
    data: ShowUserAuthData,
) -> JsonResult<JsonData> {
    let app_jwt_key = auth_dao
        .web_dao
        .app_core
        .config
        .get_string("app_jwt_key")
        .unwrap_or_default();
    let token = encode(
        &Header::default(),
        &JwtClaims::new(token.time_out as i64, token.to_string(), Some(json!(data))),
        &EncodingKey::from_secret(app_jwt_key.as_bytes()),
    )
    .map_err(JsonData::message_error)?;
    let passwrod_timeout = auth_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user_password
        .password_timeout(&data.user_password_id)
        .await
        .unwrap_or(false);
    Ok(JsonData::data(json!({
        "auth_data":data,
        "jwt":token,
        "passwrod_timeout":passwrod_timeout,
    })))
}

#[post("/login/{type}")]
pub(crate) async fn login<'t>(
    path: actix_web::web::Path<(String,)>,
    json_param: JsonQuery,
    auth_dao: UserAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    let res = match path.0.to_string().as_str() {
        "sms-send-code" => {
            user_login_mobile_send_code(json_param.param::<MobileSendCodeLoginParam>()?, &auth_dao)
                .await
        }
        "email-send-code" => {
            user_login_email_send_code(json_param.param::<EmailSendCodeLoginParam>()?, &auth_dao)
                .await
        }
        e => {
            let (token, data) = match e {
                "name" => {
                    user_login_from_name(json_param.param::<NameLoginParam>()?, &auth_dao).await
                }
                "sms" => {
                    user_login_from_mobile(json_param.param::<MobileLoginParam>()?, &auth_dao).await
                }
                "email" => {
                    user_login_from_email(json_param.param::<EmailLoginParam>()?, &auth_dao).await
                }
                "sms-code" => {
                    user_login_from_mobile_code(
                        json_param.param::<MobileCodeLoginParam>()?,
                        &auth_dao,
                    )
                    .await
                }
                "email-code" => {
                    user_login_from_email_code(
                        json_param.param::<EmailCodeLoginParam>()?,
                        &auth_dao,
                    )
                    .await
                }
                name => handler_not_found!(name),
            }?;
            jwt_login_data(&auth_dao, token, data).await
        }
    };
    Ok(res?.into())
}

#[post("/login_data")]
pub(crate) async fn user_data<'t>(
    jwt: JwtQuery,
    auth_dao: UserAuthQuery,
    json_param: JsonQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    Ok(
        login_data_from_user_auth(json_param.param::<UserAuthDataOptionParam>()?, &auth_dao)
            .await?
            .into(),
    )
}

#[post("/login_history")]
pub async fn login_history<'t>(
    auth_dao: UserAuthQuery,
    json_param: JsonQuery,
    jwt: JwtQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    let res = user_login_history(json_param.param::<LoginHistoryParam>()?, &auth_dao).await;
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
    json_param: JsonQuery,
    app_dao: Data<WebDao>,
) -> ResponseJsonResult<ResponseJson> {
    let login_param = json_param.param::<ExternalLoginParam>()?;
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
    json_param: JsonQuery,
    auth_dao: UserAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    let login_param = json_param.param::<ExternalLoginStateCheckParam>()?;
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
                let (token, data) =
                    user_external_login_callback::<WechatLogin, WechatLoginParam, _, _>(
                        "wechat", &auth_dao, &ldat,
                    )
                    .await?;
                jwt_login_data(&auth_dao, token, data).await
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
    json_param: JsonQuery,
    app_dao: Data<WebDao>,
) -> ResponseJsonResult<ResponseJson> {
    let login_param = json_param.param::<ExternalLoginStateCallbackParam>()?;
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
    json_param: JsonQuery,
    _auth_dao: UserAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    let login_param = json_param.param::<ExternalCallbackParam>()?;
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
