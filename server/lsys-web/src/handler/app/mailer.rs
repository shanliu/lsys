use std::collections::HashMap;

use crate::{dao::RequestDao, handler::access::AccessAppSenderDoMail, JsonData, JsonResult};
use lsys_app::model::AppsModel;
use lsys_core::{str_time, FluentMessage};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Debug, Deserialize)]
pub struct MailSendParam {
    pub to: Vec<String>,
    pub tpl: String,
    pub data: HashMap<String, String>,
    pub reply: Option<String>,
    pub send_time: Option<String>,
    pub max_try: Option<u8>,
}
pub async fn mail_send(
    req_dao: &RequestDao,
    app: &AppsModel,
    param: MailSendParam,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessAppSenderDoMail {
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
                    .map_err(|e| req_dao.fluent_json_data(FluentMessage::from(e)))?
                    .timestamp() as u64,
            )
        }
    } else {
        None
    };
    // 字符串转时间对象
    let to = param.to.iter().map(|e| e.as_str()).collect::<Vec<_>>();
    let data = req_dao
        .web_dao
        .sender_mailer
        .app_send(
            app,
            &param.tpl,
            &to,
            &param.data,
            &send_time,
            &param.reply,
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
                "mail":e.1,
            })
        })
        .collect::<Vec<Value>>();
    Ok(JsonData::data(json!( {
        "detail":detail
    })))
}

#[derive(Debug, Deserialize)]
pub struct MailCancelParam {
    pub snid_data: Vec<String>,
}
pub async fn mail_cancel(
    req_dao: &RequestDao,
    app: &AppsModel,
    param: MailCancelParam,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessAppSenderDoMail {
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
        .sender_mailer
        .app_send_cancel(app, &ids, Some(&req_dao.req_env))
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let detail = data
        .into_iter()
        .map(|e| {
            json!({
                "id":e.0.to_string(),
                "status":!e.1&&e.2.is_none(),
              //  "sending":e.1,
                "msg":e.2.map(|e|e.to_string())
            })
        })
        .collect::<Vec<Value>>();
    Ok(JsonData::data(json!( {
        "detail":detail
    })))
}
