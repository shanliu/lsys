use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use lsys_web::handler::api::user::app_export_task::app_export_active_count;
use lsys_web::handler::api::user::app_export_task::app_export_list;
use lsys_web::handler::api::user::app_export_task::app_export_submit;
use lsys_web::handler::api::user::app_export_task::app_export_task_mapping;
use lsys_web::handler::api::user::app_export_task::ExportActiveCountParam;
use lsys_web::handler::api::user::app_export_task::ExportListParam;
use lsys_web::handler::api::user::app_export_task::ExportSubmitParam;

#[post("/{type}")]
pub async fn export_task(
    auth_dao: UserAuthQuery,
    path: actix_web::web::Path<String>,
    json_param: JsonQuery,
    jwt: JwtQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao
        .set_request_token(&jwt)
        .await
        .map_err(|e| auth_dao.fluent_error_json_response(&e))?;
    Ok(match path.into_inner().as_str() {
        "mapping" => app_export_task_mapping(&auth_dao).await,
        "export_active_count" => {
            app_export_active_count(&json_param.param::<ExportActiveCountParam>()?, &auth_dao).await
        }
        "export_submit" => {
            app_export_submit(&json_param.param::<ExportSubmitParam>()?, &auth_dao).await
        }
        "export_list" => app_export_list(&json_param.param::<ExportListParam>()?, &auth_dao).await,
        name => handler_not_found!(name),
    }
    .map_err(|e| auth_dao.fluent_error_json_response(&e))?
    .into())
}
