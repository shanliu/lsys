use crate::common::handler::{
    JsonQuery, BearerQuery, ReqQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use actix_web::web::Data;
use lsys_web::dao::WebDao;
use lsys_web::handler::api::user::rbac::{
    SystemRoleAddParam, SystemRoleDataParam, SystemRoleDelParam, SystemRoleEditParam,
    SystemRolePermAddParam, SystemRolePermDataParam, SystemRolePermDelParam,
    SystemRoleUserAddParam, SystemRoleUserAvailableParam, SystemRoleUserDataParam,
    SystemRoleUserDelParam, system_role_add, system_role_data, system_role_del, system_role_edit,
    system_role_perm_add, system_role_perm_data, system_role_perm_del, system_role_user_add,
    system_role_user_available, system_role_user_data, system_role_user_del,
};

#[post("/role/{method}")]
pub async fn role(
    bearer: BearerQuery,
    path: actix_web::web::Path<String>,
    json_param: JsonQuery,
    auth_dao: UserAuthQuery,
    req_query: ReqQuery,
    web_dao: Data<WebDao>,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao
        .set_request_token(&bearer)
        .await
        .map_err(|e| req_query.fluent_error_json_response(&e))?;
    let data = match path.into_inner().as_str() {
        "add" => system_role_add(&json_param.param::<SystemRoleAddParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "edit" => system_role_edit(&json_param.param::<SystemRoleEditParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "delete" => system_role_del(&json_param.param::<SystemRoleDelParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "list" => system_role_data(&json_param.param::<SystemRoleDataParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "perm_add" => {
            system_role_perm_add(&json_param.param::<SystemRolePermAddParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "perm_delete" => {
            system_role_perm_del(&json_param.param::<SystemRolePermDelParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "perm_data" => {
            system_role_perm_data(&json_param.param::<SystemRolePermDataParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "user_add" => {
            system_role_user_add(&json_param.param::<SystemRoleUserAddParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "user_delete" => {
            system_role_user_del(&json_param.param::<SystemRoleUserDelParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "user_data" => {
            system_role_user_data(&json_param.param::<SystemRoleUserDataParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "available_user" => {
            system_role_user_available(
                &json_param.param::<SystemRoleUserAvailableParam>()?,
                &req_query,
                &auth_dao,
                web_dao.as_ref(),
            )
            .await
        }
        name => handler_not_found!(name),
    };
    Ok(data
        .map_err(|e| req_query.fluent_error_json_response(&e))?
        .into())
}
