use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use lsys_web::handler::api::user::rbac::{
    app_role_add, app_role_data, app_role_del, app_role_edit, app_role_perm_add,
    app_role_perm_data, app_role_perm_del, app_role_user_add, app_role_user_available,
    app_role_user_data, app_role_user_del, AppRoleAddParam, AppRoleDataParam, AppRoleDelParam,
    AppRoleEditParam, AppRolePermAddParam, AppRolePermDataParam, AppRolePermDelParam,
    AppRoleUserAddParam, AppRoleUserAvailableParam, AppRoleUserDataParam, AppRoleUserDelParam,
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
        "add" => app_role_add(&json_param.param::<AppRoleAddParam>()?, &auth_dao).await,
        "edit" => app_role_edit(&json_param.param::<AppRoleEditParam>()?, &auth_dao).await,
        "delete" => app_role_del(&json_param.param::<AppRoleDelParam>()?, &auth_dao).await,
        "list" => app_role_data(&json_param.param::<AppRoleDataParam>()?, &auth_dao).await,
        "perm_add" => {
            app_role_perm_add(&json_param.param::<AppRolePermAddParam>()?, &auth_dao).await
        }
        "perm_delete" => {
            app_role_perm_del(&json_param.param::<AppRolePermDelParam>()?, &auth_dao).await
        }
        "perm_data" => {
            app_role_perm_data(&json_param.param::<AppRolePermDataParam>()?, &auth_dao).await
        }
        "user_add" => {
            app_role_user_add(&json_param.param::<AppRoleUserAddParam>()?, &auth_dao).await
        }
        "user_delete" => {
            app_role_user_del(&json_param.param::<AppRoleUserDelParam>()?, &auth_dao).await
        }
        "user_data" => {
            app_role_user_data(&json_param.param::<AppRoleUserDataParam>()?, &auth_dao).await
        }
        "available_user" => {
            app_role_user_available(&json_param.param::<AppRoleUserAvailableParam>()?, &auth_dao)
                .await
        }
        name => handler_not_found!(name),
    };
    Ok(data
        .map_err(|e| auth_dao.fluent_error_json_response(&e))?
        .into())
}
