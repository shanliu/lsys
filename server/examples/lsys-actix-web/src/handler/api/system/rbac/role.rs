use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use lsys_web::handler::api::system::rbac::{
    role_add, role_data, role_del, role_edit, role_perm_add, role_perm_data, role_perm_del,
    role_user_add, role_user_available, role_user_data, role_user_del, RoleAddParam, RoleDataParam,
    RoleDelParam, RoleEditParam, RolePermAddParam, RolePermDelParam, RolePermParam,
    RoleUserAddParam, RoleUserAvailableParam, RoleUserDataParam, RoleUserDelParam,
};

#[post("/role/{method}")]
pub async fn role(
    jwt: JwtQuery,
    path: actix_web::web::Path<String>,
    json_param: JsonQuery,
    auth_dao: UserAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    let data = match path.into_inner().as_str() {
        "add" => role_add(&json_param.param::<RoleAddParam>()?, &auth_dao).await,
        "edit" => role_edit(&json_param.param::<RoleEditParam>()?, &auth_dao).await,
        "delete" => role_del(&json_param.param::<RoleDelParam>()?, &auth_dao).await,
        "list" => role_data(&json_param.param::<RoleDataParam>()?, &auth_dao).await,
        "perm_add" => role_perm_add(&json_param.param::<RolePermAddParam>()?, &auth_dao).await,
        "perm_delete" => role_perm_del(&json_param.param::<RolePermDelParam>()?, &auth_dao).await,
        "perm_data" => role_perm_data(&json_param.param::<RolePermParam>()?, &auth_dao).await,
        "user_add" => role_user_add(&json_param.param::<RoleUserAddParam>()?, &auth_dao).await,
        "user_delete" => role_user_del(&json_param.param::<RoleUserDelParam>()?, &auth_dao).await,
        "user_data" => role_user_data(&json_param.param::<RoleUserDataParam>()?, &auth_dao).await,
        "available_user" => {
            role_user_available(&json_param.param::<RoleUserAvailableParam>()?, &auth_dao).await
        }
        name => handler_not_found!(name),
    };
    Ok(data
        .map_err(|e| auth_dao.fluent_error_json_data(&e))?
        .into())
}
