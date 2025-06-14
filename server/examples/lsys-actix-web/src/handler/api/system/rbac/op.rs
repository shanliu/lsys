use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use lsys_web::handler::api::system::rbac::{
    op_add, op_data, op_del, op_edit, OpAddParam, OpDataParam, OpDelParam, OpEditParam,
};

#[post("/op/{method}")]
pub async fn op(
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
        "add" => op_add(&json_param.param::<OpAddParam>()?, &auth_dao).await,
        "edit" => op_edit(&json_param.param::<OpEditParam>()?, &auth_dao).await,
        "delete" => op_del(&json_param.param::<OpDelParam>()?, &auth_dao).await,
        "list" => op_data(&json_param.param::<OpDataParam>()?, &auth_dao).await,
        name => handler_not_found!(name),
    };
    Ok(data
        .map_err(|e| auth_dao.fluent_error_json_response(&e))?
        .into())
}
