use std::collections::HashMap;

use crate::{
    dao::RequestAuthDao,
    handler::access::{AccessAppSenderDoSms, AccessAppSenderSmsConfig, AccessAppSenderSmsMsg},
    LimitParam, PageParam, {JsonData, JsonResult},
};
use log::warn;
use lsys_app::{dao::app::AppDataWhere, model::AppStatus};
use lsys_core::{fluent_message, now_time, str_time, IntoFluentMessage};
use lsys_notify::dao::NotifyData;
use lsys_sender::{
    dao::{NotifySmsItem, SenderError},
    model::{SenderConfigStatus, SenderSmsConfigType, SenderSmsMessageStatus},
};
use lsys_user::dao::auth::{SessionData, SessionTokenData, UserSession};
use serde::Deserialize;
use serde_json::{json, Value};

use super::{tpl_config_del, tpl_config_list, MessageView, TplConfigDelParam, TplConfigListParam};

#[derive(Debug, Deserialize)]
pub struct SmserMessageLogParam {
    pub message_id: String,
    pub count_num: Option<bool>,
    pub page: Option<PageParam>,
}

pub async fn smser_message_log<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: SmserMessageLogParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let message_id = param
        .message_id
        .parse::<u64>()
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let msg = req_dao
        .web_dao
        .sender_smser
        .smser
        .sms_record
        .find_message_by_id(&message_id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let body = req_dao
        .web_dao
        .sender_smser
        .smser
        .sms_record
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
            &AccessAppSenderSmsMsg {
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
        .sender_smser
        .smser
        .sms_record
        .message_log_list(&message_id, &Some(param.page.unwrap_or_default().into()))
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .sender_smser
                .smser
                .sms_record
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
pub struct SmserMessageBodyParam {
    pub message_id: String,
}

pub async fn smser_message_body<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: SmserMessageBodyParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let message_id = param
        .message_id
        .parse::<u64>()
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let msg = req_dao
        .web_dao
        .sender_smser
        .smser
        .sms_record
        .find_message_by_id(&message_id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let body = req_dao
        .web_dao
        .sender_smser
        .smser
        .sms_record
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
            &AccessAppSenderSmsMsg {
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
                msg_type: "sms",
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
pub struct SmserMessageListParam {
    pub user_id: Option<u64>,
    pub app_id: Option<u64>,
    pub tpl_id: Option<String>,
    pub body_id: Option<u64>,
    pub snid: Option<String>,
    pub status: Option<i8>,
    pub mobile: Option<String>,
    pub count_num: Option<bool>,
    pub limit: Option<LimitParam>,
}

pub async fn smser_message_list<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: SmserMessageListParam,
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
            &AccessAppSenderSmsMsg {
                user_id: req_auth.user_data().user_id,
                res_user_id: param.user_id.unwrap_or(req_auth.user_data().user_id),
                app_id: param.app_id,
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let status = if let Some(e) = param.status {
        Some(SenderSmsMessageStatus::try_from(e).map_err(|e| req_dao.fluent_json_data(e))?)
    } else {
        None
    };

    let res = req_dao
        .web_dao
        .sender_smser
        .smser
        .sms_record
        .message_list(
            &param.user_id,
            &param.app_id,
            &param.tpl_id,
            &param.body_id,
            &param.snid.as_ref().and_then(|e| e.parse::<u64>().ok()),
            &status,
            &param.mobile,
            &Some(param.limit.unwrap_or_default().into()),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .sender_smser
                .smser
                .sms_record
                .message_count(
                    &param.user_id,
                    &param.app_id,
                    &param.tpl_id,
                    &param.body_id,
                    &param.snid.as_ref().and_then(|e| e.parse::<u64>().ok()),
                    &status,
                    &param.mobile,
                )
                .await
                .map_err(|e| req_dao.fluent_json_data(e))?,
        )
    } else {
        None
    };

    if let Err(err) = req_dao
        .web_dao
        .sender_smser
        .smser
        .add_status_query(
            &res.0
                .iter()
                .flat_map(|(m, b)| if b.is_some() { Some(m) } else { None })
                .collect::<Vec<_>>(),
        )
        .await
    {
        warn!(
            "query status fail:{}",
            err.to_fluent_message().default_format()
        );
    }
    let ntime = now_time().unwrap_or_default();
    let next = res.1;
    let res_data = req_dao
        .web_dao
        .sender_smser
        .smser
        .task_is_run(res.0.iter().map(|t| (&t.0.id, t)).collect::<Vec<_>>())
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let res = res_data
        .into_iter()
        .map(|(e, s)| {
            let now_send = SenderSmsMessageStatus::Init.eq(e.0.status)
                && e.1
                    .as_ref()
                    .map(|t| t.expected_time <= ntime)
                    .unwrap_or(true);

            json!({
                "id":e.0.id,
                "snid":e.0.snid,
                "app_id":e.1.as_ref().map(|t|t.app_id),
                "mobile":format!("{}-{}",e.0.area,e.0.mobile),
                "tpl_id":e.1.as_ref().map(|t|t.tpl_id.to_owned()),
                "try_num":e.0.try_num,
                "max_try_num":e.1.as_ref().map(|t|t.max_try_num),
                "add_time":e.1.as_ref().map(|t|t.add_time),
                "status":e.0.status,
                "now_send":now_send,
                "on_task":s.is_some(),
                "expected_time":e.1.as_ref().map(|t|t.expected_time),
               "send_time":e.0.send_time
            })
        })
        .collect::<Vec<_>>();
    Ok(JsonData::data(
        json!({ "data": res,"total":count,"next":next}),
    ))
}
#[derive(Debug, Deserialize)]
pub struct SmserMessageCancelParam {
    pub message_id: String,
}

pub async fn smser_message_cancel<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: SmserMessageCancelParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let message_id = param
        .message_id
        .parse::<u64>()
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let msg = req_dao
        .web_dao
        .sender_smser
        .smser
        .sms_record
        .find_message_by_id(&message_id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let body = req_dao
        .web_dao
        .sender_smser
        .smser
        .sms_record
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
            &AccessAppSenderSmsMsg {
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
        .sender_smser
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
pub struct SmserConfigAddParam {
    pub app_id: u64,
    pub user_id: Option<u64>,
    pub priority: i8,
    pub config_type: i8,
    pub config_data: Value,
}

pub async fn smser_config_add<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: SmserConfigAddParam,
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
            &AccessAppSenderSmsConfig {
                user_id: req_auth.user_data().user_id,
                res_user_id: uid,
                app_id: param.app_id,
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let config_type = SenderSmsConfigType::try_from(param.config_type)
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let id = req_dao
        .web_dao
        .sender_smser
        .smser
        .sms_record
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
pub struct SmserConfigDeleteParam {
    pub config_id: u64,
}
pub async fn smser_config_del<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: SmserConfigDeleteParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let sms_record = &req_dao.web_dao.sender_smser.smser.sms_record;
    let res = sms_record.find_config_by_id(&param.config_id).await;

    match res {
        Ok(config) => {
            if SenderConfigStatus::Enable.eq(config.status) {
                req_dao
                    .web_dao
                    .user
                    .rbac_dao
                    .rbac
                    .check(
                        &AccessAppSenderSmsConfig {
                            user_id: req_auth.user_data().user_id,
                            res_user_id: config.user_id,
                            app_id: config.app_id,
                        },
                        None,
                    )
                    .await
                    .map_err(|e| req_dao.fluent_json_data(e))?;
                sms_record
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
                // return Ok(JsonData::message("email not find"));
            }
            _ => {
                return Err(req_dao.fluent_json_data(err));
            }
        },
    }
    Ok(JsonData::default())
}

#[derive(Debug, Deserialize)]
pub struct SmserConfigListParam {
    pub user_id: Option<u64>,
    pub id: Option<u64>,
    pub app_id: Option<u64>,
}

pub async fn smser_config_list<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: SmserConfigListParam,
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
            &AccessAppSenderSmsConfig {
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
        .sender_smser
        .smser
        .sms_record
        .config_list(param.user_id, param.id, param.app_id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let data = data
        .into_iter()
        .map(|(e, v)| {
            let config_data = match v {
                lsys_sender::model::SenderSmsConfigData::Limit(t) => json!(&t).to_string(),
                lsys_sender::model::SenderSmsConfigData::MaxOfSend(u) => u.to_string(),
                lsys_sender::model::SenderSmsConfigData::Block { area, mobile } => {
                    format!("{}{}", area, mobile)
                }
                lsys_sender::model::SenderSmsConfigData::PassTpl(val) => val,
                lsys_sender::model::SenderSmsConfigData::Close => "".to_string(),
                lsys_sender::model::SenderSmsConfigData::None => "".to_string(),
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

pub async fn smser_notify_get_config<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
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
            &AccessAppSenderSmsConfig {
                user_id: req_auth.user_data().user_id,
                res_user_id: req_auth.user_data().user_id,
                app_id: 0,
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let app_param = AppDataWhere {
        user_id: &Some(req_auth.user_data().user_id),
        status: &Some(vec![AppStatus::Ok]),
        client_ids: &None,
        app_ids: &None,
    };
    let apps = req_dao
        .web_dao
        .app
        .app_dao
        .app
        .app_data(&app_param, &None)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let notify = req_dao
        .web_dao
        .notify
        .record
        .find_config_by_apps(
            &apps.iter().map(|e| e.id).collect::<Vec<_>>(),
            &NotifySmsItem::method(),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let data = apps
        .into_iter()
        .map(|e| {
            let n = notify.iter().find(|t| t.app_id == e.id);
            let url = n.map(|t| &t.call_url);
            let change_time = n.map(|t| {
                if t.change_time > 0 {
                    t.change_time
                } else {
                    t.create_time
                }
            });
            let change_user_id = n.map(|t| t.change_user_id);
            json!({
                "app_id":e.id,
                "app_name":e.name,
                 "call_url":url,
                 "change_time":change_time,
                 "change_user_id":change_user_id,
            })
        })
        .collect::<Vec<_>>();
    Ok(JsonData::data(json!({
        "data":data,
    })))
}

#[derive(Debug, Deserialize)]
pub struct SmserNotifyConfigParam {
    pub user_id: Option<u64>,
    pub app_id: u64,
    pub url: String,
}

pub async fn smser_notify_set_config<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: SmserNotifyConfigParam,
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
            &AccessAppSenderSmsConfig {
                user_id: req_auth.user_data().user_id,
                res_user_id: param.user_id.unwrap_or(req_auth.user_data().user_id),
                app_id: param.app_id,
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .web_dao
        .notify
        .record
        .set_app_config(
            &param.app_id,
            &NotifySmsItem::method(),
            &param.url,
            &req_auth.user_data().user_id,
            Some(&req_dao.req_env),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::default())
}

pub async fn smser_tpl_config_list<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: TplConfigListParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    tpl_config_list(
        param,
        &req_dao.web_dao.sender_smser.smser.tpl_config,
        req_dao,
    )
    .await
}

pub async fn smser_tpl_config_del<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: TplConfigDelParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    tpl_config_del(
        param,
        &req_dao.web_dao.sender_smser.smser.tpl_config,
        req_dao,
    )
    .await
}

#[derive(Debug, Deserialize)]
pub struct SmserMessageSendParam {
    pub tpl_id: u64,
    pub area: Option<String>,
    pub mobile: Vec<String>,
    pub max_try: Option<u8>,
    //body 对外统一格式{key:val}
    // 这里判断不同发送端进行统一转换匹配
    pub data: HashMap<String, String>,
    pub send_time: Option<String>,
}
//后台界面发送短信接口
pub async fn smser_message_send<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: SmserMessageSendParam,
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
        .sender_smser
        .smser
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
        return Ok(req_dao.fluent_json_data(fluent_message!("sms-use-other-user-app")));
        //JsonData::message("can't use other user app")
    }
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
    let mobile = param.mobile.iter().map(|e| e.as_str()).collect::<Vec<_>>();
    req_dao
        .web_dao
        .sender_smser
        .app_send(
            &app,
            &tpl.tpl_id,
            param.area.as_deref().unwrap_or("86"),
            &mobile,
            &param.data,
            &send_time,
            &param.max_try,
            Some(&req_dao.req_env),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::default())
}
