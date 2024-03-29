use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use lsys_core::fluent_message;
use lsys_user::dao::auth::{SessionData, UserSession};
use lsys_web::{
    handler::{
        api::user::{
            user_external_delete, user_external_list_data, ExternalDeleteParam,
            ExternalListDataParam,
        },
        oauth::user_external_login_url,
    },
    JsonData,
};
use lsys_web_module_oauth::module::{WechatLogin, WechatLoginParam};
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

#[post("external/{method}")]
pub(crate) async fn external<'t>(
    jwt: JwtQuery,
    path: actix_web::web::Path<String>,
    json_param: JsonQuery,
    auth_dao: UserAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    Ok(match path.into_inner().as_str() {
        "list_data" => {
            user_external_list_data(json_param.param::<ExternalListDataParam>()?, &auth_dao).await
        }
        "bind_check" => {
            let req_auth = auth_dao
                .user_session
                .read()
                .await
                .get_session_data()
                .await
                .map_err(|e| auth_dao.fluent_json_data(e))?;
            let login_param = json_param.param::<ExternalBindCheckParam>()?;
            match login_param.login_type.as_str() {
                "wechat" => {
                    let wechat = &auth_dao
                        .web_dao
                        .user
                        .user_external_oauth::<WechatLogin, WechatLoginParam, _, _>("wechat")
                        .await
                        .map_err(|e| auth_dao.fluent_json_data(e))?;
                    let (reload, login_data) = wechat
                        .state_check(&auth_dao, &login_param.login_state)
                        .await?;
                    if let Some(ldat) = login_data {
                        let (ext_model, _, _) = &auth_dao
                            .web_dao
                            .user
                            .user_external_bind(
                                wechat,
                                &ldat,
                                req_auth.user_data().user_id,
                                Some(&auth_dao.req_env),
                            )
                            .await
                            .map_err(|e| auth_dao.fluent_json_data(e))?;
                        Ok(JsonData::data(json!({ "id": ext_model.id })))
                    } else {
                        Ok(JsonData::data(json!({ "reload": reload })))
                    }
                }
                name => handler_not_found!(name),
            }
        }
        "bind_url" => {
            auth_dao
                .user_session
                .read()
                .await
                .get_session_data()
                .await
                .map_err(|e| auth_dao.fluent_json_data(e))?;
            let param = json_param.param::<ExternalBindUrlParam>()?;
            match param.login_type.as_str() {
                "wechat" => {
                    user_external_login_url::<WechatLogin, WechatLoginParam, _, _>(
                        "wechat",
                        &WechatLoginParam {
                            state: param.login_state,
                            callback_url: param.callback_url,
                        },
                        &auth_dao,
                    )
                    .await
                }
                name => Ok(
                    auth_dao
                        .fluent_json_data(fluent_message!("external-not-support",{
                            "name":name
                        }))
                        .set_sub_code("type_not_support"), // JsonData::message(format!("not support login type:{}", name))
                                                           //     .set_sub_code(""),
                ),
            }
        }
        "delete" => {
            user_external_delete(json_param.param::<ExternalDeleteParam>()?, &auth_dao).await
        }
        name => handler_not_found!(name),
    }?
    .into())
}
