use crate::common::handler::{
    JsonQuery, BearerQuery, ReqQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use actix_web::web::Data;
use lsys_web::dao::WebDao;
use lsys_web::handler::api::user::app_rbac::{
    AppAuditParam, AppResInfoFromUserParam, AppResListFromSessionParam, AppResListFromUserParam,
    AppResRoleFromResParam, AppResUserDataFromResParam, AppResUserFromUserParam,
    AppUserFromResParam, app_audit_data, app_res_info_from_session, app_res_info_from_user,
    app_res_list_from_user, app_res_session_role_data_from_res, app_res_user_data_from_res,
    app_res_user_from_res, app_res_user_from_user, mapping_data,
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
        "audit_data" => app_audit_data(&json_param.param::<AppAuditParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "res_user_from_user" => {
            app_res_user_from_user(&json_param.param::<AppResUserFromUserParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "res_info_from_user" => {
            app_res_info_from_user(&json_param.param::<AppResInfoFromUserParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "res_list_from_user" => {
            app_res_list_from_user(&json_param.param::<AppResListFromUserParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "res_info_from_session" => {
            app_res_info_from_session(
                &json_param.param::<AppResListFromSessionParam>()?,
                &req_query,
                &auth_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "res_user_from_res" => {
            app_res_user_from_res(&json_param.param::<AppUserFromResParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "res_session_role_data_from_res" => {
            app_res_session_role_data_from_res(
                &json_param.param::<AppResRoleFromResParam>()?,
                &req_query,
                &auth_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "res_user_data_from_res" => {
            app_res_user_data_from_res(
                &json_param.param::<AppResUserDataFromResParam>()?,
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
