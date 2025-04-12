use std::collections::HashMap;

use crate::common::{JsonData, JsonResponse, JsonResult, UserAuthQueryDao};
use crate::dao::access::api::system::CheckAdminSmsMgr;
use lsys_access::dao::AccessSession;

use crate::common::PageParam;
use lsys_app_sender::model::SenderTplConfigStatus;
use lsys_app_sender::{
    dao::SenderError,
    model::{SenderConfigStatus, SenderSmsConfigType},
};
use serde::Deserialize;
use serde_json::{json, Value};
#[derive(Debug, Deserialize)]
pub struct SmserConfigAddParam {
    pub priority: i8,
    pub config_type: i8,
    pub config_data: Value,
}

pub async fn smser_config_add(
    param: &SmserConfigAddParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminSmsMgr {})
        .await?;

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
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({ "id": id }))))
}

#[derive(Debug, Deserialize)]
pub struct SmserConfigDeleteParam {
    pub config_id: u64,
}
pub async fn smser_config_del(
    param: &SmserConfigDeleteParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminSmsMgr {})
        .await?;
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
                    .config_del(&config, auth_data.user_id(), Some(&req_dao.req_env))
                    .await?;
            }
        }
        Err(err) => match &err {
            SenderError::Sqlx(sqlx::Error::RowNotFound) => {
                return Ok(JsonResponse::message("email not find"));
            }
            _ => {
                return Err(err.into());
            }
        },
    }
    Ok(JsonResponse::default())
}

#[derive(Debug, Deserialize)]
pub struct SmserConfigListParam {
    pub id: Option<u64>,
}

pub async fn smser_config_list(
    param: &SmserConfigListParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminSmsMgr {})
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

    Ok(JsonResponse::data(JsonData::body(json!({ "data": data }))))
}

#[derive(Debug, Deserialize)]
pub struct SmserTplConfigListParam {
    pub id: Option<u64>,
    pub tpl: Option<String>,
    pub app_info: Option<bool>,
    pub page: Option<PageParam>,
    pub count_num: Option<bool>,
}

pub async fn smser_tpl_config_list(
    param: &SmserTplConfigListParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminSmsMgr {})
        .await?;
    let tpl_data = req_dao
        .web_dao
        .app_sender
        .smser
        .smser_dao
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
                "tpl_id":a.tpl_id,
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
                .smser
                .smser_dao
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
pub struct SmserTplConfigDelParam {
    pub app_config_id: u64,
}

pub async fn smser_tpl_config_del(
    param: &SmserTplConfigDelParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminSmsMgr {})
        .await?;
    let config = req_dao
        .web_dao
        .app_sender
        .smser
        .smser_dao
        .tpl_config
        .find_by_id(&param.app_config_id)
        .await?;
    if SenderTplConfigStatus::Delete.eq(config.status) {
        return Ok(JsonResponse::data(JsonData::body(json!({ "num": 0 }))));
    }
    let row = req_dao
        .web_dao
        .app_sender
        .smser
        .smser_dao
        .tpl_config
        .del_config(&config, auth_data.user_id(), Some(&req_dao.req_env))
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({ "num": row }))))
}
