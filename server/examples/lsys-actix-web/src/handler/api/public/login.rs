use crate::common::handler::{
    JsonQuery, JwtClaims, JwtQuery, ReqQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};

use actix_web::post;

use jsonwebtoken::{encode, EncodingKey, Header};
use lsys_web::lsys_core::fluent_message;
use lsys_web::lsys_user::dao::UserAuthToken;

use lsys_web::common::{JsonData, JsonError, JsonResponse, JsonResult};
use lsys_web::dao::ShowUserAuthData;
use lsys_web::handler::api::auth::user_login_email_send_code;
use lsys_web::handler::api::auth::user_login_from_app_code;
use lsys_web::handler::api::auth::user_login_from_email;
use lsys_web::handler::api::auth::user_login_from_email_code;
use lsys_web::handler::api::auth::user_login_from_external;
use lsys_web::handler::api::auth::user_login_from_mobile;
use lsys_web::handler::api::auth::user_login_from_mobile_code;
use lsys_web::handler::api::auth::user_login_from_name;
use lsys_web::handler::api::auth::user_login_mobile_send_code;
use lsys_web::handler::api::auth::AppCodeLoginParam;
use lsys_web::handler::api::auth::EmailCodeLoginParam;
use lsys_web::handler::api::auth::EmailLoginParam;
use lsys_web::handler::api::auth::EmailSendCodeLoginParam;
use lsys_web::handler::api::auth::MobileCodeLoginParam;
use lsys_web::handler::api::auth::MobileLoginParam;
use lsys_web::handler::api::auth::MobileSendCodeLoginParam;
use lsys_web::handler::api::auth::NameLoginParam;
use lsys_web::handler::api::auth::UserAuthDataOptionParam;
use lsys_web::handler::api::auth::{login_data_from_user_auth, user_external_login_url};

use lsys_web_module_oauth::module::{
    WeChatConfig, WechatCallbackParam, WechatLogin, WechatLoginParam, OAUTH_TYPE_WECHAT,
};
use serde::Deserialize;
use serde_json::json;

async fn jwt_login_data(
    auth_dao: &UserAuthQuery,
    token: UserAuthToken,
    data: ShowUserAuthData,
) -> JsonResult<JsonResponse> {
    let app_jwt_key = auth_dao
        .web_dao
        .app_core
        .config
        .find(None)
        .get_string("app_jwt_key")
        .unwrap_or_default();
    let token = encode(
        &Header::default(),
        &JwtClaims::new(token.time_out as i64, token.to_string(), Some(json!(data))),
        &EncodingKey::from_secret(app_jwt_key.as_bytes()),
    )
    .map_err(|e| JsonError::Message(fluent_message!("jwt-encode-error", e)))?;
    let passwrod_timeout = auth_dao
        .web_dao
        .web_user
        .user_dao
        .account_dao
        .account_password
        .password_timeout(data.account_id)
        .await
        .unwrap_or(false);
    Ok(JsonResponse::data(JsonData::body(json!({
        "auth_data":data,
        "jwt":token,
        "passwrod_timeout":passwrod_timeout,
    }))))
}

#[post("/login/{type}")]
pub(crate) async fn login(
    path: actix_web::web::Path<String>,
    json_param: JsonQuery,
    auth_dao: UserAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    let res = match path.into_inner().as_str() {
        "sms-send-code" => {
            user_login_mobile_send_code(&json_param.param::<MobileSendCodeLoginParam>()?, &auth_dao)
                .await
        }
        "email-send-code" => {
            user_login_email_send_code(&json_param.param::<EmailSendCodeLoginParam>()?, &auth_dao)
                .await
        }
        e => {
            let (token, data) = match e {
                "name" => {
                    user_login_from_name(&json_param.param::<NameLoginParam>()?, &auth_dao).await
                }
                "sms" => {
                    user_login_from_mobile(&json_param.param::<MobileLoginParam>()?, &auth_dao)
                        .await
                }
                "email" => {
                    user_login_from_email(&json_param.param::<EmailLoginParam>()?, &auth_dao).await
                }
                "sms-code" => {
                    user_login_from_mobile_code(
                        &json_param.param::<MobileCodeLoginParam>()?,
                        &auth_dao,
                    )
                    .await
                }
                "email-code" => {
                    user_login_from_email_code(
                        &json_param.param::<EmailCodeLoginParam>()?,
                        &auth_dao,
                    )
                    .await
                }
                "app-code" => {
                    user_login_from_app_code(&json_param.param::<AppCodeLoginParam>()?, &auth_dao)
                        .await
                }
                name => handler_not_found!(name),
            }
            .map_err(|e| auth_dao.fluent_error_json_response(&e))?;
            jwt_login_data(&auth_dao, token, data).await
        }
    };
    Ok(res
        .map_err(|e| auth_dao.fluent_error_json_response(&e))?
        .into())
}

