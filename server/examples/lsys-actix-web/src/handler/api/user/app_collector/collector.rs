use crate::common::handler::{
    JsonQuery, BearerQuery, ReqQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};


use actix_web::post;
use actix_web::web::Data;
use lsys_web::dao::WebDao;

use lsys_web::handler::api::user::app_collector::RecordFilesParam;
use lsys_web::handler::api::user::app_collector::RecordLogsParam;

use lsys_web::handler::api::user::app_collector::ScriptAddParam;
use lsys_web::handler::api::user::app_collector::ScriptDeleteParam;
use lsys_web::handler::api::user::app_collector::ScriptDetailParam;
use lsys_web::handler::api::user::app_collector::ScriptEditParam;
use lsys_web::handler::api::user::app_collector::ScriptFilesParam;
use lsys_web::handler::api::user::app_collector::ScriptListParam;
use lsys_web::handler::api::user::app_collector::ScriptLogsParam;
use lsys_web::handler::api::user::app_collector::ScriptRecordsParam;
use lsys_web::handler::api::user::app_collector::ScriptStatusParam;
use lsys_web::handler::api::user::app_collector::SubmitTaskParam;
use lsys_web::handler::api::user::app_collector::mapping_data;

use lsys_web::handler::api::user::app_collector::record_file_list;
use lsys_web::handler::api::user::app_collector::record_logs;
use lsys_web::handler::api::user::app_collector::script_add;
use lsys_web::handler::api::user::app_collector::script_del;
use lsys_web::handler::api::user::app_collector::script_detail;
use lsys_web::handler::api::user::app_collector::script_edit;
use lsys_web::handler::api::user::app_collector::script_files;
use lsys_web::handler::api::user::app_collector::script_logs;
use lsys_web::handler::api::user::app_collector::script_records;
use lsys_web::handler::api::user::app_collector::script_status;
use lsys_web::handler::api::user::app_collector::scripts;
use lsys_web::handler::api::user::app_collector::submit_task;

#[post("/{type}")]
pub async fn collector(
    auth_dao: UserAuthQuery,
    path: actix_web::web::Path<String>,
    json_param: JsonQuery,
    bearer: BearerQuery,
    req_query: ReqQuery,
    web_dao: Data<WebDao>,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao
        .set_request_token(&bearer)
        .await
        .map_err(|e| req_query.fluent_error_json_response(&e))?;
    Ok(match path.into_inner().as_str() {
        "mapping" => mapping_data(&req_query).await,
        "scripts" => scripts(&json_param.param::<ScriptListParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "script_add" => script_add(&json_param.param::<ScriptAddParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "script_edit" => script_edit(&json_param.param::<ScriptEditParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "script_status" => {
            script_status(&json_param.param::<ScriptStatusParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "script_del" => script_del(&json_param.param::<ScriptDeleteParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "script_detail" => {
            script_detail(&json_param.param::<ScriptDetailParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "script_records" => {
            script_records(&json_param.param::<ScriptRecordsParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "script_files" => script_files(&json_param.param::<ScriptFilesParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "script_logs" => script_logs(&json_param.param::<ScriptLogsParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "submit_task" => submit_task(&json_param.param::<SubmitTaskParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "record_files" => record_file_list(&json_param.param::<RecordFilesParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "record_logs" => record_logs(&json_param.param::<RecordLogsParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        name => handler_not_found!(name),
    }
    .map_err(|e| req_query.fluent_error_json_response(&e))?
    .into())
}

