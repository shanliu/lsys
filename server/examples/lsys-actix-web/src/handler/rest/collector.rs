use crate::common::handler::{ResponseJson, ResponseJsonResult, RestQuery};
use actix_web::post;
use lsys_web::handler::rest::collector::{
    record_files, record_logs, records, status, trigger,
    RecordFilesParam, RecordLogsParam, RecordsParam, StatusParam, TriggerParam,
};

#[post("")]
pub(crate) async fn collector(rest: RestQuery) -> ResponseJsonResult<ResponseJson> {
    Ok(match rest.rfc.method.as_deref().unwrap_or_default() {
        "trigger" => {
            let param = rest.param::<TriggerParam>()?;
            trigger(&param, &rest.get_app().await?, &rest).await
        }
        "status" => {
            let param = rest.param::<StatusParam>()?;
            status(&param, &rest.get_app().await?, &rest).await
        }
        "records" => {
            let param = rest.param::<RecordsParam>()?;
            records(&param, &rest.get_app().await?, &rest).await
        }
        "record_files" => {
            let param = rest.param::<RecordFilesParam>()?;
            record_files(&param, &rest.get_app().await?, &rest).await
        }
        "record_logs" => {
            let param = rest.param::<RecordLogsParam>()?;
            record_logs(&param, &rest.get_app().await?, &rest).await
        }
        var => handler_not_found!(var),
    }
    .map_err(|e| rest.fluent_error_json_response(&e))?
    .into())
}
