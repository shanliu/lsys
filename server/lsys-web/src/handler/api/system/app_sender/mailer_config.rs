use super::{tpl_config_del, tpl_config_list, TplConfigDelParam, TplConfigListParam};
use crate::{
    common::{JsonData, JsonResult, UserAuthQueryDao},
    dao::access::api::system::CheckAdminMailMgr,
};
use lsys_access::dao::AccessSession;
use lsys_app_sender::{
    dao::SenderError,
    model::{SenderConfigStatus, SenderMailConfigType},
};
use serde::Deserialize;
use serde_json::{json, Value};
#[derive(Debug, Deserialize)]
pub struct MailerConfigAddParam {
    pub priority: i8,
    pub config_type: i8,
    pub config_data: Value,
}

pub async fn mailer_config_add(
    param: &MailerConfigAddParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.access_env().await?, &CheckAdminMailMgr {}, None)
        .await?;
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    let config_type = SenderMailConfigType::try_from(param.config_type)?;
    let id = req_dao
        .web_dao
        .app_sender
        .mailer
        .mailer_dao
        .mail_record
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
pub struct MailerConfigDeleteParam {
    pub config_id: u64,
}
pub async fn mailer_config_del(
    param: &MailerConfigDeleteParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.access_env().await?, &CheckAdminMailMgr {}, None)
        .await?;
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    let res = req_dao
        .web_dao
        .app_sender
        .mailer
        .mailer_dao
        .mail_record
        .find_config_by_id(&param.config_id)
        .await;
    match res {
        Ok(config) => {
            if SenderConfigStatus::Enable.eq(config.status) {
                req_dao
                    .web_dao
                    .app_sender
                    .mailer
                    .mailer_dao
                    .mail_record
                    .config_del(&config, req_auth.user_id(), Some(&req_dao.req_env))
                    .await?;
            }
        }
        Err(err) => match &err {
            SenderError::Sqlx(sqlx::Error::RowNotFound) => return Ok(JsonData::default()),
            _ => {
                return Err(err.into());
            }
        },
    }
    Ok(JsonData::default())
}

#[derive(Debug, Deserialize)]
pub struct MailerConfigListParam {
    pub id: Option<u64>,
}

pub async fn mailer_config_list(
    param: &MailerConfigListParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.access_env().await?, &CheckAdminMailMgr {}, None)
        .await?;

    let data = req_dao
        .web_dao
        .app_sender
        .mailer
        .mailer_dao
        .mail_record
        .config_list(Some(0), param.id, Some(0))
        .await?;
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

pub async fn mailer_tpl_config_list(
    param: &TplConfigListParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.access_env().await?, &CheckAdminMailMgr {}, None)
        .await?;
    tpl_config_list(
        param,
        &req_dao.web_dao.app_sender.mailer.mailer_dao.tpl_config,
        req_dao,
    )
    .await
}

pub async fn mailer_tpl_config_del(
    param: &TplConfigDelParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.access_env().await?, &CheckAdminMailMgr {}, None)
        .await?;
    tpl_config_del(
        param,
        &req_dao.web_dao.app_sender.mailer.mailer_dao.tpl_config,
        req_dao,
    )
    .await
}
