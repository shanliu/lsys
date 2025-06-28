use crate::common::{JsonData, PageParam};
use crate::dao::access::RbacAccessCheckEnv;
use crate::{
    common::{JsonResponse, JsonResult, UserAuthQueryDao},
    dao::access::api::system::admin::CheckAdminMailMgr,
};
use lsys_access::dao::AccessSession;
use lsys_app_sender::model::{SenderMailConfigType, SenderTplConfigStatus};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct MailerConfigAddParam {
    #[serde(deserialize_with = "crate::common::deserialize_i8")]
    pub priority: i8,
    #[serde(deserialize_with = "crate::common::deserialize_i8")]
    pub config_type: i8,
    pub config_data: Value,
}

pub async fn mailer_config_add(
    param: &MailerConfigAddParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminMailMgr {},
        )
        .await?;
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
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({ "id": id }))))
}

#[derive(Debug, Deserialize)]
pub struct MailerConfigDeleteParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub config_id: u64,
}
pub async fn mailer_config_del(
    param: &MailerConfigDeleteParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminMailMgr {},
        )
        .await?;
    let config = req_dao
        .web_dao
        .app_sender
        .mailer
        .mailer_dao
        .mail_record
        .find_config_by_id(param.config_id)
        .await?;
    req_dao
        .web_dao
        .app_sender
        .mailer
        .mailer_dao
        .mail_record
        .config_del(&config, auth_data.user_id(), Some(&req_dao.req_env))
        .await?;
    Ok(JsonResponse::default())
}

#[derive(Debug, Deserialize)]
pub struct MailerConfigListParam {
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u64")]
    pub id: Option<u64>,
}

pub async fn mailer_config_list(
    param: &MailerConfigListParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminMailMgr {},
        )
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

    Ok(JsonResponse::data(JsonData::body(json!({ "data": data }))))
}

#[derive(Debug, Deserialize)]
pub struct MailerTplConfigListParam {
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u64")]
    pub id: Option<u64>,
    pub tpl: Option<String>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub app_info: Option<bool>,
    pub page: Option<PageParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
}

pub async fn mailer_tpl_config_list(
    param: &MailerTplConfigListParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminMailMgr {},
        )
        .await?;

    let tpl_data = req_dao
        .web_dao
        .app_sender
        .mailer
        .mailer_dao
        .tpl_config
        .list_config(
            param.id,
            Some(0),
            Some(0),
            param.tpl.as_deref(),
            param.page.as_ref().map(|e| e.into()).as_ref(),
        )
        .await?;
    let app_data = if param.app_info.unwrap_or(false) && !tpl_data.is_empty() {
        req_dao
            .web_dao
            .web_app
            .app_dao
            .app
            .find_by_ids(&tpl_data.iter().map(|e| e.0.app_id).collect::<Vec<_>>())
            .await?
    } else {
        HashMap::new()
    };

    let row = tpl_data
        .into_iter()
        .map(|(a, b)| {
            let (setting_id, setting_key, setting_name) = match b {
                Some(s) => (s.id, s.setting_key, s.name),
                None => (0, "".to_string(), "".to_string()),
            };
            let (app_name, client_id) = match app_data.get(&a.app_id) {
                Some(app) => (app.name.as_str(), app.client_id.as_str()),
                None => ("", ""),
            };
            let mut json = json!({
                "id":a.id,
                "app_id":a.app_id,
                "config_data":serde_json::from_str::<Value>(&a.config_data).ok(),
                "name":a.name,
                "tpl_key":a.tpl_key,
                "user_id":a.user_id,
                "change_user_id":a.change_user_id,
                "change_time":a.change_time,
                "setting_key":setting_key,
                "setting_id":setting_id,
                "setting_name":setting_name,
            });
            if param.app_info.unwrap_or(false) {
                json["app_name"] = json!(app_name);
                json["app_client_id"] = json!(client_id);
            }
            json
        })
        .collect::<Vec<_>>();
    let total = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .app_sender
                .mailer
                .mailer_dao
                .tpl_config
                .count_config(param.id, Some(0), Some(0), param.tpl.as_deref())
                .await?,
        )
    } else {
        None
    };
    Ok(JsonResponse::data(JsonData::body(
        json!({ "data": row ,"total":total}),
    )))
}

#[derive(Debug, Deserialize)]
pub struct MailerTplConfigDelParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub tpl_config_id: u64,
}

pub async fn mailer_tpl_config_del(
    param: &MailerTplConfigDelParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminMailMgr {},
        )
        .await?;
    let config = req_dao
        .web_dao
        .app_sender
        .mailer
        .mailer_dao
        .tpl_config
        .find_by_id(param.tpl_config_id)
        .await?;
    if SenderTplConfigStatus::Delete.eq(config.status) {
        return Ok(JsonResponse::data(JsonData::body(json!({ "num": 0 }))));
    }
    let row = req_dao
        .web_dao
        .app_sender
        .mailer
        .mailer_dao
        .tpl_config
        .del_config(&config, auth_data.user_id(), Some(&req_dao.req_env))
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({ "num": row }))))
}
