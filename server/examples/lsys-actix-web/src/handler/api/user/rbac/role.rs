use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use lsys_web::handler::api::user::rbac::{
    system_role_add, system_role_data, system_role_del, system_role_edit, system_role_perm_add,
    system_role_perm_data, system_role_perm_del, system_role_user_add, system_role_user_available,
    system_role_user_data, system_role_user_del, SystemRoleAddParam, SystemRoleDataParam,
    SystemRoleDelParam, SystemRoleEditParam, SystemRolePermAddParam, SystemRolePermDataParam,
    SystemRolePermDelParam, SystemRoleUserAddParam, SystemRoleUserAvailableParam,
    SystemRoleUserDataParam, SystemRoleUserDelParam,
};

#[post("/role/{method}")]
pub async fn role(
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
        "add" => system_role_add(&json_param.param::<SystemRoleAddParam>()?, &auth_dao).await,
        "edit" => system_role_edit(&json_param.param::<SystemRoleEditParam>()?, &auth_dao).await,
        "delete" => system_role_del(&json_param.param::<SystemRoleDelParam>()?, &auth_dao).await,
        "list" => system_role_data(&json_param.param::<SystemRoleDataParam>()?, &auth_dao).await,
        "perm_add" => {
            system_role_perm_add(&json_param.param::<SystemRolePermAddParam>()?, &auth_dao).await
        }
        "perm_delete" => {
            system_role_perm_del(&json_param.param::<SystemRolePermDelParam>()?, &auth_dao).await
        }
        "perm_data" => {
            system_role_perm_data(&json_param.param::<SystemRolePermDataParam>()?, &auth_dao).await
        }
        "user_add" => {
            system_role_user_add(&json_param.param::<SystemRoleUserAddParam>()?, &auth_dao).await
        }
        "user_delete" => {
            system_role_user_del(&json_param.param::<SystemRoleUserDelParam>()?, &auth_dao).await
        }
        "user_data" => {
            system_role_user_data(&json_param.param::<SystemRoleUserDataParam>()?, &auth_dao).await
        }
        "available_user" => {
            system_role_user_available(
                &json_param.param::<SystemRoleUserAvailableParam>()?,
                &auth_dao,
            )
            .await
        }
        name => handler_not_found!(name),
    };
    Ok(data
        .map_err(|e| auth_dao.fluent_error_json_response(&e))?
        .into())
}
