use crate::{
    dao::RequestDao,
    handler::access::{AccessAppSenderSmsConfig, AccessAppSenderSmsMsg},
    PageParam, {JsonData, JsonResult},
};
use lsys_sender::{
    dao::SenderError,
    model::{SenderConfigStatus, SenderSmsConfigType, SenderSmsMessageStatus},
};
use lsys_user::dao::auth::{SessionData, SessionTokenData, UserSession};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Debug, Deserialize)]
pub struct SmserMessageLogParam {
    pub message_id: String,
    pub count_num: Option<bool>,
    pub page: Option<PageParam>,
}

pub async fn smser_message_log<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: SmserMessageLogParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let message_id = param.message_id.parse::<u64>().map_err(JsonData::message)?;
    let data = req_dao
        .web_dao
        .sender_smser
        .sms_record()
        .find_message_by_id(&message_id)
        .await?;
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessAppSenderSmsMsg {
                user_id: req_auth.user_data().user_id,
                res_user_id: data.user_id,
                app_id: Some(data.app_id),
            },
            None,
        )
        .await?;

    let res = req_dao
        .web_dao
        .sender_smser
        .sms_record()
        .message_log_list(&message_id, &param.page.map(|e| e.into()))
        .await?;
    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .sender_smser
                .sms_record()
                .message_log_count(&message_id)
                .await?,
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
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let message_id = param.message_id.parse::<u64>().map_err(JsonData::message)?;
    let data = req_dao
        .web_dao
        .sender_smser
        .sms_record()
        .find_message_by_id(&message_id)
        .await?;
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessAppSenderSmsMsg {
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
pub struct SmserMessageListParam {
    pub user_id: Option<u64>,
    pub app_id: Option<u64>,
    pub tpl_id: Option<String>,
    pub status: Option<i8>,
    pub mobile: Option<String>,
    pub count_num: Option<bool>,
    pub page: Option<PageParam>,
}

pub async fn smser_message_list<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: SmserMessageListParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
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
        .await?;
    let status = if let Some(e) = param.status {
        Some(SenderSmsMessageStatus::try_from(e)?)
    } else {
        None
    };
    let res = req_dao
        .web_dao
        .sender_smser
        .sms_record()
        .message_list(
            &param.user_id,
            &param.app_id,
            &param.tpl_id,
            &status,
            &param.mobile,
            &param.page.map(|e| e.into()),
        )
        .await?;
    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .sender_smser
                .sms_record()
                .message_count(
                    &param.user_id,
                    &param.app_id,
                    &param.tpl_id,
                    &status,
                    &param.mobile,
                )
                .await?,
        )
    } else {
        None
    };
    let res = res
        .into_iter()
        .map(|e| {
            json!({
                "id":e.id,
                "app_id":e.app_id,
                "mobile":format!("{}-{}",e.area,e.mobile),
                "tpl_id":e.tpl_id,
                "try_num":e.try_num,
                "add_time":e.add_time,
                "status":e.status,
                "expected_time":e.expected_time,
                "send_time":e.send_time
            })
        })
        .collect::<Vec<_>>();
    Ok(JsonData::data(json!({ "data": res,"total":count})))
}
#[derive(Debug, Deserialize)]
pub struct SmserMessageCancelParam {
    pub message_id: String,
}

pub async fn smser_message_cancel<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: SmserMessageCancelParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let message_id = param.message_id.parse::<u64>().map_err(JsonData::message)?;
    let data = req_dao
        .web_dao
        .sender_smser
        .sms_record()
        .find_message_by_id(&message_id)
        .await?;
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessAppSenderSmsMsg {
                user_id: req_auth.user_data().user_id,
                res_user_id: data.user_id,
                app_id: Some(data.app_id),
            },
            None,
        )
        .await?;

    req_dao
        .web_dao
        .sender_smser
        .send_cancel(&data, req_auth.user_data().user_id)
        .await?;
    Ok(JsonData::message("ok"))
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
            &AccessAppSenderSmsConfig {
                user_id: req_auth.user_data().user_id,
                res_user_id: uid,
                app_id: param.app_id,
            },
            None,
        )
        .await?;

    let config_type = SenderSmsConfigType::try_from(param.config_type)?;
    let id = req_dao
        .web_dao
        .sender_smser
        .sms_record()
        .config_add(
            Some(param.app_id),
            param.priority,
            config_type,
            param.config_data,
            uid,
            req_auth.user_data().user_id,
        )
        .await?;
    Ok(JsonData::data(json!({ "id": id })))
}

#[derive(Debug, Deserialize)]
pub struct SmserConfigDeleteParam {
    pub config_id: u64,
}
pub async fn smser_config_del<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: SmserConfigDeleteParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    let sms_record = req_dao.web_dao.sender_smser.sms_record();
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
                    .await?;

                sms_record
                    .config_del(&config, req_auth.user_data().user_id)
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
    Ok(JsonData::message("del ok"))
}

#[derive(Debug, Deserialize)]
pub struct SmserConfigListParam {
    pub user_id: Option<u64>,
    pub id: Option<u64>,
    pub app_id: Option<u64>,
}

pub async fn smser_config_list<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: SmserConfigListParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;

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
        .await?;

    let data = req_dao
        .web_dao
        .sender_smser
        .sms_record()
        .config_list(param.user_id, param.id, param.app_id)
        .await?;
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
               "add_time": e.add_time,
               "priority": e.priority,
               "config_data": config_data,
            })
        })
        .collect::<Vec<Value>>();

    Ok(JsonData::data(json!({ "data": data })))
}
