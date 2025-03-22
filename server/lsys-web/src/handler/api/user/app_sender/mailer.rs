use std::collections::HashMap;

use crate::{
    common::{JsonData, JsonError, JsonResult, PageParam, UserAuthQueryDao},
    dao::access::api::user::{CheckAppSenderMailMsg, CheckAppSenderMailSend},
};

use crate::common::LimitParam;
use lsys_access::dao::AccessSession;
use lsys_app_sender::model::SenderMailMessageStatus;
use lsys_core::now_time;
use lsys_core::str_time;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct MailerMessageLogParam {
    pub message_id: String,
    pub count_num: Option<bool>,
    pub page: Option<PageParam>,
}

pub async fn mailer_message_log(
    param: &MailerMessageLogParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let message_id = param.message_id.parse::<u64>()?;
    let msg = req_dao
        .web_dao
        .app_sender
        .mailer
        .mailer_dao
        .mail_record
        .find_message_by_id(&message_id)
        .await?;
    let body = req_dao
        .web_dao
        .app_sender
        .mailer
        .mailer_dao
        .mail_record
        .find_body_by_id(&msg.sender_body_id)
        .await?;
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &req_dao.req_env,
            Some(&auth_data),
            &CheckAppSenderMailMsg {
                res_user_id: body.user_id,
            },
        )
        .await?;
    let res = req_dao
        .web_dao
        .app_sender
        .mailer
        .mailer_dao
        .mail_record
        .message_log_list(message_id, param.page.as_ref().map(|e| e.into()).as_ref())
        .await?;
    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .app_sender
                .mailer
                .mailer_dao
                .mail_record
                .message_log_count(message_id)
                .await?,
        )
    } else {
        None
    };
    Ok(JsonData::data(json!({ "data": res,"total":count})))
}

#[derive(Debug, Deserialize)]
pub struct MailerMessageBodyParam {
    pub message_id: String,
}

pub async fn mailer_message_body(
    param: &MailerMessageBodyParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let message_id = param.message_id.parse::<u64>()?;
    let msg = req_dao
        .web_dao
        .app_sender
        .mailer
        .mailer_dao
        .mail_record
        .find_message_by_id(&message_id)
        .await?;
    let body = req_dao
        .web_dao
        .app_sender
        .mailer
        .mailer_dao
        .mail_record
        .find_body_by_id(&msg.sender_body_id)
        .await?;
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &req_dao.req_env,
            Some(&auth_data),
            &CheckAppSenderMailMsg {
                res_user_id: body.user_id,
            },
        )
        .await?;

    req_dao
        .web_dao
        .app_sender
        .mailer
        .mailer_message_body(&msg, &body, &auth_data, Some(&req_dao.req_env))
        .await?;
    Ok(JsonData::data(json!({ "body": body.tpl_var})))
}

#[derive(Debug, Deserialize)]
pub struct MailerMessageListParam {
    pub app_id: Option<u64>,
    pub tpl_id: Option<String>,
    pub status: Option<i8>,
    pub body_id: Option<u64>,
    pub snid: Option<String>,
    pub to_mail: Option<String>,
    pub count_num: Option<bool>,
    pub limit: Option<LimitParam>,
}

