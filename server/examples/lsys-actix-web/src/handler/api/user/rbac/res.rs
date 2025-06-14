use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use lsys_web::handler::api::user::rbac::{system_res_data, system_res_global_data, SystemResParam};

#[post("/res/{method}")]
pub async fn res(
    jwt: JwtQuery,
    path: actix_web::web::Path<String>,
    json_param: JsonQuery,
    auth_dao: UserAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao
        .set_request_token(&jwt)
        .await
        .map_err(|e| auth_dao.fluent_error_json_response(&e))?;
    let data = match path.into_inner().as_str() {
        "global_data" => system_res_global_data(&auth_dao).await,
        "good_res" => system_res_data(&json_param.param::<SystemResParam>()?, &auth_dao).await,
        name => handler_not_found!(name),
    };
    Ok(data
        .map_err(|e| auth_dao.fluent_error_json_response(&e))?
        .into())
}
