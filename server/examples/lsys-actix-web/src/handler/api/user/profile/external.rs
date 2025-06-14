use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use lsys_web::common::JsonData;
use lsys_web::handler::api::user::account::{external_bind, external_bind_url};
use lsys_web::lsys_access::dao::AccessSession;
use lsys_web::lsys_core::fluent_message;
use lsys_web::{
    common::{JsonError, JsonResponse},
    handler::api::user::account::{
        external_delete, external_list_data, ExternalDeleteParam, ExternalListDataParam,
    },
};
use lsys_web_module_oauth::module::{
    WeChatConfig, WechatLogin, WechatLoginParam, OAUTH_TYPE_WECHAT,
};
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct ExternalBindUrlParam {
    pub login_type: String,
    pub login_state: String,
    pub callback_url: String,
}

#[derive(Debug, Deserialize)]
pub struct ExternalBindCheckParam {
    pub login_type: String,
    pub login_state: String,
}

#[post("/exter/{method}")]
pub(crate) async fn external(
    jwt: JwtQuery,
    path: actix_web::web::Path<String>,
    json_param: JsonQuery,
    auth_dao: UserAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao
        .set_request_token(&jwt)
        .await
        .map_err(|e| auth_dao.fluent_error_json_response(&e))?;
    Ok(match path.into_inner().as_str() {
        "list_data" => external_list_data(&json_param.param::<ExternalListDataParam>()?, &auth_dao)
            .await
            .map_err(|e| auth_dao.fluent_error_json_response(&e))?,
        "bind_check" => {
            let login_param = json_param.param::<ExternalBindCheckParam>()?;
            match login_param.login_type.as_str() {
                OAUTH_TYPE_WECHAT => {
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
                        external_bind(&wechat, &ldat, &auth_dao)
                            .await
                            .map_err(|e| auth_dao.fluent_error_json_response(&e))?
                    } else {
                        JsonResponse::data(JsonData::body(json!({ "reload": reload })))
                    }
                }
                name => {
                    handler_not_found!(name).map_err(|e| auth_dao.fluent_error_json_response(&e))?
                }
            }
        }
        "bind_url" => {
            auth_dao
                .user_session
                .read()
                .await
                .get_session_data()
                .await
                .map_err(|e| auth_dao.fluent_error_json_response(&e.into()))?;
            let param = json_param.param::<ExternalBindUrlParam>()?;
            match param.login_type.as_str() {
                OAUTH_TYPE_WECHAT => {
                    let config = auth_dao
                        .web_dao
                        .web_setting
                        .setting_dao
                        .single
                        .load::<WeChatConfig>(None)
                        .await
                        .map_err(|e| auth_dao.fluent_error_json_response(&e.into()))?;
                    external_bind_url(
                        &WechatLogin::new(
                            auth_dao.web_dao.clone(),
                            &config.app_id,
                            &config.app_secret,
                            OAUTH_TYPE_WECHAT,
                        ),
                        &WechatLoginParam {
                            state: param.login_state,
                            callback_url: param.callback_url,
                        },
                        &auth_dao,
                    )
                    .await
                    .map_err(|e| auth_dao.fluent_error_json_response(&e))?
                }
                name => auth_dao.fluent_error_json_response(&JsonError::JsonResponse(
                    JsonData::default()
                        .set_sub_code("type_not_support")
                        .set_code(400),
                    fluent_message!("external-not-support",{
                        "name":name
                    }),
                )),
            }
        }
        "delete" => external_delete(&json_param.param::<ExternalDeleteParam>()?, &auth_dao)
            .await
            .map_err(|e| auth_dao.fluent_error_json_response(&e))?,
        name => handler_not_found!(name).map_err(|e| auth_dao.fluent_error_json_response(&e))?,
    }
    //
    .into())
}
