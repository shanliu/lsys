use crate::common::{JsonData, JsonResult, UserAuthQueryDao};
use crate::dao::access::api::system::CheckAdminSmsMgr;
use lsys_access::dao::AccessSession;

use lsys_app_sender::{
    dao::SenderError,
    model::{SenderConfigStatus, SenderSmsConfigType},
};
use serde::Deserialize;
use serde_json::{json, Value};

use super::{tpl_config_del, tpl_config_list, TplConfigDelParam, TplConfigListParam};

#[derive(Debug, Deserialize)]
pub struct SmserConfigAddParam {
    pub priority: i8,
    pub config_type: i8,
    pub config_data: Value,
}

pub async fn smser_config_add(
    param: &SmserConfigAddParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.access_env().await?, &CheckAdminSmsMgr {}, None)
        .await?;
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;

    let config_type = SenderSmsConfigType::try_from(param.config_type)?;
    let id = req_dao
        .web_dao
        .app_sender
        .smser
        .smser_dao
        .sms_record
        .config_add(
            Some(0),
            param.priority,
            config_type,
            &param.config_data,
            0,
            req_auth.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::data(json!({ "id": id })))
}

#[derive(Debug, Deserialize)]
pub struct SmserConfigDeleteParam {
    pub config_id: u64,
}
pub async fn smser_config_del(
    param: &SmserConfigDeleteParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.access_env().await?, &CheckAdminSmsMgr {}, None)
        .await?;
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    let res = req_dao
        .web_dao
        .app_sender
        .smser
        .smser_dao
        .sms_record
        .find_config_by_id(&param.config_id)
        .await;

    match res {
        Ok(config) => {
            if SenderConfigStatus::Enable.eq(config.status) {
                req_dao
                    .web_dao
                    .app_sender
                    .smser
                    .smser_dao
                    .sms_record
                    .config_del(&config, req_auth.user_id(), Some(&req_dao.req_env))
                    .await?;
            }
        }
        Err(err) => match &err {
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
pub struct SmserConfigListParam {
    pub id: Option<u64>,
}

pub async fn smser_config_list(
    param: &SmserConfigListParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.access_env().await?, &CheckAdminSmsMgr {}, None)
        .await?;

    let data = req_dao
        .web_dao
        .app_sender
        .smser
        .smser_dao
        .sms_record
        .config_list(Some(0), param.id, Some(0))
        .await?;
    let data = data
        .into_iter()
        .map(|(e, v)| {
            let config_data = match v {
                lsys_app_sender::model::SenderSmsConfigData::Limit(t) => json!(&t).to_string(),
                lsys_app_sender::model::SenderSmsConfigData::MaxOfSend(u) => u.to_string(),
                lsys_app_sender::model::SenderSmsConfigData::Block { area, mobile } => {
                    format!("{}{}", area, mobile)
                }
                lsys_app_sender::model::SenderSmsConfigData::PassTpl(val) => val,
                lsys_app_sender::model::SenderSmsConfigData::Close => "".to_string(),
                lsys_app_sender::model::SenderSmsConfigData::None => "".to_string(),
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

pub async fn smser_tpl_config_list(
    param: &TplConfigListParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.access_env().await?, &CheckAdminSmsMgr {}, None)
        .await?;
    tpl_config_list(
        param,
        &req_dao.web_dao.app_sender.smser.smser_dao.tpl_config,
        req_dao,
    )
    .await
}

pub async fn smser_tpl_config_del(
    param: &TplConfigDelParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.access_env().await?, &CheckAdminSmsMgr {}, None)
        .await?;
    tpl_config_del(
        param,
        &req_dao.web_dao.app_sender.smser.smser_dao.tpl_config,
        req_dao,
    )
    .await
}
