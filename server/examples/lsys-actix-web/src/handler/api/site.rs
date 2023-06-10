use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::get;
use actix_web::post;

use lsys_web::handler::api::utils::{
    site_config_get, site_config_info, site_config_set, SiteConfigParam,
};
use lsys_web_module_oauth::handler::{wechat_get_config, wechat_set_config, WechatSetConfigParam};

//OAUTH配置
#[post("/oauth/{type}")]
pub async fn oauth_config(
    path: actix_web::web::Path<(String,)>,
    jwt: JwtQuery,
    auth_dao: UserAuthQuery,
    json_param: JsonQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    let res = match path.0.to_string().as_str() {
        "wechat-get" => wechat_get_config(&auth_dao).await,
        "wechat-set" => {
            wechat_set_config(json_param.param::<WechatSetConfigParam>()?, &auth_dao).await
        }
        name => handler_not_found!(name),
    };
    Ok(res?.into())
}

#[post("/system/{type}")]
pub async fn system_config(
    path: actix_web::web::Path<(String,)>,
    jwt: JwtQuery,
    auth_dao: UserAuthQuery,
    json_param: JsonQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    let res = match path.0.to_string().as_str() {
        "base-get" => site_config_get(&auth_dao).await,
        "base-set" => site_config_set(json_param.param::<SiteConfigParam>()?, &auth_dao).await,

        name => handler_not_found!(name),
    };
    Ok(res?.into())
}

#[get("/info")]
pub async fn system_info(auth_dao: UserAuthQuery) -> ResponseJsonResult<ResponseJson> {
    Ok(site_config_info(&auth_dao.web_dao).await?.into())
}
