use std::collections::HashMap;

use crate::{
    common::{JsonData, JsonError, JsonResult, RequestDao},
    dao::{access::rest::CheckRestApp, APP_FEATURE_SMS},
};
use lsys_app::model::AppModel;
use lsys_core::{str_time, IntoFluentMessage};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Debug, Deserialize)]
pub struct SmsSendParam {
    pub area: Option<String>,
    pub mobile: Vec<String>,
    pub tpl: String,
    pub data: HashMap<String, String>,
    pub send_time: Option<String>,
    pub max_try: Option<u8>,
}
pub async fn sms_send(
    param: &SmsSendParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(
            &req_dao.access_env(),
            &CheckRestApp { app_id: app.id },
            None,
        )
        .await?;
    req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .cache()
        .exter_feature_check(app, &[APP_FEATURE_SMS])
        .await?;

    let send_time = if let Some(ref t) = param.send_time {
        if t.is_empty() {
            None
        } else {
            Some(str_time(t).map_err(JsonError::Message)?.timestamp() as u64)
        }
    } else {
        None
    };
    // 字符串转时间对象
    let mobile = param.mobile.iter().map(|e| e.as_str()).collect::<Vec<_>>();
    let data = req_dao
        .web_dao
        .app_sender
        .smser
        .app_send(
            app,
            &param.tpl,
            param.area.as_deref().unwrap_or("86"),
            &mobile,
            &param
                .data
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect(),
            send_time,
            param.max_try,
            Some(&req_dao.req_env),
        )
        .await?;
    let detail = data
        .into_iter()
        .map(|e| {
            json!({
                "snid":e.0.to_string(),
                "mobile":e.1,
            })
        })
        .collect::<Vec<Value>>();
    Ok(JsonData::data(json!(
       { "detail":detail}
    )))
}

#[derive(Debug, Deserialize)]
pub struct SmsCancelParam {
    pub snid_data: Vec<String>,
}
pub async fn sms_cancel(
    param: &SmsCancelParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonData> {
    let mut ids = Vec::with_capacity(param.snid_data.len());
    for e in param.snid_data.iter() {
        ids.push(e.parse::<u64>()?);
    }
    let data = req_dao
        .web_dao
        .app_sender
        .smser
        .app_send_cancel(app, &ids, Some(&req_dao.req_env))
        .await?;
    let detail = data
        .into_iter()
        .map(|e| {
            json!({
                "snid":e.0.to_string(),
                "status":!e.1&&e.2.is_none(),
                "msg":e.2.map(|e|e.to_fluent_message().default_format())
            })
        })
        .collect::<Vec<Value>>();
    Ok(JsonData::data(json!(
       { "detail":detail}
    )))
}
