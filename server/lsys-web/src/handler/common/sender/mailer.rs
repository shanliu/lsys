use std::collections::HashMap;

use super::{tpl_config_del, tpl_config_list, MessageView, TplConfigDelParam, TplConfigListParam};
use crate::{
    dao::RequestAuthDao,
    handler::access::{AccessAppSenderDoMail, AccessAppSenderMailConfig, AccessAppSenderMailMsg},
    LimitParam, PageParam, {JsonData, JsonResult},
};
use lsys_app_sender::{
    // dao::SenderError,
    dao::SenderError,
    model::{SenderConfigStatus, SenderMailConfigType, SenderMailMessageStatus},
};
use lsys_core::now_time;
use lsys_core::{fluent_message, str_time};
use lsys_user::dao::auth::{SessionData, SessionTokenData, UserSession};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Debug, Deserialize)]
pub struct MailerMessageLogParam {
    pub message_id: String,
    pub count_num: Option<bool>,
    pub page: Option<PageParam>,
}

pub async fn mailer_message_log<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: MailerMessageLogParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let message_id = param
        .message_id
        .parse::<u64>()
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let msg = req_dao
        .web_dao
        .sender_mailer
        .mailer
        .mail_record
        .find_message_by_id(&message_id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let body = req_dao
        .web_dao
        .sender_mailer
        .mailer
        .mail_record
        .find_body_by_id(&msg.sender_body_id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessAppSenderMailMsg {
                user_id: req_auth.user_data().user_id,
                res_user_id: body.user_id,
                app_id: Some(body.app_id),
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let res = req_dao
        .web_dao
        .sender_mailer
        .mailer
        .mail_record
        .message_log_list(&message_id, &Some(param.page.unwrap_or_default().into()))
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .sender_mailer
                .mailer
                .mail_record
                .message_log_count(&message_id)
                .await
                .map_err(|e| req_dao.fluent_json_data(e))?,
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

pub async fn mailer_message_body<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: MailerMessageBodyParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let message_id = param
        .message_id
        .parse::<u64>()
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let msg = req_dao
        .web_dao
        .sender_mailer
        .mailer
        .mail_record
        .find_message_by_id(&message_id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let body = req_dao
        .web_dao
        .sender_mailer
        .mailer
        .mail_record
        .find_body_by_id(&msg.sender_body_id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessAppSenderMailMsg {
                user_id: req_auth.user_data().user_id,
                res_user_id: body.user_id,
                app_id: Some(body.app_id),
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .web_dao
        .logger
        .add(
            &MessageView {
                msg_type: "mail",
                id: message_id,
            },
            &Some(message_id),
            &Some(body.user_id),
            &Some(req_auth.user_data().user_id),
            None,
            Some(&req_dao.req_env),
        )
        .await;
    Ok(JsonData::data(json!({ "body": body.tpl_var})))
}

#[derive(Debug, Deserialize)]
pub struct MailerMessageListParam {
    pub user_id: Option<u64>,
    pub app_id: Option<u64>,
    pub tpl_id: Option<String>,
    pub status: Option<i8>,
    pub body_id: Option<u64>,
    pub snid: Option<String>,
    pub to_mail: Option<String>,
    pub count_num: Option<bool>,
    pub limit: Option<LimitParam>,
}

pub async fn mailer_message_list<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: MailerMessageListParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessAppSenderMailMsg {
                user_id: req_auth.user_data().user_id,
                res_user_id: param.user_id.unwrap_or(req_auth.user_data().user_id),
                app_id: param.app_id,
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let status = if let Some(e) = param.status {
        Some(SenderMailMessageStatus::try_from(e).map_err(|e| req_dao.fluent_json_data(e))?)
    } else {
        None
    };
    let res = req_dao
        .web_dao
        .sender_mailer
        .mailer
        .mail_record
        .message_list(
            &param.user_id,
            &param.app_id,
            &param.tpl_id,
            &param.body_id,
            &param.snid.as_ref().and_then(|e| e.parse::<u64>().ok()),
            &status,
            &param.to_mail,
            &Some(param.limit.unwrap_or_default().into()),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .sender_mailer
                .mailer
                .mail_record
                .message_count(
                    &param.user_id,
                    &param.app_id,
                    &param.tpl_id,
                    &param.body_id,
                    &param.snid.as_ref().and_then(|e| e.parse::<u64>().ok()),
                    &status,
                    &param.to_mail,
                )
                .await
                .map_err(|e| req_dao.fluent_json_data(e))?,
        )
    } else {
        None
    };
    let ntime = now_time().unwrap_or_default();
    let next = res.1;
    let res_data = req_dao
        .web_dao
        .sender_mailer
        .mailer
        .task_is_run(res.0.iter().map(|t| (&t.0.id, t)).collect::<Vec<_>>())
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
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

pub async fn mailer_message_cancel<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: MailerMessageCancelParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let message_id = param
        .message_id
        .parse::<u64>()
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let msg = req_dao
        .web_dao
        .sender_mailer
        .mailer
        .mail_record
        .find_message_by_id(&message_id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let body = req_dao
        .web_dao
        .sender_mailer
        .mailer
        .mail_record
        .find_body_by_id(&msg.sender_body_id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessAppSenderMailMsg {
                user_id: req_auth.user_data().user_id,
                res_user_id: body.user_id,
                app_id: Some(body.app_id),
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let mut res = req_dao
        .web_dao
        .sender_mailer
        .send_cancel(
            &body,
            &[&msg],
            req_auth.user_data().user_id,
            Some(&req_dao.req_env),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let mut out = None;
    if !res.is_empty() {
        if let Some(err) = res.remove(0).2 {
            return Err(req_dao.fluent_json_data(err));
        } else {
            out = Some(message_id.to_string())
        }
    }
    Ok(JsonData::data(json!({
        "data":out
    })))
}

#[derive(Debug, Deserialize)]
pub struct MailerConfigAddParam {
    pub app_id: u64,
    pub user_id: Option<u64>,
    pub priority: i8,
    pub config_type: i8,
    pub config_data: Value,
}

pub async fn mailer_config_add<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: MailerConfigAddParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let uid = param.user_id.unwrap_or(req_auth.user_data().user_id);
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessAppSenderMailConfig {
                user_id: req_auth.user_data().user_id,
                res_user_id: uid,
                app_id: param.app_id,
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let config_type = SenderMailConfigType::try_from(param.config_type)
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let id = req_dao
        .web_dao
        .sender_mailer
        .mailer
        .mail_record
        .config_add(
            Some(param.app_id),
            param.priority,
            config_type,
            param.config_data,
            uid,
            req_auth.user_data().user_id,
            Some(&req_dao.req_env),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::data(json!({ "id": id })))
}

#[derive(Debug, Deserialize)]
pub struct MailerConfigDeleteParam {
    pub config_id: u64,
}
pub async fn mailer_config_del<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: MailerConfigDeleteParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let mail_record = &req_dao.web_dao.sender_mailer.mailer.mail_record;
    let res = mail_record.find_config_by_id(&param.config_id).await;

    match res {
        Ok(config) => {
            if SenderConfigStatus::Enable.eq(config.status) {
                req_dao
                    .web_dao
                    .user
                    .rbac_dao
                    .rbac
                    .check(
                        &AccessAppSenderMailConfig {
                            user_id: req_auth.user_data().user_id,
                            res_user_id: config.user_id,
                            app_id: config.app_id,
                        },
                        None,
                    )
                    .await
                    .map_err(|e| req_dao.fluent_json_data(e))?;
                mail_record
                    .config_del(
                        &config,
                        req_auth.user_data().user_id,
                        Some(&req_dao.req_env),
                    )
                    .await
                    .map_err(|e| req_dao.fluent_json_data(e))?;
            }
        }
        Err(err) => match &err {
            SenderError::Sqlx(sqlx::Error::RowNotFound) => {
                return Ok(req_dao.fluent_json_data(err));
            }
            _ => {
                return Err(req_dao.fluent_json_data(err));
            }
        },
    }
    Ok(JsonData::default())
}

#[derive(Debug, Deserialize)]
pub struct MailerConfigListParam {
    pub user_id: Option<u64>,
    pub id: Option<u64>,
    pub app_id: Option<u64>,
}

pub async fn mailer_config_list<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: MailerConfigListParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessAppSenderMailConfig {
                user_id: req_auth.user_data().user_id,
                res_user_id: param.user_id.unwrap_or(req_auth.user_data().user_id),
                app_id: param.app_id.unwrap_or_default(),
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let data = req_dao
        .web_dao
        .sender_mailer
        .mailer
        .mail_record
        .config_list(param.user_id, param.id, param.app_id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let data = data
        .into_iter()
        .map(|(e, v)| {
            let config_data = match v {
                lsys_app_sender::model::SenderMailConfigData::Limit(t) => json!(&t).to_string(),
                lsys_app_sender::model::SenderMailConfigData::MaxOfSend(u) => u.to_string(),
                lsys_app_sender::model::SenderMailConfigData::Block { to } => to,
                lsys_app_sender::model::SenderMailConfigData::BlockDomain { domain } => domain,
                lsys_app_sender::model::SenderMailConfigData::PassTpl(val) => val,
                lsys_app_sender::model::SenderMailConfigData::Close => "".to_string(),
                lsys_app_sender::model::SenderMailConfigData::None => "".to_string(),
            };
            json!({
               "id": e.id,
               "app_id": e.app_id,
               "config_type": e.config_type,
               "add_time": e.change_time,
               "priority": e.priority,
               "config_data": config_data,
            })
        })
        .collect::<Vec<Value>>();

    Ok(JsonData::data(json!({ "data": data })))
}

pub async fn mailer_tpl_config_list<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: TplConfigListParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    tpl_config_list(
        param,
        &req_dao.web_dao.sender_mailer.mailer.tpl_config,
        req_dao,
    )
    .await
}

pub async fn mailer_tpl_config_del<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: TplConfigDelParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    tpl_config_del(
        param,
        &req_dao.web_dao.sender_mailer.mailer.tpl_config,
        req_dao,
    )
    .await
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
pub async fn mailer_message_send<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: MailerMessageSendParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let tpl = req_dao
        .web_dao
        .sender_mailer
        .mailer
        .tpl_config
        .find_by_id(&param.tpl_id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let app = req_dao
        .web_dao
        .app
        .app_dao
        .app
        .find_by_id(&tpl.app_id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    if req_auth.user_data().user_id != app.user_id {
        return Ok(req_dao.fluent_json_data(fluent_message!("mail-use-other-user-app")));
        //JsonData::message("can't use other user app")
    }
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
                    .map_err(|e| req_dao.fluent_json_data(e))?
                    .timestamp() as u64,
            )
        }
    } else {
        None
    };
    let to = param.to.iter().map(|e| e.as_str()).collect::<Vec<_>>();
    // 字符串转时间对象
    req_dao
        .web_dao
        .sender_mailer
        .app_send(
            &app,
            &tpl.tpl_id,
            &to,
            &param.data,
            &send_time,
            &param.reply,
            &param.max_try,
            Some(&req_dao.req_env),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::default())
}
