use std::collections::HashMap;

use crate::{dao::WebDao, handler::access::AccessAppSenderDoMail, JsonData, JsonResult};
use lsys_app::model::AppsModel;
use lsys_core::{str_time, RequestEnv};
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
    app_dao: &WebDao,
    app: &AppsModel,
    param: MailSendParam,
    env_data: Option<&RequestEnv>,
) -> JsonResult<JsonData> {
    app_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessAppSenderDoMail {
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
    let to = param.to.iter().map(|e| e.as_str()).collect::<Vec<_>>();
    let data = app_dao
        .sender_mailer
        .app_send(
            app,
            &param.tpl,
            &to,
            &param.data,
            &send_time,
            &param.reply,
            &param.max_try,
            env_data,
        )
        .await?;
    let detail = data
        .into_iter()
        .map(|e| {
            json!({
                "id":e.0.to_string(),
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
    pub id_data: Vec<String>,
}
pub async fn mail_cancel(
    app_dao: &WebDao,
    app: &AppsModel,
    param: MailCancelParam,
    env_data: Option<&RequestEnv>,
) -> JsonResult<JsonData> {
    app_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessAppSenderDoMail {
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
    Ok(JsonData::data(json!( {
        "detail":detail
    })))
}
