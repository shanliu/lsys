use crate::common::handler::{
    JsonQuery, BearerQuery, ReqQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use actix_web::web::Data;
use lsys_web::dao::WebDao;
use lsys_web::handler::api::user::app_rbac::{
    AppRoleAddParam, AppRoleDataParam, AppRoleDelParam, AppRoleEditParam, AppRolePermAddParam,
    AppRolePermDataParam, AppRolePermDelParam, AppRoleUserAddParam, AppRoleUserAvailableParam,
    AppRoleUserDataParam, AppRoleUserDelParam, app_role_add, app_role_data, app_role_del,
    app_role_edit, app_role_perm_add, app_role_perm_data, app_role_perm_del, app_role_user_add,
    app_role_user_available, app_role_user_data, app_role_user_del,
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
        "add" => app_role_add(&json_param.param::<AppRoleAddParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "edit" => app_role_edit(&json_param.param::<AppRoleEditParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "delete" => app_role_del(&json_param.param::<AppRoleDelParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "list" => app_role_data(&json_param.param::<AppRoleDataParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "perm_add" => {
            app_role_perm_add(&json_param.param::<AppRolePermAddParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "perm_delete" => {
            app_role_perm_del(&json_param.param::<AppRolePermDelParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "perm_data" => {
            app_role_perm_data(&json_param.param::<AppRolePermDataParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "user_add" => {
            app_role_user_add(&json_param.param::<AppRoleUserAddParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "user_delete" => {
            app_role_user_del(&json_param.param::<AppRoleUserDelParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "user_data" => {
            app_role_user_data(&json_param.param::<AppRoleUserDataParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "available_user" => {
            app_role_user_available(&json_param.param::<AppRoleUserAvailableParam>()?, &req_query, &auth_dao, web_dao.as_ref())
                .await
        }
        name => handler_not_found!(name),
    };
    Ok(data
        .map_err(|e| req_query.fluent_error_json_response(&e))?
        .into())
}
