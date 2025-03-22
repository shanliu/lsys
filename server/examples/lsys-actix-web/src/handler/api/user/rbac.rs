use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;

use lsys_web::handler::api::{user::rbac::user_role_add, user::rbac::UserRoleAddParam};

#[post("/role/{method}")]
pub async fn role(
    jwt: JwtQuery,
    path: actix_web::web::Path<String>,
    json_param: JsonQuery,
    auth_dao: UserAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    let data = match path.into_inner().as_str() {
        "add" => user_role_add(&json_param.param::<UserRoleAddParam>()?, &auth_dao).await,
        // "edit" => user_role_edit(json_param.param::<RoleEditParam>()?, &auth_dao).await,
        // "delete" => user_role_delete(json_param.param::<RoleDeleteParam>()?, &auth_dao).await,
        // "add_user" => user_role_add_user(json_param.param::<RoleAddUserParam>()?, &auth_dao).await,
        // "delete_user" => {
        //     user_role_delete_user(json_param.param::<RoleDeleteUserParam>()?, &auth_dao).await
        // }
        // "list_user" => {
        //     user_role_list_user(json_param.param::<RoleListUserParam>()?, &auth_dao).await
        // }
        // "list_data" => {
        //     user_role_list_data(json_param.param::<RoleListDataParam>()?, &auth_dao).await
        // }
        // "options" => user_role_options(json_param.param::<RoleOptionsParam>()?, &auth_dao).await,
        // "relation" => user_relation_data(json_param.param::<RoleKeyDataParam>()?, &auth_dao).await,
        // "tags" => user_role_tags(json_param.param::<RoleTagsParam>()?, &auth_dao).await,
        name => Err(lsys_web::common::JsonError::JsonData(
            lsys_web::common::JsonData::default().set_sub_code("method_not_found"),
            lsys_core::fluent_message!("method_not_found",{"msg":name}),
        )),
    };
    Ok(data
        .map_err(|e| auth_dao.fluent_error_json_data(&e))?
        .into())
}
