use crate::{
    dao::RequestDao,
    PageParam, {JsonData, JsonResult},
};
use lsys_sender::model::{SenderSmsConfigStatus, SenderSmsConfigType, SenderSmsMessageStatus};
use lsys_user::dao::auth::{SessionData, SessionTokenData, UserSession};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Debug, Deserialize)]
pub struct SmserMessageHistoryParam {
    pub message_id: String,
    pub count_num: Option<bool>,
    pub page: Option<PageParam>,
}

pub async fn smser_message_history<
    't,
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    param: SmserMessageHistoryParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let message_id = param.message_id.parse::<u64>().map_err(JsonData::message)?;
    let data = req_dao
        .web_dao
        .smser
        .sms_record()
        .find_message_by_id(&message_id)
        .await?;
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .access
        .check(
            req_auth.user_data().user_id,
            &[],
            &res_data!(AppSender(data.app_id, req_auth.user_data().user_id)),
        )
        .await?;
    let res = req_dao
        .web_dao
        .smser
        .sms_record()
        .message_history_list(&message_id, &param.page.map(|e| e.into()))
        .await?;
    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .smser
                .sms_record()
                .message_history_count(&message_id)
                .await?,
        )
    } else {
        None
    };
    Ok(JsonData::message("ok").set_data(json!({ "data": res,"count":count})))
}
#[derive(Debug, Deserialize)]
pub struct SmserMessageListParam {
    pub user_id: u64,
    pub app_id: Option<u64>,
    pub tpl_id: Option<String>,
    pub status: Option<i8>,
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
        .access
        .check(
            req_auth.user_data().user_id,
            &[],
            &res_data!(AppSender(param.app_id.unwrap_or_default(), param.user_id)),
        )
        .await?;
    let status = if let Some(e) = param.status {
        Some(SenderSmsMessageStatus::try_from(e)?)
    } else {
        None
    };
    let res = req_dao
        .web_dao
        .smser
        .sms_record()
        .message_list(
            &Some(param.user_id),
            &param.app_id,
            &param.tpl_id,
            &status,
            &param.page.map(|e| e.into()),
        )
        .await?;
    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .smser
                .sms_record()
                .message_count(&Some(param.user_id), &param.app_id, &param.tpl_id, &status)
                .await?,
        )
    } else {
        None
    };
    Ok(JsonData::message("ok").set_data(json!({ "data": res,"count":count})))
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
        .smser
        .sms_record()
        .find_message_by_id(&message_id)
        .await?;
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .access
        .check(
            req_auth.user_data().user_id,
            &[],
            &res_data!(AppSender(data.app_id, req_auth.user_data().user_id)),
        )
        .await?;
    req_dao
        .web_dao
        .smser
        .send_cancel(&data, req_auth.user_data().user_id)
        .await?;
    Ok(JsonData::message("ok"))
}

#[derive(Debug, Deserialize)]
pub struct SmserConfigAddParam {
    pub app_id: u64,
    pub priority: i8,
    pub config_type: i8,
    pub config_data: Value,
}

pub async fn smser_config_add<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: SmserConfigAddParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .access
        .check(
            req_auth.user_data().user_id,
            &[],
            &res_data!(AppSender(param.app_id, req_auth.user_data().user_id)),
        )
        .await?;
    let config_type = SenderSmsConfigType::try_from(param.config_type)?;
    let id = req_dao
        .web_dao
        .smser
        .sms_record()
        .config_add(
            Some(param.app_id),
            param.priority,
            config_type,
            param.config_data,
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
    let sms_record = req_dao.web_dao.smser.sms_record();
    let res = sms_record.find_config_by_id(&param.config_id).await;

    match res {
        Ok(config) => {
            if SenderSmsConfigStatus::Enable.eq(config.status) {
                req_dao
                    .web_dao
                    .user
                    .rbac_dao
                    .rbac
                    .access
                    .check(
                        req_auth.user_data().user_id,
                        &[],
                        &res_data!(AppSender(config.app_id, req_auth.user_data().user_id)),
                    )
                    .await?;
                sms_record
                    .config_del(&config, req_auth.user_data().user_id)
                    .await?;
            }
        }
        Err(err) => match err {
            sqlx::Error::RowNotFound => {
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
    pub user_id: u64,
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
        .access
        .check(
            req_auth.user_data().user_id,
            &[],
            &res_data!(AppSender(param.app_id.unwrap_or_default(), param.user_id)),
        )
        .await?;
    let data = req_dao
        .web_dao
        .smser
        .sms_record()
        .config_list(Some(param.user_id), param.app_id)
        .await?;
    let data = data
        .into_iter()
        .map(|(e, v)| {
            let config_data = match v {
                lsys_sender::model::SenderSmsConfigData::Limit(t) => json!(&t).to_string(),
                lsys_sender::model::SenderSmsConfigData::MaxOfSend(u) => u.to_string(),
                lsys_sender::model::SenderSmsConfigData::Block(area, mobile) => {
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
