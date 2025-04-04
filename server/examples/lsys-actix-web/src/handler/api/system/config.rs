use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use lsys_web::handler::api::system::setting::site_config_get;
use lsys_web::handler::api::system::setting::site_config_set;
use lsys_web::handler::api::system::setting::SiteConfigParam;
use lsys_web_module_oauth::handler::{wechat_get_config, wechat_set_config, WechatSetConfigParam};

//OAUTH配置
#[post("/{type}")]
pub async fn config(
    path: actix_web::web::Path<String>,
    jwt: JwtQuery,
    auth_dao: UserAuthQuery,
    json_param: JsonQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    let res = match path.into_inner().as_str() {
        "oauth-wechat-get" => wechat_get_config(&auth_dao).await,
        "oauth-wechat-set" => {
            wechat_set_config(json_param.param::<WechatSetConfigParam>()?, &auth_dao).await
        }
        "site-config-get" => site_config_get(&auth_dao).await,
        "site-config-set" => {
            site_config_set(&json_param.param::<SiteConfigParam>()?, &auth_dao).await
        }
        name => handler_not_found!(name),
    };
    Ok(res.map_err(|e| auth_dao.fluent_error_json_data(&e))?.into())
}
