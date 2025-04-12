use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use lsys_web::handler::api::user::rbac::{
    app_op_add, app_op_data, app_op_del, app_op_edit, AppOpAddParam, AppOpDataParam, AppOpDelParam,
    AppOpEditParam,
};

#[post("/op/{method}")]
pub async fn op(
    jwt: JwtQuery,
    path: actix_web::web::Path<String>,
    json_param: JsonQuery,
    auth_dao: UserAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    let data = match path.into_inner().as_str() {
        "add" => app_op_add(&json_param.param::<AppOpAddParam>()?, &auth_dao).await,
        "edit" => app_op_edit(&json_param.param::<AppOpEditParam>()?, &auth_dao).await,
        "delete" => app_op_del(&json_param.param::<AppOpDelParam>()?, &auth_dao).await,
        "list" => app_op_data(&json_param.param::<AppOpDataParam>()?, &auth_dao).await,
        name => handler_not_found!(name),
    };
    Ok(data
        .map_err(|e| auth_dao.fluent_error_json_response(&e))?
        .into())
}
