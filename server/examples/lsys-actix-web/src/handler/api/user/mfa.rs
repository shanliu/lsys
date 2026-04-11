use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};

use actix_web::post;
use lsys_web::handler::api::user::account::{
    MfaBindParam, mfa_bind_device, mfa_bind_qrcode, mfa_status, mfa_unbind,
};
#[post("/{type}")]
pub async fn mfa(
    auth_dao: UserAuthQuery,
    path: actix_web::web::Path<String>,
    json_param: JsonQuery,
    jwt: JwtQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao
        .set_request_token(&jwt)
        .await
        .map_err(|e| auth_dao.fluent_error_json_response(&e))?;
    Ok(match path.into_inner().as_str() {
        "bind_qrcode" => mfa_bind_qrcode(&auth_dao).await,
        "bind_device" => mfa_bind_device(&json_param.param::<MfaBindParam>()?, &auth_dao).await,
        "bind_status" => mfa_status(&auth_dao).await,
        "unbind" => mfa_unbind(&auth_dao).await,
        name => handler_not_found!(name),
    }
    .map_err(|e| auth_dao.fluent_error_json_response(&e))?
    .into())
}
