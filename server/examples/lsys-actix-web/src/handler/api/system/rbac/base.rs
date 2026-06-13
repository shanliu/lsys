use crate::common::handler::{
    BearerQuery, JsonQuery, ReqQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use actix_web::web::Data;
use lsys_web::dao::WebDao;
use lsys_web::handler::api::system::rbac::{
    AuditParam, ResInfoFromUserParam, ResListFromSessionParam, ResListFromUserParam,
    ResRoleFromResParam, ResUserDataFromResParam, ResUserFromUserParam, UserFromResParam,
    audit_data, check_res_info_from_session, check_res_info_from_user, check_res_list_from_user,
    check_res_role_data_from_res, check_res_user_data_from_res, check_res_user_from_res,
    check_res_user_from_user, mapping_data,
};

#[post("/base/{method}")]
pub async fn base(
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
        "mapping" => mapping_data(&req_query).await,
        "audit_data" => audit_data(&json_param.param::<AuditParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "check_res_user_from_user" => {
            check_res_user_from_user(&json_param.param::<ResUserFromUserParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "check_res_info_from_user" => {
            check_res_info_from_user(&json_param.param::<ResInfoFromUserParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "check_res_list_from_user" => {
            check_res_list_from_user(&json_param.param::<ResListFromUserParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "check_res_info_from_session" => {
            check_res_info_from_session(&json_param.param::<ResListFromSessionParam>()?, &req_query, &auth_dao, web_dao.as_ref())
                .await
        }
        "check_res_user_from_res" => {
            check_res_user_from_res(&json_param.param::<UserFromResParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "check_res_role_data_from_res" => {
            check_res_role_data_from_res(&json_param.param::<ResRoleFromResParam>()?, &req_query, &auth_dao, web_dao.as_ref())
                .await
        }
        "check_res_user_data_from_res" => {
            check_res_user_data_from_res(&json_param.param::<ResUserDataFromResParam>()?, &req_query, &auth_dao, web_dao.as_ref())
                .await
        }
        name => handler_not_found!(name),
    };
    Ok(data
        .map_err(|e| req_query.fluent_error_json_response(&e))?
        .into())
}
