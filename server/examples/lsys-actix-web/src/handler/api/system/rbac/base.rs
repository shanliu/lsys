use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use lsys_web::handler::api::system::rbac::{
    audit_data, check_res_info_from_session, check_res_info_from_user, check_res_list_from_user,
    check_res_role_data_from_res, check_res_user_data_from_res, check_res_user_from_res,
    check_res_user_from_user, mapping_data, AuditParam, ResInfoFromUserParam,
    ResListFromSessionParam, ResListFromUserParam, ResRoleFromResParam, ResUserDataFromResParam,
    ResUserFromUserParam, UserFromResParam,
};

#[post("/base/{method}")]
pub async fn base(
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
        "mapping" => mapping_data(&auth_dao).await,
        "audit_data" => audit_data(&json_param.param::<AuditParam>()?, &auth_dao).await,
        "check_res_user_from_user" => {
            check_res_user_from_user(&json_param.param::<ResUserFromUserParam>()?, &auth_dao).await
        }
        "check_res_info_from_user" => {
            check_res_info_from_user(&json_param.param::<ResInfoFromUserParam>()?, &auth_dao).await
        }
        "check_res_list_from_user" => {
            check_res_list_from_user(&json_param.param::<ResListFromUserParam>()?, &auth_dao).await
        }
        "check_res_info_from_session" => {
            check_res_info_from_session(&json_param.param::<ResListFromSessionParam>()?, &auth_dao)
                .await
        }
        "check_res_user_from_res" => {
            check_res_user_from_res(&json_param.param::<UserFromResParam>()?, &auth_dao).await
        }
        "check_res_role_data_from_res" => {
            check_res_role_data_from_res(&json_param.param::<ResRoleFromResParam>()?, &auth_dao)
                .await
        }
        "check_res_user_data_from_res" => {
            check_res_user_data_from_res(&json_param.param::<ResUserDataFromResParam>()?, &auth_dao)
                .await
        }
        name => handler_not_found!(name),
    };
    Ok(data
        .map_err(|e| auth_dao.fluent_error_json_response(&e))?
        .into())
}
