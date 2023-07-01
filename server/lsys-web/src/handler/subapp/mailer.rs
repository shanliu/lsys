use std::collections::HashMap;

use crate::{dao::WebDao, handler::access::AccessAppSenderDoMail, JsonData, JsonResult};
use lsys_app::model::AppsModel;
use lsys_core::{str_time, RequestEnv};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct MailSendParam {
    pub to: Vec<String>,
    pub tpl: String,
    pub data: HashMap<String, String>,
    pub reply: Option<String>,
    pub cancel: Option<String>,
    pub send_time: Option<String>,
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

    app_dao
        .sender_mailer
        .app_send(
            app,
            &param.tpl,
            &param.to,
            &param.data,
            &send_time,
            &param.reply,
            &param.cancel,
            env_data,
        )
        .await?;
    Ok(JsonData::default())
}

#[derive(Debug, Deserialize)]
pub struct MailCancelParam {
    pub cancel: String,
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
    app_dao
        .sender_smser
        .app_send_cancel(app, &param.cancel, env_data)
        .await?;
    Ok(JsonData::default())
}
