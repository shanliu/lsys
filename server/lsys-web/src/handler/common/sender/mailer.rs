use std::collections::HashMap;

use crate::{
    dao::RequestDao,
    handler::access::{AccessAppSenderDoMail, AccessAppSenderMailConfig, AccessAppSenderMailMsg},
    LimitParam, PageParam, {JsonData, JsonResult},
};
use lsys_core::{rand_str, str_time, RandType};
use lsys_sender::{
    dao::SenderError,
    model::{SenderConfigStatus, SenderMailConfigType, SenderMailMessageStatus},
};
use lsys_user::dao::auth::{SessionData, SessionTokenData, UserSession};
use serde::Deserialize;
use serde_json::{json, Value};

use super::{tpl_config_del, tpl_config_list, TplConfigDelParam, TplConfigListParam};

#[derive(Debug, Deserialize)]
pub struct MailerMessageLogParam {
    pub message_id: String,
    pub count_num: Option<bool>,
    pub page: Option<PageParam>,
}

pub async fn mailer_message_log<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: MailerMessageLogParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let message_id = param.message_id.parse::<u64>().map_err(JsonData::message)?;
    let data = req_dao
        .web_dao
        .sender_mailer
        .mail_record()
        .find_message_by_id(&message_id)
        .await?;
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessAppSenderMailMsg {
                user_id: req_auth.user_data().user_id,
                res_user_id: data.user_id,
                app_id: Some(data.app_id),
            },
            None,
        )
        .await?;

    let res = req_dao
        .web_dao
        .sender_mailer
        .mail_record()
        .message_log_list(&message_id, &Some(param.page.unwrap_or_default().into()))
        .await?;
    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .sender_mailer
                .mail_record()
                .message_log_count(&message_id)
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

pub async fn mailer_message_body<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: MailerMessageBodyParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let message_id = param.message_id.parse::<u64>().map_err(JsonData::message)?;
    let data = req_dao
        .web_dao
        .sender_mailer
        .mail_record()
        .find_message_by_id(&message_id)
        .await?;
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessAppSenderMailMsg {
                user_id: req_auth.user_data().user_id,
                res_user_id: data.user_id,
                app_id: Some(data.app_id),
            },
            None,
        )
        .await?;

    Ok(JsonData::data(json!({ "body": data.tpl_var})))
}

#[derive(Debug, Deserialize)]
pub struct MailerMessageListParam {
    pub user_id: Option<u64>,
    pub app_id: Option<u64>,
    pub tpl_id: Option<String>,
    pub status: Option<i8>,
    pub to_mail: Option<String>,
    pub count_num: Option<bool>,
    pub limit: Option<LimitParam>,
}

