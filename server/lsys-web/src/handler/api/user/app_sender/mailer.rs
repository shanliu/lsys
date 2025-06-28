use crate::common::JsonData;
use crate::{
    common::{JsonError, JsonResponse, JsonResult, PageParam, UserAuthQueryDao},
    dao::access::api::system::user::{CheckUserAppSenderMailSend, CheckUserAppSenderMailView},
};
use std::collections::HashMap;

use crate::common::LimitParam;
use crate::dao::access::RbacAccessCheckEnv;
use lsys_access::dao::AccessSession;
use lsys_app_sender::model::SenderMailMessageStatus;
use lsys_core::now_time;
use lsys_core::str_time;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct MailerMessageLogParam {
    #[serde(default, deserialize_with = "crate::common::deserialize_u64")]
    pub message_id: u64,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
    pub page: Option<PageParam>,
}

pub async fn mailer_message_log(
    param: &MailerMessageLogParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let msg = req_dao
        .web_dao
        .app_sender
        .mailer
        .mailer_dao
        .mail_record
        .find_message_by_id(param.message_id)
        .await?;
    let body = req_dao
        .web_dao
        .app_sender
        .mailer
        .mailer_dao
        .mail_record
        .find_body_by_id(msg.sender_body_id)
        .await?;
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserAppSenderMailView {
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
        .message_log_list(
            param.message_id,
            param.page.as_ref().map(|e| e.into()).as_ref(),
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
                .message_log_count(param.message_id)
                .await?,
        )
    } else {
        None
    };
    Ok(JsonResponse::data(JsonData::body(
        json!({ "data": res,"total":count}),
    )))
}

#[derive(Debug, Deserialize)]
pub struct MailerMessageBodyParam {
    #[serde(default, deserialize_with = "crate::common::deserialize_u64")]
    pub message_id: u64,
}

pub async fn mailer_message_body(
    param: &MailerMessageBodyParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let msg = req_dao
        .web_dao
        .app_sender
        .mailer
        .mailer_dao
        .mail_record
        .find_message_by_id(param.message_id)
        .await?;
    let body = req_dao
        .web_dao
        .app_sender
        .mailer
        .mailer_dao
        .mail_record
        .find_body_by_id(msg.sender_body_id)
        .await?;
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserAppSenderMailView {
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
    Ok(JsonResponse::data(JsonData::body(
        json!({ "body": body.tpl_var}),
    )))
}

#[derive(Debug, Deserialize)]
pub struct MailerMessageListParam {
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u64")]
    pub app_id: Option<u64>,
    pub tpl_key: Option<String>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_i8")]
    pub status: Option<i8>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u64")]
    pub body_id: Option<u64>,
    pub snid: Option<String>,
    pub to_mail: Option<String>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
    pub limit: Option<LimitParam>,
}

pub async fn mailer_message_list(
    param: &MailerMessageListParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserAppSenderMailView {
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
            param.tpl_key.as_deref(),
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
                    param.tpl_key.as_deref(),
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
                "tpl_key":e.1.as_ref().map(|t|t.tpl_key.to_owned()),
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
    Ok(JsonResponse::data(JsonData::body(
        json!({ "data": res,"total":count,"next":next}),
    )))
}

#[derive(Debug, Deserialize)]
pub struct MailerMessageCancelParam {
    #[serde(default, deserialize_with = "crate::common::deserialize_u64")]
    pub message_id: u64,
}

pub async fn mailer_message_cancel(
    param: &MailerMessageCancelParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let msg = req_dao
        .web_dao
        .app_sender
        .mailer
        .mailer_dao
        .mail_record
        .find_message_by_id(param.message_id)
        .await?;
    let body = req_dao
        .web_dao
        .app_sender
        .mailer
        .mailer_dao
        .mail_record
        .find_body_by_id(msg.sender_body_id)
        .await?;
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserAppSenderMailView {
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
            out = Some(param.message_id.to_string())
        }
    }
    Ok(JsonResponse::data(JsonData::body(json!({
        "data":out
    }))))
}

#[derive(Debug, Deserialize)]
pub struct MailerMessageSendParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    pub tpl_key: String,
    pub data: HashMap<String, String>,
    pub to: Vec<String>,
    pub reply: Option<String>,
    pub send_time: Option<String>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u8")]
    pub max_try: Option<u8>,
}

//后台界面发送邮件接口
pub async fn mailer_message_send(
    param: &MailerMessageSendParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .find_by_id(param.app_id)
        .await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserAppSenderMailSend {
                res_user_id: app.user_id,
            },
        )
        .await?;
    app.app_status_check()?;

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
            param.tpl_key.as_str(),
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
    Ok(JsonResponse::default())
}
