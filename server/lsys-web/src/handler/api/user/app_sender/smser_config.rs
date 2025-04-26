use crate::common::{JsonData, PageParam};
use crate::common::{JsonResponse, JsonResult, UserAuthQueryDao};
use crate::dao::access::api::user::CheckUserAppSenderSmsConfig;
use lsys_access::dao::AccessSession;
use lsys_app::dao::UserAppDataParam;
use lsys_app::model::AppStatus;
use lsys_app_sender::dao::SMS_NOTIFY_TYPE;
use lsys_app_sender::model::SenderSmsConfigType;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;

pub(super) async fn smser_inner_access_check(
    app_id: u64,
    res_user_id: u64,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<()> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .cache()
        .find_by_id(&app_id)
        .await?;
    req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .cache()
        .exter_feature_check(&app, &[crate::handler::APP_FEATURE_SMS])
        .await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &req_dao.req_env,
            Some(&auth_data),
            &CheckUserAppSenderSmsConfig { res_user_id },
        )
        .await?;
    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct SmserConfigAddParam {
    pub app_id: u64,
    pub priority: i8,
    pub config_type: i8,
    pub config_data: Value,
}

pub async fn smser_config_add(
    param: &SmserConfigAddParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    smser_inner_access_check(param.app_id, auth_data.user_id(), req_dao).await?;

    let config_type = SenderSmsConfigType::try_from(param.config_type)?;
    let id = req_dao
        .web_dao
        .app_sender
        .smser
        .smser_dao
        .sms_record
        .config_add(
            Some(param.app_id),
            param.priority,
            config_type,
            &param.config_data,
            auth_data.user_id(),
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
    let config = req_dao
        .web_dao
        .app_sender
        .smser
        .smser_dao
        .sms_record
        .find_config_by_id(&param.config_id)
        .await?;
    smser_inner_access_check(config.app_id, config.user_id, req_dao).await?;
    req_dao
        .web_dao
        .app_sender
        .smser
        .smser_dao
        .sms_record
        .config_del(&config, auth_data.user_id(), Some(&req_dao.req_env))
        .await?;
    Ok(JsonResponse::default())
}

#[derive(Debug, Deserialize)]
pub struct SmserConfigListParam {
    pub id: Option<u64>,
    pub app_id: Option<u64>,
}

pub async fn smser_config_list(
    param: &SmserConfigListParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &req_dao.req_env,
            Some(&auth_data),
            &CheckUserAppSenderSmsConfig {
                res_user_id: auth_data.user_id(),
            },
        )
        .await?;
    let data = req_dao
        .web_dao
        .app_sender
        .smser
        .smser_dao
        .sms_record
        .config_list(Some(auth_data.user_id()), param.id, param.app_id)
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

pub async fn smser_notify_get_config(req_dao: &UserAuthQueryDao) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &req_dao.req_env,
            Some(&auth_data),
            &CheckUserAppSenderSmsConfig {
                res_user_id: auth_data.user_id(),
            },
        )
        .await?;
    let app_param = UserAppDataParam {
        status: Some(AppStatus::Enable),
        parent_app_id: Some(auth_data.session().user_app_id),
        client_id: None,
        app_id: None,
    };
    let apps = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .user_app_data(auth_data.user_id(), &app_param, None, None)
        .await?;
    let notify = req_dao
        .web_dao
        .web_app
        .app_dao
        .app_notify
        .record
        .find_config_by_apps(
            &apps.iter().map(|e| e.0.id).collect::<Vec<_>>(),
            SMS_NOTIFY_TYPE,
        )
        .await?;
    let data = apps
        .into_iter()
        .map(|e| {
            let n = notify.iter().find(|t| t.app_id == e.0.id);
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
                "app_id":e.0.id,
                "app_name":e.0.name,
                 "call_url":url,
                 "change_time":change_time,
                 "change_user_id":change_user_id,
            })
        })
        .collect::<Vec<_>>();
    Ok(JsonResponse::data(JsonData::body(json!({
        "data":data,
    }))))
}

#[derive(Debug, Deserialize)]
pub struct SmserNotifyConfigParam {
    pub app_id: u64,
    pub url: String,
}

pub async fn smser_notify_set_config(
    param: &SmserNotifyConfigParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    smser_inner_access_check(param.app_id, auth_data.user_id(), req_dao).await?;
    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .find_by_id(&param.app_id)
        .await?;

    req_dao
        .web_dao
        .web_app
        .app_dao
        .app_notify
        .record
        .set_app_config(
            &app,
            SMS_NOTIFY_TYPE,
            &param.url,
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::default())
}

#[derive(Debug, Deserialize)]
pub struct SmserTplConfigListParam {
    pub id: Option<u64>,
    pub app_id: Option<u64>,
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
        .check(
            &req_dao.req_env,
            Some(&auth_data),
            &CheckUserAppSenderSmsConfig {
                res_user_id: auth_data.user_id(),
            },
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
            Some(auth_data.user_id()),
            param.app_id,
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
                .mailer
                .mailer_dao
                .tpl_config
                .count_config(
                    param.id,
                    Some(auth_data.user_id()),
                    param.app_id,
                    param.tpl.as_deref(),
                )
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
pub struct SmserTplConfigDeleteParam {
    pub config_id: u64,
}
pub async fn smser_tpl_config_del(
    param: &SmserTplConfigDeleteParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let config = req_dao
        .web_dao
        .app_sender
        .mailer
        .mailer_dao
        .mail_record
        .find_config_by_id(&param.config_id)
        .await?;
    smser_inner_access_check(config.app_id, config.user_id, req_dao).await?;
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
