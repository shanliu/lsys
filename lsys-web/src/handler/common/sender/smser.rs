use crate::{
    dao::RequestDao,
    {JsonData, JsonResult},
};
use lsys_sender::model::{SenderSmsConfigStatus, SenderSmsConfigType};
use lsys_user::dao::auth::{SessionData, SessionTokenData, UserSession};
use serde::Deserialize;
use serde_json::{json, Value};

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
    pub app_id: u64,
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
            &res_data!(AppSender(param.app_id, req_auth.user_data().user_id)),
        )
        .await?;
    let data = req_dao
        .web_dao
        .smser
        .sms_record()
        .config_list(Some(param.app_id))
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

    let alisender = &req_dao.web_dao.smser.aliyun_sender;
    let ali_config = alisender
        .list_config(None)
        .await?
        .into_iter()
        .map(|e| {
            json!({
               "id": e.id,
               "name": e.name,
               "app_id":format!("{}***",e.access_id.chars().take(4).collect::<String>()),
            })
        })
        .collect::<Vec<Value>>();

    Ok(JsonData::data(
        json!({ "config": data,"ali_config": ali_config}),
    ))
}
