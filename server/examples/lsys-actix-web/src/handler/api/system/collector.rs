use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};

use actix_web::post;

use lsys_web::handler::api::system::collector::RecordFilesParam;
use lsys_web::handler::api::system::collector::RecordLogsParam;
use lsys_web::handler::api::system::collector::ScriptAddParam;
use lsys_web::handler::api::system::collector::ScriptDeleteParam;
use lsys_web::handler::api::system::collector::ScriptEditParam;
use lsys_web::handler::api::system::collector::ScriptFilesParam;
use lsys_web::handler::api::system::collector::ScriptListParam;
use lsys_web::handler::api::system::collector::ScriptLogsParam;
use lsys_web::handler::api::system::collector::ScriptRecordsParam;
use lsys_web::handler::api::system::collector::ScriptStatusParam;
use lsys_web::handler::api::system::collector::SubmitTaskParam;
use lsys_web::handler::api::system::collector::mapping_data;
use lsys_web::handler::api::system::collector::record_files;
use lsys_web::handler::api::system::collector::record_logs;
use lsys_web::handler::api::system::collector::script_add;
use lsys_web::handler::api::system::collector::script_del;
use lsys_web::handler::api::system::collector::script_edit;
use lsys_web::handler::api::system::collector::script_files;
use lsys_web::handler::api::system::collector::script_logs;
use lsys_web::handler::api::system::collector::script_records;
use lsys_web::handler::api::system::collector::script_status;
use lsys_web::handler::api::system::collector::scripts;
use lsys_web::handler::api::system::collector::submit_task;

#[post("/{type}")]
pub async fn collector(
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
        "mapping" => mapping_data(&auth_dao).await,
        "scripts" => scripts(&json_param.param::<ScriptListParam>()?, &auth_dao).await,
        "script_add" => script_add(&json_param.param::<ScriptAddParam>()?, &auth_dao).await,
        "script_edit" => script_edit(&json_param.param::<ScriptEditParam>()?, &auth_dao).await,
        "script_status" => {
            script_status(&json_param.param::<ScriptStatusParam>()?, &auth_dao).await
        }
        "script_del" => script_del(&json_param.param::<ScriptDeleteParam>()?, &auth_dao).await,
        "script_records" => {
            script_records(&json_param.param::<ScriptRecordsParam>()?, &auth_dao).await
        }
        "script_files" => script_files(&json_param.param::<ScriptFilesParam>()?, &auth_dao).await,
        "script_logs" => script_logs(&json_param.param::<ScriptLogsParam>()?, &auth_dao).await,
        "submit_task" => submit_task(&json_param.param::<SubmitTaskParam>()?, &auth_dao).await,
        "record_files" => record_files(&json_param.param::<RecordFilesParam>()?, &auth_dao).await,
        "record_logs" => record_logs(&json_param.param::<RecordLogsParam>()?, &auth_dao).await,
        name => handler_not_found!(name),
    }
    .map_err(|e| auth_dao.fluent_error_json_response(&e))?
    .into())
}
