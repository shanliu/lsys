use crate::common::handler::{
    BearerQuery, JsonQuery, ReqQuery, ResponseJson, ResponseJsonResult, TokenSignConfig,
    UserAuthQuery, wrap_token,
};

use actix_web::{post, web};
use actix_web::web::Data;

use lsys_web::const_json_format;
use lsys_web::dao::WebDao;
// use lsys_web::lsys_core::IntoFluentMessage;
use lsys_web::lsys_user::dao::UserAuthToken;

use lsys_web::common::{JsonData, JsonResponse, JsonResult};
use lsys_web::dao::ShowUserAuthData;
use lsys_web::handler::api::auth::AppCodeLoginParam;
use lsys_web::handler::api::auth::EmailCodeLoginParam;
use lsys_web::handler::api::auth::EmailLoginParam;
use lsys_web::handler::api::auth::EmailSendCodeLoginParam;
use lsys_web::handler::api::auth::MfaVerifyParam;
use lsys_web::handler::api::auth::MobileCodeLoginParam;
use lsys_web::handler::api::auth::MobileLoginParam;
use lsys_web::handler::api::auth::MobileSendCodeLoginParam;
use lsys_web::handler::api::auth::NameLoginParam;
use lsys_web::handler::api::auth::UserAuthDataOptionParam;
use lsys_web::handler::api::auth::user_login_from_app_code;
use lsys_web::handler::api::auth::user_login_from_email;
use lsys_web::handler::api::auth::user_login_from_email_code;
use lsys_web::handler::api::auth::user_login_from_external;
use lsys_web::handler::api::auth::user_login_from_mobile;
use lsys_web::handler::api::auth::user_login_from_mobile_code;
use lsys_web::handler::api::auth::user_login_from_name;
use lsys_web::handler::api::auth::user_login_mobile_send_code;
use lsys_web::handler::api::auth::user_mfa_verify;
use lsys_web::handler::api::auth::{login_data_from_user_auth, user_external_login_url};
use lsys_web::handler::api::auth::{mapping_data, user_login_email_send_code};
use lsys_web::lsys_access::dao::AccessSession;
use lsys_web_module_oauth::module::{
    OAUTH_TYPE_WECHAT, WeChatConfig, WechatCallbackParam, WechatLogin, WechatLoginParam,
};
use serde::Deserialize;
use serde_json::json;

async fn login_token_data(
    web_dao: &WebDao,
    token: UserAuthToken,
    data: ShowUserAuthData,
) -> JsonResult<JsonResponse> {
    let sign_key = TokenSignConfig::from_config(web_dao).sign_key().to_string();
    let passwrod_timeout = web_dao
        .web_user
        .user_dao
        .account_dao
        .account_password
        .password_timeout(data.account_id)
        .await
        .unwrap_or((false, 0))
        .0;
    Ok(JsonResponse::data(JsonData::body(json!({
        "auth_data":data,
        "token":wrap_token(&token.to_string(), &sign_key),
        "passwrod_timeout":passwrod_timeout,
    }))))
}

