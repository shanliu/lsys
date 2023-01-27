use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use lsys_user::dao::auth::UserSession;
use lsys_web::{
    handler::api::{
        login::user_oauth_login,
        user::{
            user_external_bind, user_external_delete, user_external_list_data, ExternalBindParam,
            ExternalDeleteParam, ExternalListDataParam,
        },
    },
    JsonData,
};
use lsys_web_module_oauth::module::{WechatLogin, WechatLoginParam};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ExternalBindLink {
    pub login_type: String,
    pub state: String,
    pub callback_url: String,
}
#[post("external/{method}")]
pub(crate) async fn external<'t>(
    jwt: JwtQuery,
    path: actix_web::web::Path<(String,)>,
    rest: JsonQuery,
    auth_dao: UserAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    Ok(match path.0.to_string().as_str() {
        "list_data" => {
            user_external_list_data(rest.param::<ExternalListDataParam>()?, &auth_dao).await
        }
        "bind" => user_external_bind(rest.param::<ExternalBindParam>()?, &auth_dao).await,
        // "qrcode_bind" => {

        //     let param = rest.param::<ExternalQrCodeBindParam>()?;
        //     match param.external_type.as_str() {
        //         "wechat" => {
        //             let wechat = user_oauth::<WechatLogin, WechatLoginParam, _, _>(
        //                 "wechat",
        //                 &auth_dao.web_dao,
        //             )
        //             .await?;
        //             let (reload, login_data) = wechat
        //                 .get_state_login_data(&auth_dao.web_dao.user, &param.login_state)
        //                 .await?;
        //             if let Some(ldat) = login_data {
        //                 //@todo there.....
        //             } else {
        //                 Ok(JsonData::data(json!({ "reload": reload })))
        //             }
        //         }
        //         name => handler_not_found!(name),
        //     }
        // }
        "link" => {
            auth_dao
                .user_session
                .read()
                .await
                .get_session_data()
                .await
                .map_err(Into::<JsonData>::into)?;
            let param = rest.param::<ExternalBindLink>()?;
            match param.login_type.as_str() {
                "wechat" => {
                    user_oauth_login::<WechatLogin, WechatLoginParam, _, _>(
                        "wechat",
                        &auth_dao.web_dao,
                        &WechatLoginParam {
                            state: param.state,
                            callback_url: param.callback_url,
                        },
                    )
                    .await
                }
                name => {
                    Ok(JsonData::message(format!("not support login type:{}", name)).set_code(500))
                }
            }
        }
        "delete" => user_external_delete(rest.param::<ExternalDeleteParam>()?, &auth_dao).await,
        name => handler_not_found!(name),
    }?
    .into())
}