pub async fn mailer_message_list<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: MailerMessageListParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
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
        .await?;
    let status = if let Some(e) = param.status {
        Some(SenderMailMessageStatus::try_from(e)?)
    } else {
        None
    };
    let res = req_dao
        .web_dao
        .sender_mailer
        .mail_record()
        .message_list(
            &param.user_id,
            &param.app_id,
            &param.tpl_id,
            &status,
            &param.to_mail,
            &Some(param.limit.unwrap_or_default().into()),
        )
        .await?;
    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .sender_mailer
                .mail_record()
                .message_count(
                    &param.user_id,
                    &param.app_id,
                    &param.tpl_id,
                    &status,
                    &param.to_mail,
                )
                .await?,
        )
    } else {
        None
    };
    let next = res.1;
    let res = res
        .0
        .into_iter()
        .map(|e| {
            json!({
                "id":e.id,
                "app_id":e.app_id,
                "to_mail":e.to_mail,
                "tpl_id":e.tpl_id,
                "try_num":e.try_num,
                "max_try_num":e.max_try_num,
                "add_time":e.add_time,
                "status":e.status,
                "expected_time":e.expected_time,
                "send_time":e.send_time
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

pub async fn mailer_message_cancel<
    't,
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    param: MailerMessageCancelParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let message_id = param.message_id.parse::<u64>().map_err(JsonData::message)?;
    let data = req_dao
        .web_dao
        .sender_mailer
        .mail_record()
        .find_message_by_id(&message_id)
        .await?;
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessAppSenderMailMsg {
                user_id: req_auth.user_data().user_id,
                res_user_id: data.user_id,
                app_id: Some(data.app_id),
            },
            None,
        )
        .await?;

    req_dao
        .web_dao
        .sender_mailer
        .send_cancel(&data, req_auth.user_data().user_id, Some(&req_dao.req_env))
        .await?;
    Ok(JsonData::default())
}

#[derive(Debug, Deserialize)]
pub struct MailerConfigAddParam {
    pub app_id: u64,
    pub user_id: Option<u64>,
    pub priority: i8,
    pub config_type: i8,
    pub config_data: Value,
}

pub async fn mailer_config_add<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: MailerConfigAddParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
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
        .await?;

    let config_type = SenderMailConfigType::try_from(param.config_type)?;
    let id = req_dao
        .web_dao
        .sender_mailer
        .mail_record()
        .config_add(
            Some(param.app_id),
            param.priority,
            config_type,
            param.config_data,
            uid,
            req_auth.user_data().user_id,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::data(json!({ "id": id })))
}

#[derive(Debug, Deserialize)]
pub struct MailerConfigDeleteParam {
    pub config_id: u64,
}
pub async fn mailer_config_del<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: MailerConfigDeleteParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    let mail_record = req_dao.web_dao.sender_mailer.mail_record();
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
                    .await?;

                mail_record
                    .config_del(
                        &config,
                        req_auth.user_data().user_id,
                        Some(&req_dao.req_env),
                    )
                    .await?;
            }
        }
        Err(err) => match err {
            SenderError::Sqlx(sqlx::Error::RowNotFound) => {
                return Ok(JsonData::message("email not find"));
            }
            _ => {
                return Err(err.into());
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

pub async fn mailer_config_list<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: MailerConfigListParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;

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
        .await?;

    let data = req_dao
        .web_dao
        .sender_mailer
        .mail_record()
        .config_list(param.user_id, param.id, param.app_id)
        .await?;
    let data = data
        .into_iter()
        .map(|(e, v)| {
            let config_data = match v {
                lsys_sender::model::SenderMailConfigData::Limit(t) => json!(&t).to_string(),
                lsys_sender::model::SenderMailConfigData::MaxOfSend(u) => u.to_string(),
                lsys_sender::model::SenderMailConfigData::Block { to } => to,
                lsys_sender::model::SenderMailConfigData::BlockDomain { domain } => domain,
                lsys_sender::model::SenderMailConfigData::PassTpl(val) => val,
                lsys_sender::model::SenderMailConfigData::Close => "".to_string(),
                lsys_sender::model::SenderMailConfigData::None => "".to_string(),
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

pub async fn mailer_tpl_config_list<
    't,
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    param: TplConfigListParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    tpl_config_list(param, req_dao.web_dao.sender_mailer.tpl_config(), req_dao).await
}

pub async fn mailer_tpl_config_del<
    't,
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    param: TplConfigDelParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    tpl_config_del(param, req_dao.web_dao.sender_mailer.tpl_config(), req_dao).await
}

#[derive(Debug, Deserialize)]
pub struct MailerMessageSendParam {
    pub tpl_id: u64,
    pub data: HashMap<String, String>,
    pub to: Vec<String>,
    pub reply: Option<String>,
    pub send_time: Option<String>,
}

//后台界面发送邮件接口
pub async fn mailer_message_send<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: MailerMessageSendParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    let tpl = req_dao
        .web_dao
        .sender_mailer
        .tpl_config()
        .find_by_id(&param.tpl_id)
        .await?;
    let app = req_dao
        .web_dao
        .app
        .app_dao
        .app
        .find_by_id(&tpl.app_id)
        .await?;
    if req_auth.user_data().user_id != app.user_id {
        return Ok(JsonData::message("can't use other user app"));
    }
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(&AccessAppSenderDoMail { app: app.clone() }, None)
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
    req_dao
        .web_dao
        .sender_mailer
        .app_send(
            &app,
            &tpl.tpl_id,
            &param.to,
            &param.data,
            &send_time,
            &param.reply,
            &Some(rand_str(RandType::UpperHex, 12)),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::default())
}