#[post("/login_data")]
pub(crate) async fn user_data(
    jwt: JwtQuery,
    auth_dao: UserAuthQuery,
    json_param: JsonQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;

    let (token_data, out_auth_data, user_data, passwrod_timeout) =
        login_data_from_user_auth(&json_param.param::<UserAuthDataOptionParam>()?, &auth_dao)
            .await
            .map_err(|e| auth_dao.fluent_error_json_response(&e))?;
    let jwt = if let Some(ref ua) = out_auth_data {
        let app_jwt_key = auth_dao
            .web_dao
            .app_core
            .config
            .find(None)
            .get_string("app_jwt_key")
            .unwrap_or_default();
        let utoken = UserAuthToken::from(&token_data);
        let token = encode(
            &Header::default(),
            &JwtClaims::new(utoken.time_out as i64, utoken.to_string(), Some(json!(ua))),
            &EncodingKey::from_secret(app_jwt_key.as_bytes()),
        )
        .map_err(|e| {
            auth_dao
                .fluent_error_json_response(&JsonError::Message(fluent_message!("system-error", e)))
        })?;
        Some(token)
    } else {
        None
    };

    Ok(JsonResponse::data(JsonData::body(json!({
        "auth_data": out_auth_data ,
        "jwt":jwt,
        "user_data":json!({
            "user":user_data.0,
            "name":user_data.1,
            "info":user_data.2,
            "address":user_data.3,
            "email":user_data.4,
            "external":user_data.5,
            "mobile":user_data.6,
            "passwrod_timeut":passwrod_timeout
        }),
    })))
    .into())
}

#[post("/logout")]
pub async fn logout(jwt: JwtQuery, auth_dao: UserAuthQuery) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    auth_dao
        .web_dao
        .web_user
        .auth
        .user_logout(&auth_dao.user_session)
        .await
        .map_err(|e| auth_dao.fluent_error_json_response(&e))?;
    Ok(JsonResponse::default().into())
}

//------------------------外部OAUTH登录------------------------

#[derive(Debug, Deserialize)]
pub struct WechatExternalLoginParam {
    pub login_callback: String,
    pub login_state: String,
}
//获取外部登录URL地址
#[post("/exter_login_url/{method}")]
pub async fn external_login_url(
    path: actix_web::web::Path<String>,
    json_param: JsonQuery,
    req_dao: ReqQuery,
) -> ResponseJsonResult<ResponseJson> {
    let res = match path.into_inner().as_str() {
        OAUTH_TYPE_WECHAT => {
            let config = req_dao
                .web_dao
                .web_setting
                .setting_dao
                .single
                .load::<WeChatConfig>(None)
                .await
                .map_err(|e| req_dao.fluent_error_json_response(&e.into()))?;
            let login_param = json_param.param::<WechatExternalLoginParam>()?;
            user_external_login_url(
                &WechatLogin::new(
                    req_dao.web_dao.clone(),
                    &config.app_id,
                    &config.app_secret,
                    OAUTH_TYPE_WECHAT,
                ),
                &WechatLoginParam {
                    state: login_param.login_state,
                    callback_url: login_param.login_callback,
                },
                &req_dao,
            )
            .await
        }
        name => handler_not_found!(name),
    };
    Ok(res
        .map_err(|e| req_dao.fluent_error_json_response(&e))?
        .into())
}

