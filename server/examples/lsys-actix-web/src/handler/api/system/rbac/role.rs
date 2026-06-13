use crate::common::handler::{
    BearerQuery, JsonQuery, ReqQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use actix_web::web::Data;
use lsys_web::dao::WebDao;
use lsys_web::handler::api::system::rbac::{
    RoleAddParam, RoleDataParam, RoleDelParam, RoleEditParam, RolePermAddParam, RolePermDelParam,
    RolePermParam, RoleUserAddParam, RoleUserAvailableParam, RoleUserDataParam, RoleUserDelParam,
    role_add, role_data, role_del, role_edit, role_perm_add, role_perm_data, role_perm_del,
    role_user_add, role_user_available, role_user_data, role_user_del,
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
        "add" => role_add(&json_param.param::<RoleAddParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "edit" => role_edit(&json_param.param::<RoleEditParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "delete" => role_del(&json_param.param::<RoleDelParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "list" => role_data(&json_param.param::<RoleDataParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "perm_add" => role_perm_add(&json_param.param::<RolePermAddParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "perm_delete" => role_perm_del(&json_param.param::<RolePermDelParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "perm_data" => role_perm_data(&json_param.param::<RolePermParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "user_add" => role_user_add(&json_param.param::<RoleUserAddParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "user_delete" => role_user_del(&json_param.param::<RoleUserDelParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "user_data" => role_user_data(&json_param.param::<RoleUserDataParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "available_user" => {
            role_user_available(&json_param.param::<RoleUserAvailableParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        name => handler_not_found!(name),
    };
    Ok(data
        .map_err(|e| req_query.fluent_error_json_response(&e))?
        .into())
}
