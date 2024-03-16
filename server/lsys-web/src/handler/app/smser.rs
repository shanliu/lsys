use std::collections::HashMap;

use crate::{dao::RequestDao, handler::access::AccessAppSenderDoSms, JsonData, JsonResult};
use lsys_app::model::AppsModel;
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
    req_dao: &RequestDao,
    app: &AppsModel,
    param: SmsSendParam,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessAppSenderDoSms {
                app_id: app.id,
                user_id: app.user_id,
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;

    let send_time = if let Some(t) = param.send_time {
        if t.is_empty() {
            None
        } else {
            Some(
                str_time(&t)
                    .map_err(|e| req_dao.fluent_json_data(e))?
                    .timestamp() as u64,
            )
        }
    } else {
        None
    };
    // 字符串转时间对象
    let mobile = param.mobile.iter().map(|e| e.as_str()).collect::<Vec<_>>();
    let data = req_dao
        .web_dao
        .sender_smser
        .app_send(
            app,
            &param.tpl,
            param.area.as_deref().unwrap_or("86"),
            &mobile,
            &param.data,
            &send_time,
            &param.max_try,
            Some(&req_dao.req_env),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
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
    req_dao: &RequestDao,
    app: &AppsModel,
    param: SmsCancelParam,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessAppSenderDoSms {
                app_id: app.id,
                user_id: app.user_id,
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let mut ids = Vec::with_capacity(param.snid_data.len());
    for e in param.snid_data {
        ids.push(e.parse::<u64>().map_err(|e| req_dao.fluent_json_data(e))?);
    }
    let data = req_dao
        .web_dao
        .sender_smser
        .app_send_cancel(app, &ids, Some(&req_dao.req_env))
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let detail = data
        .into_iter()
        .map(|e| {
            json!({
                "id":e.0.to_string(),
                "status":!e.1&&e.2.is_none(),
               // "sending":e.1,
                "msg":e.2.map(|e|e.to_fluent_message().default_format())
            })
        })
        .collect::<Vec<Value>>();
    Ok(JsonData::data(json!(
       { "detail":detail}
    )))
}