pub async fn mailer_message_list(
    param: &MailerMessageListParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &req_dao.req_env,
            Some(&auth_data),
            &CheckAppSenderMailMsg {
                res_user_id: auth_data.user_id(),
            },
        )
        .await?;
    let status = if let Some(e) = param.status {
        Some(SenderMailMessageStatus::try_from(e)?)
    } else {
        None
    };
    let res = req_dao
        .web_dao
        .app_sender
        .mailer
        .mailer_dao
        .mail_record
        .message_list(
            Some(auth_data.user_id()),
            param.app_id,
            param.tpl_id.as_deref(),
            param.body_id,
            param.snid.as_ref().and_then(|e| e.parse::<u64>().ok()),
            status,
            param.to_mail.as_deref(),
            param.limit.as_ref().map(|e| e.into()).as_ref(),
        )
        .await?;
    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .app_sender
                .mailer
                .mailer_dao
                .mail_record
                .message_count(
                    Some(auth_data.user_id()),
                    param.app_id,
                    param.tpl_id.as_deref(),
                    param.body_id,
                    param.snid.as_ref().and_then(|e| e.parse::<u64>().ok()),
                    status,
                    param.to_mail.as_deref(),
                )
                .await?,
        )
    } else {
        None
    };
    let ntime = now_time().unwrap_or_default();
    let next = res.1;
    let res_data = req_dao
        .web_dao
        .app_sender
        .mailer
        .mailer_dao
        .task_is_run(res.0.iter().map(|t| (&t.0.id, t)).collect::<Vec<_>>())
        .await?;
    let res = res_data
        .into_iter()
        .map(|(e, s)| {
            let now_send = SenderMailMessageStatus::Init.eq(e.0.status)
                && e.1
                    .as_ref()
                    .map(|t| t.expected_time <= ntime)
                    .unwrap_or(true);
            json!({
                "id":e.0.id,
                "snid":e.0.snid,
                "app_id":e.1.as_ref().map(|t|t.app_id),
                "tpl_id":e.1.as_ref().map(|t|t.tpl_id.to_owned()),
                "max_try_num":e.1.as_ref().map(|t|t.max_try_num),
                "add_time":e.1.as_ref().map(|t|t.add_time),
                "expected_time":e.1.as_ref().map(|t|t.expected_time),
                "to_mail":e.0.to_mail,
                "try_num":e.0.try_num,
                "status":e.0.status,
                "now_send":now_send,
                "on_task":s.is_some(),
                "send_time":e.0.send_time
            })
        })
        .collect::<Vec<_>>();
    Ok(JsonData::data(
        json!({ "data": res,"total":count,"next":next}),
    ))
}

#[derive(Debug, Deserialize)]
pub struct MailerMessageCancelParam {
    pub message_id: String,
}

pub async fn mailer_message_cancel(
    param: &MailerMessageCancelParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let message_id = param.message_id.parse::<u64>()?;
    let msg = req_dao
        .web_dao
        .app_sender
        .mailer
        .mailer_dao
        .mail_record
        .find_message_by_id(&message_id)
        .await?;
    let body = req_dao
        .web_dao
        .app_sender
        .mailer
        .mailer_dao
        .mail_record
        .find_body_by_id(&msg.sender_body_id)
        .await?;
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &req_dao.req_env,
            Some(&auth_data),
            &CheckAppSenderMailMsg {
                res_user_id: body.user_id,
            },
        )
        .await?;
    let mut res = req_dao
        .web_dao
        .app_sender
        .mailer
        .send_cancel(&body, &[&msg], auth_data.user_id(), Some(&req_dao.req_env))
        .await?;
    let mut out = None;
    if !res.is_empty() {
        if let Some(err) = res.remove(0).2 {
            return Err(err)?;
        } else {
            out = Some(message_id.to_string())
        }
    }
    Ok(JsonData::data(json!({
        "data":out
    })))
}

#[derive(Debug, Deserialize)]
pub struct MailerMessageSendParam {
    pub tpl_id: u64,
    pub data: HashMap<String, String>,
    pub to: Vec<String>,
    pub reply: Option<String>,
    pub send_time: Option<String>,
    pub max_try: Option<u8>,
}

//后台界面发送邮件接口
pub async fn mailer_message_send(
    param: &MailerMessageSendParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
   
    let tpl = req_dao
        .web_dao
        .app_sender
        .mailer
        .mailer_dao
        .tpl_config
        .find_by_id(&param.tpl_id)
        .await?;
    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .find_by_id(&tpl.app_id)
        .await?;
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
  
    req_dao
        .web_dao
        .web_rbac
        .check(
            &req_dao.req_env,
            Some(&auth_data),
            &CheckAppSenderMailSend {
                res_user_id: app.user_id,
            },
        )
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
    let to = param.to.iter().map(|e| e.as_str()).collect::<Vec<_>>();
    // 字符串转时间对象
    req_dao
        .web_dao
        .app_sender
        .mailer
        .app_send(
            &app,
            tpl.tpl_id.as_str(),
            &to,
            &param
                .data
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect::<HashMap<_, _>>(),
            send_time,
            param.reply.as_deref(),
            param.max_try,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::default())
}