#[post("/login/{type}")]
pub(crate) async fn login(
    path: actix_web::web::Path<String>,
    json_param: JsonQuery,
    auth_dao: UserAuthQuery,
    req_query: ReqQuery,
    web_dao: Data<WebDao>,
) -> ResponseJsonResult<ResponseJson> {
    let res = match path.into_inner().as_str() {
        "mapping" => {
            mapping_data(
                &req_query,
                json!([const_json_format!(&req_query, OAUTH_TYPE_WECHAT)]),
            )
            .await
        }
        "sms-send-code" => {
            user_login_mobile_send_code(
                &json_param.param::<MobileSendCodeLoginParam>()?,
                &req_query,
                web_dao.as_ref(),
            )
            .await
        }
        "email-send-code" => {
            user_login_email_send_code(
                &json_param.param::<EmailSendCodeLoginParam>()?,
                &req_query,
                web_dao.as_ref(),
            )
            .await
        }
        e => {
            match e {
                "mfa-verify" => {
                    let (token, data) = user_mfa_verify(
                        &json_param.param::<MfaVerifyParam>()?,
                        &req_query,
                        &auth_dao,
                        web_dao.as_ref(),
                    )
                    .await
                    .map_err(|e| req_query.fluent_error_json_response(&e))?;
                    login_token_data(web_dao.as_ref(), token, data).await
                }
                _ => {
                    let (token, auth_data) = match e {
                        "name" => {
                            user_login_from_name(
                                &json_param.param::<NameLoginParam>()?,
                                &req_query,
                                &auth_dao,
                                web_dao.as_ref(),
                            )
                            .await
                        }
                        "sms" => {
                            user_login_from_mobile(
                                &json_param.param::<MobileLoginParam>()?,
                                &req_query,
                                &auth_dao,
                                web_dao.as_ref(),
                            )
                            .await
                        }
                        "email" => {
                            user_login_from_email(
                                &json_param.param::<EmailLoginParam>()?,
                                &req_query,
                                &auth_dao,
                                web_dao.as_ref(),
                            )
                            .await
                        }
                        "sms-code" => {
                            user_login_from_mobile_code(
                                &json_param.param::<MobileCodeLoginParam>()?,
                                &req_query,
                                &auth_dao,
                                web_dao.as_ref(),
                            )
                            .await
                        }
                        "email-code" => {
                            user_login_from_email_code(
                                &json_param.param::<EmailCodeLoginParam>()?,
                                &req_query,
                                &auth_dao,
                                web_dao.as_ref(),
                            )
                            .await
                        }
                        "app-code" => {
                            // code login remains single-phase
                            let (token, data) = user_login_from_app_code(
                                &json_param.param::<AppCodeLoginParam>()?,
                                &req_query,
                                &auth_dao,
                                web_dao.as_ref(),
                            )
                            .await
                            .map_err(|e| req_query.fluent_error_json_response(&e))?;
                            return Ok(login_token_data(web_dao.as_ref(), token, data)
                                .await
                                .map_err(|e| req_query.fluent_error_json_response(&e))?
                                .into());
                        }
                        name => handler_not_found!(name),
                    }
                    .map_err(|e| req_query.fluent_error_json_response(&e))?;

                    login_token_data(web_dao.as_ref(), token, auth_data).await
                }
            }
        }
    };
    Ok(res
        .map_err(|e| req_query.fluent_error_json_response(&e))?
        .into())
}

