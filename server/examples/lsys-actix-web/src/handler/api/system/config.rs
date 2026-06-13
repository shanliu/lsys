use crate::common::handler::{
    BearerQuery, JsonQuery, ReqQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use actix_web::web::Data;
use lsys_web::dao::WebDao;
use lsys_web::handler::api::system::setting::SiteConfigParam;
use lsys_web::handler::api::system::setting::site_config_get;
use lsys_web::handler::api::system::setting::site_config_set;
use lsys_web_module_oauth::handler::{WechatSetConfigParam, wechat_get_config, wechat_set_config};
use lsys_web_module_oauth::module::OAUTH_TYPE_WECHAT;

//OAUTH配置
#[post("site_config/{type}")]
pub async fn site_config(
    path: actix_web::web::Path<String>,
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
    let res = match path.into_inner().as_str() {
        "get" => site_config_get(&req_query, &auth_dao, web_dao.as_ref()).await,
        "set" => site_config_set(&json_param.param::<SiteConfigParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        name => handler_not_found!(name),
    };
    Ok(res
        .map_err(|e| req_query.fluent_error_json_response(&e))?
        .into())
}

#[derive(Debug, serde::Deserialize)]
pub struct OAuthConfigParam {
    pub oauth_type: String,
    pub op_type: String,
}

#[post("oauth_config/{oauth_type}/{op_type}")]
pub async fn oauth_config(
    param: actix_web::web::Path<OAuthConfigParam>,
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
    let res = match param.oauth_type.as_str() {
        OAUTH_TYPE_WECHAT => match param.op_type.as_str() {
            "get" => wechat_get_config(&req_query, &auth_dao, web_dao.as_ref()).await,
            "set" => {
                wechat_set_config(json_param.param::<WechatSetConfigParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
            }
            name => handler_not_found!(name),
        },
        name => handler_not_found!(name),
    };
    Ok(res
        .map_err(|e| req_query.fluent_error_json_response(&e))?
        .into())
}
