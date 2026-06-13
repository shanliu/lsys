use crate::common::handler::{
    JsonQuery, BearerQuery, ReqQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};

use actix_web::post;
use actix_web::web::Data;
use lsys_web::dao::WebDao;
use lsys_web::handler::api::user::account::{
    MfaBindParam, mfa_bind_device, mfa_bind_qrcode, mfa_status, mfa_unbind,
};
#[post("/{type}")]
pub async fn mfa(
    auth_dao: UserAuthQuery,
    path: actix_web::web::Path<String>,
    json_param: JsonQuery,
    bearer: BearerQuery,
    req_query: ReqQuery,
    web_dao: Data<WebDao>,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao
        .set_request_token(&bearer)
        .await
        .map_err(|e| req_query.fluent_error_json_response(&e))?;
    Ok(match path.into_inner().as_str() {
        "bind_qrcode" => mfa_bind_qrcode(&auth_dao, web_dao.as_ref()).await,
        "bind_device" => mfa_bind_device(&json_param.param::<MfaBindParam>()?, &auth_dao, web_dao.as_ref()).await,
        "bind_status" => mfa_status(&auth_dao, web_dao.as_ref()).await,
        "unbind" => mfa_unbind(&auth_dao, web_dao.as_ref()).await,
        name => handler_not_found!(name),
    }
    .map_err(|e| req_query.fluent_error_json_response(&e))?
    .into())
}