#[post("/login_data")]
pub(crate) async fn user_data(
    bearer: BearerQuery,
    auth_dao: UserAuthQuery,
    json_param: JsonQuery,
    req_query: ReqQuery,
    web_dao: Data<WebDao>,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao
        .set_request_token(&bearer)
        .await
        .map_err(|e| req_query.fluent_error_json_response(&e))?;

    let (token_data, out_auth_data, user_data, passwrod_timeout) = login_data_from_user_auth(
        &json_param.param::<UserAuthDataOptionParam>()?,
        &auth_dao,
        web_dao.as_ref(),
    )
    .await
    .map_err(|e| req_query.fluent_error_json_response(&e))?;
    let token = if out_auth_data.is_some() {
        let sign_key = TokenSignConfig::from_config(web_dao.as_ref()).sign_key().to_string();
        Some(wrap_token(
            &UserAuthToken::from(&token_data).to_string(),
            &sign_key,
        ))
    } else {
        None
    };

    Ok(JsonResponse::data(JsonData::body(json!({
        "auth_data": out_auth_data ,
        "token":token,
        "user_data":json!({
            "account":user_data.0,
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

/// 显式刷新 token（非 cookie）：轮换 token 字符串并重置有效期。
///
/// 与被动续期不同，这是**客户端主动调用**的接口：
/// - 旧 token 立即失效（服务端删除旧 cache key 并改写 DB）；
/// - 返回携带**全新 token 字符串**的 `token`，客户端须用它替换原 token；
/// - 同时回传刷新后的 `auth_data` / `user_data`（按请求参数裁剪）。
#[post("/token_refresh")]
pub(crate) async fn token_refresh(
    bearer: BearerQuery,
    auth_dao: UserAuthQuery,
    json_param: JsonQuery,
    req_query: ReqQuery,
    web_dao: Data<WebDao>,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao
        .set_request_token(&bearer)
        .await
        .map_err(|e| req_query.fluent_error_json_response(&e))?;

    // 取当前会话 token，轮换出全新 token（旧 token 此刻起失效）。
    let cur_token = auth_dao
        .user_session
        .read()
        .await
        .get_session_token()
        .clone();
    let new_token = web_dao
        .web_user
        .user_dao
        .auth_dao
        .reload(&cur_token, true)
        .await
        .map_err(|e| req_query.fluent_error_json_response(&e.into()))?;
    // 回写会话 token，使后续读取（含本次 login_data 加载）走新 token。
    auth_dao
        .user_session
        .write()
        .await
        .set_session_token(new_token.clone());

    let (_token_data, out_auth_data, user_info, passwrod_timeout) = login_data_from_user_auth(
        &json_param.param::<UserAuthDataOptionParam>()?,
        &auth_dao,
        web_dao.as_ref(),
    )
    .await
    .map_err(|e| req_query.fluent_error_json_response(&e))?;

    // 刷新接口总是回传新 token（与 /login_data 的「按需」语义不同）。
    let sign_key = TokenSignConfig::from_config(web_dao.as_ref())
        .sign_key()
        .to_string();
    let token = wrap_token(&new_token.to_string(), &sign_key);

    Ok(JsonResponse::data(JsonData::body(json!({
        "auth_data": out_auth_data ,
        "token":token,
        "user_data":json!({
            "account":user_info.0,
            "name":user_info.1,
            "info":user_info.2,
            "address":user_info.3,
            "email":user_info.4,
            "external":user_info.5,
            "mobile":user_info.6,
            "passwrod_timeut":passwrod_timeout
        }),
    })))
    .into())
}

#[post("/logout")]
pub async fn logout(
    bearer: BearerQuery,
    auth_dao: UserAuthQuery,
    req_query: ReqQuery,
    web_dao: Data<WebDao>,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao
        .set_request_token(&bearer)
        .await
        .map_err(|e| req_query.fluent_error_json_response(&e))?;

    let auth_data = auth_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_query.fluent_error_json_response(&e.into()))?;

    web_dao
        .web_user
        .user_dao
        .auth_dao
        .logout(&auth_data)
        .await
        .map_err(|e| req_query.fluent_error_json_response(&e.into()))?;
    auth_dao
        .user_session
        .write()
        .await
        .set_session_token(UserAuthToken::default());
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
    web_dao: web::Data<WebDao>,
) -> ResponseJsonResult<ResponseJson> {
    let res = match path.into_inner().as_str() {
        OAUTH_TYPE_WECHAT => {
            let config = web_dao
                .as_ref()
                .web_setting
                .setting_dao
                .single
                .load::<WeChatConfig>(None)
                .await
                .map_err(|e| req_dao.fluent_error_json_response(&e.into()))?;
            let login_param = json_param.param::<WechatExternalLoginParam>()?;
            user_external_login_url(
                &WechatLogin::new(
                    web_dao.clone().into_inner(),
                    &config.app_id,
                    &config.app_secret,
                    OAUTH_TYPE_WECHAT,
                ),
                &WechatLoginParam {
                    state: login_param.login_state,
                    callback_url: login_param.login_callback,
                },
                web_dao.as_ref(),
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
    req_query: ReqQuery,
    web_dao: Data<WebDao>,
) -> ResponseJsonResult<ResponseJson> {
    let res = match path.into_inner().as_str() {
        OAUTH_TYPE_WECHAT => {
            let login_param = json_param.param::<WechatExternalLoginStateCheckParam>()?;
            let config = web_dao
                .web_setting
                .setting_dao
                .single
                .load::<WeChatConfig>(None)
                .await
                .map_err(|e| req_query.fluent_error_json_response(&e.into()))?;
            let wechat = WechatLogin::new(
                web_dao.clone().into_inner(),
                &config.app_id,
                &config.app_secret,
                OAUTH_TYPE_WECHAT,
            );
            let (reload, login_data) = wechat
                .state_check(web_dao.as_ref(), &login_param.login_state)
                .await
                .map_err(|e| req_query.fluent_error_json_response(&e))?;
            if let Some(ldat) = login_data {
                let (token, auth_data) =
                    user_login_from_external::<WechatLogin, WechatLoginParam, _, _>(
                        &wechat, &ldat, 0, &req_query, &auth_dao, web_dao.as_ref(),
                    )
                    .await
                    .map_err(|e| req_query.fluent_error_json_response(&e))?;

                login_token_data(web_dao.as_ref(), token, auth_data).await
            } else {
                Ok(JsonResponse::data(JsonData::body(
                    json!({ "reload": reload }),
                )))
            }
        }
        name => handler_not_found!(name),
    };
    Ok(res
        .map_err(|e| req_query.fluent_error_json_response(&e))?
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
    web_dao: web::Data<WebDao>,
) -> ResponseJsonResult<ResponseJson> {
    let res = match path.into_inner().as_str() {
        OAUTH_TYPE_WECHAT => {
            let config = web_dao
                .as_ref()
                .web_setting
                .setting_dao
                .single
                .load::<WeChatConfig>(None)
                .await
                .map_err(|e| app_dao.fluent_error_json_response(&e.into()))?;
            let login_param = json_param.param::<WechatExternalLoginStateCallbackParam>()?;
            let wechat = WechatLogin::new(
                web_dao.clone().into_inner(),
                &config.app_id,
                &config.app_secret,
                OAUTH_TYPE_WECHAT,
            );
            wechat
                .state_callback(
                    web_dao.as_ref(),
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