#[derive(Debug, Deserialize)]
pub struct WechatExternalLoginStateCheckParam {
    pub login_state: String,
}
//扫码登录检测是否已经完成登录
#[post("/exter_state_check/{method}")]
pub async fn external_state_check(
    path: actix_web::web::Path<String>,
    json_param: JsonQuery,
    auth_dao: UserAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    let res = match path.into_inner().as_str() {
        OAUTH_TYPE_WECHAT => {
            let login_param = json_param.param::<WechatExternalLoginStateCheckParam>()?;
            let config = auth_dao
                .web_dao
                .web_setting
                .setting_dao
                .single
                .load::<WeChatConfig>(None)
                .await
                .map_err(|e| auth_dao.fluent_error_json_response(&e.into()))?;
            let wechat = WechatLogin::new(
                auth_dao.web_dao.clone(),
                &config.app_id,
                &config.app_secret,
                OAUTH_TYPE_WECHAT,
            );
            let (reload, login_data) = wechat
                .state_check(&auth_dao, &login_param.login_state)
                .await
                .map_err(|e| auth_dao.fluent_error_json_response(&e))?;
            if let Some(ldat) = login_data {
                let (token, data) =
                    user_login_from_external::<WechatLogin, WechatLoginParam, _, _>(
                        &wechat, &ldat, 0, &auth_dao,
                    )
                    .await
                    .map_err(|e| auth_dao.fluent_error_json_response(&e))?;
                jwt_login_data(&auth_dao, token, data).await
            } else {
                Ok(JsonResponse::data(JsonData::body(
                    json!({ "reload": reload }),
                )))
            }
        }
        name => handler_not_found!(name),
    };
    Ok(res
        .map_err(|e| auth_dao.fluent_error_json_response(&e))?
        .into())
}

#[derive(Debug, Deserialize)]
pub struct WechatExternalLoginStateCallbackParam {
    pub code: String,
    pub callback_state: String,
}
//APP端完成扫码登录后，页面上要提醒“确认登陆!!!”
//请求此回调地址完成登录操作
#[post("/exter_state_callback/{method}")]
pub async fn external_state_callback(
    path: actix_web::web::Path<String>,
    json_param: JsonQuery,
    app_dao: ReqQuery,
) -> ResponseJsonResult<ResponseJson> {
    let res = match path.into_inner().as_str() {
        OAUTH_TYPE_WECHAT => {
            let config = app_dao
                .web_dao
                .web_setting
                .setting_dao
                .single
                .load::<WeChatConfig>(None)
                .await
                .map_err(|e| app_dao.fluent_error_json_response(&e.into()))?;
            let login_param = json_param.param::<WechatExternalLoginStateCallbackParam>()?;
            let wechat = WechatLogin::new(
                app_dao.web_dao.clone(),
                &config.app_id,
                &config.app_secret,
                OAUTH_TYPE_WECHAT,
            );
            wechat
                .state_callback(
                    &app_dao,
                    &WechatCallbackParam {
                        code: login_param.code,
                        state: login_param.callback_state,
                    },
                )
                .await
        }
        name => handler_not_found!(name),
    };
    Ok(res
        .map_err(|e| app_dao.fluent_error_json_response(&e))?
        .into())
}

// //外部登录完成回调地址,不包含扫码登录,目前没用到
// #[derive(Debug, Deserialize)]
// pub struct ExternalCallbackParam {
//     pub login_type: String,
//     pub code: String,
//     pub callback_state: String,
// }

// #[post("/exter_login_callback")]
// pub async fn external_login_callback(
//     json_param: JsonQuery,
//     auth_dao: UserAuthQuery,
// ) -> ResponseJsonResult<ResponseJson> {
//     let login_param = json_param.param::<ExternalCallbackParam>()?;
//     let res = match login_param.login_type.as_str() {
//         "qq" => {
//             //调用外部API检查 code??
//             let config = auth_dao
//                 .web_dao
//                 .web_setting
//                 .setting_dao
//                 .single
//                 .load::<WeChatConfig>(None)
//                 .await
//                 .map_err(|e| auth_dao.fluent_error_json_response(&e.into()))?;
//             let wechat = WechatLogin::new(
//                 auth_dao.web_dao.clone(),
//                 &config.app_id,
//                 &config.app_secret,
//                OAUTH_TYPE_WECHAT,
//             );
//             let (_, login_data) = wechat
//                 .state_check(&auth_dao, &login_param.callback_state)
//                 .await
//                 .map_err(|e| auth_dao.fluent_error_json_response(&e))?;
//             if let Some(ldat) = login_data {
//                 let (token, _) = user_login_from_external::<WechatLogin, WechatLoginParam, _, _>(
//                     &wechat, &ldat, 0, &auth_dao,
//                 )
//                 .await
//                 .map_err(|e| auth_dao.fluent_error_json_response(&e))?;
//                 return Ok(JsonResponse::data(JsonData::body(json!({ "token": token.to_string() }))).into());
//             }
//             Ok(JsonResponse::message("unimplemented"))
//         }
//         name => handler_not_found!(name),
//     };
//     Ok(res.map_err(|e| auth_dao.fluent_error_json_response(&e))?.into())
// }
