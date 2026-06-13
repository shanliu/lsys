use crate::common::handler::{ResponseJson, ResponseJsonResult, RestQuery, ReqQuery};
use actix_web::{post, web};
use lsys_web::dao::WebDao;
use lsys_web::handler::rest::collector::{
    RecordFilesParam, RecordLogsParam, RecordsParam, StatusParam, TriggerParam, record_files,
    record_logs, records, status, trigger,
};

#[post("")]
pub(crate) async fn collector(rest: RestQuery, req_dao: ReqQuery, web_dao: web::Data<WebDao>) -> ResponseJsonResult<ResponseJson> {
    Ok(match rest.rfc.method.as_deref().unwrap_or_default() {
        "trigger" => {
            let param = rest.param::<TriggerParam>()?;
            trigger(&param, &rest.get_app().await?, &req_dao, web_dao.as_ref()).await
        }
        "status" => {
            let param = rest.param::<StatusParam>()?;
            status(&param, &rest.get_app().await?, &req_dao, web_dao.as_ref()).await
        }
        "records" => {
            let param = rest.param::<RecordsParam>()?;
            records(&param, &rest.get_app().await?, &req_dao, web_dao.as_ref()).await
        }
        "record_files" => {
            let param = rest.param::<RecordFilesParam>()?;
            record_files(&param, &rest.get_app().await?, &req_dao, web_dao.as_ref()).await
        }
        "record_logs" => {
            let param = rest.param::<RecordLogsParam>()?;
            record_logs(&param, &rest.get_app().await?, &req_dao, web_dao.as_ref()).await
        }
        var => handler_not_found!(var),
    }
    .map_err(|e| req_dao.fluent_error_json_response(&e))?
    .into())
}
