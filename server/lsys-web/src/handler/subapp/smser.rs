use std::collections::HashMap;

use crate::{dao::WebDao, handler::access::AccessAppSenderDoSms, JsonData, JsonResult};
use lsys_app::model::AppsModel;
use lsys_core::{str_time, RequestEnv};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Debug, Deserialize)]
pub struct SmsSendParam {
    pub area: Option<String>,
    pub mobile: Vec<String>,
    pub tpl: String,
    pub data: HashMap<String, String>,
    pub max_try: Option<u8>,
    pub send_time: Option<String>,
}
pub async fn sms_send(
    app_dao: &WebDao,
    app: &AppsModel,
    param: SmsSendParam,
    env_data: Option<&RequestEnv>,
) -> JsonResult<JsonData> {
    app_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessAppSenderDoSms {
                app: app.to_owned(),
            },
            None,
        )
        .await?;

    let send_time = if let Some(t) = param.send_time {
        if t.is_empty() {
            None
        } else {
            Some(str_time(&t)?.timestamp() as u64)
        }
    } else {
        None
    };
    // 字符串转时间对象
    let mobile = param.mobile.iter().map(|e| e.as_str()).collect::<Vec<_>>();
    let data = app_dao
        .sender_smser
        .app_send(
            app,
            &param.tpl,
            param.area.as_deref().unwrap_or("86"),
            &mobile,
            &param.data,
            &send_time,
            &param.max_try,
            env_data,
        )
        .await?;
    let detail = data
        .into_iter()
        .map(|e| {
            json!({
                "id":e.0.to_string(),
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
    pub id_data: Vec<String>,
}
pub async fn sms_cancel(
    app_dao: &WebDao,
    app: &AppsModel,
    param: SmsCancelParam,
    env_data: Option<&RequestEnv>,
) -> JsonResult<JsonData> {
    app_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessAppSenderDoSms {
                app: app.to_owned(),
            },
            None,
        )
        .await?;
    let mut ids = Vec::with_capacity(param.id_data.len());
    for e in param.id_data {
        ids.push(e.parse::<u64>().map_err(JsonData::message)?);
    }
    let data = app_dao
        .sender_smser
        .app_send_cancel(app, &ids, env_data)
        .await?;
    let detail = data
        .into_iter()
        .map(|e| {
            json!({
                "id":e.0.to_string(),
                "sending":e.1,
            })
        })
        .collect::<Vec<Value>>();
    Ok(JsonData::data(json!(
       { "detail":detail}
    )))
}
