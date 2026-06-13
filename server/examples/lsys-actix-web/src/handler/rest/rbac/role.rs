use crate::common::handler::{ResponseJson, ResponseJsonResult, RestQuery, ReqQuery};
use actix_web::{post, web};
use lsys_web::dao::WebDao;
use lsys_web::handler::rest::rbac::{
    RoleAddParam, RoleDataParam, RoleDelParam, RoleEditParam, RolePermAddParam, RolePermDelParam,
    RolePermParam, RoleUserAddParam, RoleUserDataParam, RoleUserDelParam, role_add, role_data,
    role_del, role_edit, role_perm_add, role_perm_data, role_perm_del, role_user_add,
    role_user_data, role_user_del,
};

#[post("/role")]
pub async fn role(rest: RestQuery, req_dao: ReqQuery, web_dao: web::Data<WebDao>) -> ResponseJsonResult<ResponseJson> {
    let data = match rest.rfc.method.as_deref().unwrap_or_default() {
        "add" => {
            role_add(
                &rest.param::<RoleAddParam>()?,
                &rest.get_app().await?,
                &req_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "edit" => {
            role_edit(
                &rest.param::<RoleEditParam>()?,
                &rest.get_app().await?,
                &req_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "delete" => {
            role_del(
                &rest.param::<RoleDelParam>()?,
                &rest.get_app().await?,
                &req_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "list" => {
            role_data(
                &rest.param::<RoleDataParam>()?,
                &rest.get_app().await?,
                &req_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "perm_add" => {
            role_perm_add(
                &rest.param::<RolePermAddParam>()?,
                &rest.get_app().await?,
                &req_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "perm_delete" => {
            role_perm_del(
                &rest.param::<RolePermDelParam>()?,
                &rest.get_app().await?,
                &req_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "perm_data" => {
            role_perm_data(
                &rest.param::<RolePermParam>()?,
                &rest.get_app().await?,
                &req_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "user_add" => {
            role_user_add(
                &rest.param::<RoleUserAddParam>()?,
                &rest.get_app().await?,
                &req_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "user_delete" => {
            role_user_del(
                &rest.param::<RoleUserDelParam>()?,
                &rest.get_app().await?,
                &req_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "user_data" => {
            role_user_data(
                &rest.param::<RoleUserDataParam>()?,
                &rest.get_app().await?,
                &req_dao,
                web_dao.as_ref(),
            )
            .await
        }
        name => handler_not_found!(name),
    };
    Ok(data
        .map_err(|e| req_dao.fluent_error_json_response(&e))?
        .into())
}
