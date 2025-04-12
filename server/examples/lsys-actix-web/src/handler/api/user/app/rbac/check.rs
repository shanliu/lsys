use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use lsys_web::handler::api::user::rbac::{
    app_audit_data, app_res_info_from_session, app_res_info_from_user, app_res_list_from_user,
    app_res_session_role_data_from_res, app_res_user_data_from_res, app_res_user_from_res,
    app_res_user_from_user, AppAuditParam, AppResInfoFromUserParam, AppResListFromSessionParam,
    AppResListFromUserParam, AppResRoleFromResParam, AppResUserDataFromResParam,
    AppResUserFromUserParam, AppUserFromResParam,
};

#[post("/check/{method}")]
pub async fn check(
    jwt: JwtQuery,
    path: actix_web::web::Path<String>,
    json_param: JsonQuery,
    auth_dao: UserAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    let data = match path.into_inner().as_str() {
        "audit_data" => app_audit_data(&json_param.param::<AppAuditParam>()?, &auth_dao).await,
        "res_user_from_user" => {
            app_res_user_from_user(&json_param.param::<AppResUserFromUserParam>()?, &auth_dao).await
        }
        "res_info_from_user" => {
            app_res_info_from_user(&json_param.param::<AppResInfoFromUserParam>()?, &auth_dao).await
        }
        "res_list_from_user" => {
            app_res_list_from_user(&json_param.param::<AppResListFromUserParam>()?, &auth_dao).await
        }
        "res_info_from_session" => {
            app_res_info_from_session(
                &json_param.param::<AppResListFromSessionParam>()?,
                &auth_dao,
            )
            .await
        }
        "res_user_from_res" => {
            app_res_user_from_res(&json_param.param::<AppUserFromResParam>()?, &auth_dao).await
        }
        "res_session_role_data_from_res" => {
            app_res_session_role_data_from_res(
                &json_param.param::<AppResRoleFromResParam>()?,
                &auth_dao,
            )
            .await
        }
        "res_user_data_from_res" => {
            app_res_user_data_from_res(
                &json_param.param::<AppResUserDataFromResParam>()?,
                &auth_dao,
            )
            .await
        }
        name => handler_not_found!(name),
    };
    Ok(data
        .map_err(|e| auth_dao.fluent_error_json_response(&e))?
        .into())
}
