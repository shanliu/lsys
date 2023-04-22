use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;

use lsys_web::handler::access::res_tpls;
use lsys_web::handler::api::rbac::{
    rbac_all_res_list, RbacAccessParam, RbacMenuParam, ResAddParam, ResDeleteParam, ResEditParam,
    ResListDataParam, RoleRelationDataParam,
};
use lsys_web::handler::api::rbac::{ResAllParam, ResTagsParam, RoleOptionsParam, RoleTagsParam};
use lsys_web::handler::api::rbac::{
    RoleAddParam, RoleAddUserParam, RoleDeleteParam, RoleDeleteUserParam, RoleEditParam,
    RoleListDataParam, RoleListUserParam,
};
use lsys_web::handler::api::user::{
    user_access_check, user_menu_check, user_res_tags, user_role_options, user_role_tags,
};
use lsys_web::handler::api::user::{
    user_res_add, user_res_delete, user_res_edit, user_res_list_data,
};
use lsys_web::handler::api::user::{
    user_role_add, user_role_add_user, user_role_delete, user_role_delete_user, user_role_edit,
    user_role_list_data, user_role_list_user,
};
use lsys_web::handler::oauth::user::user_relation_data;

#[post("/res/{method}")]
pub async fn res<'t>(
    jwt: JwtQuery,
    path: actix_web::web::Path<(String,)>,
    rest: JsonQuery,
    auth_dao: UserAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    let data = match path.0.to_string().as_str() {
        "add" => user_res_add(rest.param::<ResAddParam>()?, &auth_dao).await,
        "edit" => user_res_edit(rest.param::<ResEditParam>()?, &auth_dao).await,
        "delete" => user_res_delete(rest.param::<ResDeleteParam>()?, &auth_dao).await,
        "list_data" => user_res_list_data(rest.param::<ResListDataParam>()?, &auth_dao).await,
        "tags" => user_res_tags(rest.param::<ResTagsParam>()?, &auth_dao).await,
        "all" => rbac_all_res_list(&res_tpls(), rest.param::<ResAllParam>()?).await,
        name => handler_not_found!(name),
    };
    Ok(data?.into())
}

#[post("/role/{method}")]
pub async fn role<'t>(
    jwt: JwtQuery,
    path: actix_web::web::Path<(String,)>,
    rest: JsonQuery,
    auth_dao: UserAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    let data = match path.0.to_string().as_str() {
        "add" => user_role_add(rest.param::<RoleAddParam>()?, &auth_dao).await,
        "edit" => user_role_edit(rest.param::<RoleEditParam>()?, &auth_dao).await,
        "delete" => user_role_delete(rest.param::<RoleDeleteParam>()?, &auth_dao).await,
        "add_user" => user_role_add_user(rest.param::<RoleAddUserParam>()?, &auth_dao).await,
        "delete_user" => {
            user_role_delete_user(rest.param::<RoleDeleteUserParam>()?, &auth_dao).await
        }
        "list_user" => user_role_list_user(rest.param::<RoleListUserParam>()?, &auth_dao).await,
        "list_data" => user_role_list_data(rest.param::<RoleListDataParam>()?, &auth_dao).await,
        "options" => user_role_options(rest.param::<RoleOptionsParam>()?, &auth_dao).await,
        "relation" => user_relation_data(rest.param::<RoleRelationDataParam>()?, &auth_dao).await,
        "tags" => user_role_tags(rest.param::<RoleTagsParam>()?, &auth_dao).await,
        name => Err(lsys_web::JsonData::message(name).set_sub_code("method_not_found")),
    };
    Ok(data?.into())
}

#[post("/access/{method}")]
pub async fn access<'t>(
    jwt: JwtQuery,
    rest: JsonQuery,
    auth_dao: UserAuthQuery,
    path: actix_web::web::Path<(String,)>,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    let data = match path.0.to_string().as_str() {
        "check" => user_access_check(rest.param::<RbacAccessParam>()?, &auth_dao).await,
        "menu" => user_menu_check(rest.param::<RbacMenuParam>()?, &auth_dao).await,
        name => Err(lsys_web::JsonData::message(name).set_sub_code("method_not_found")),
    };
    Ok(data?.into())
}
