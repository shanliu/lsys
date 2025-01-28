use std::collections::HashMap;

use lsys_access::dao::AccessSession;
use lsys_app_sender::{dao::SenderTplConfig, model::SenderTplConfigStatus};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::common::{JsonData, JsonResult, PageParam, UserAuthQueryDao};

#[derive(Debug, Deserialize)]
pub struct TplConfigDelParam {
    pub app_config_id: u64,
}

pub(crate) async fn tpl_config_del(
    param: &TplConfigDelParam,
    tpl_config: &SenderTplConfig,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    let config = tpl_config.find_by_id(&param.app_config_id).await?;
    if SenderTplConfigStatus::Delete.eq(config.status) {
        return Ok(JsonData::data(json!({ "num": 0 })));
    }
    let row = tpl_config
        .del_config(&config, req_auth.user_id(), Some(&req_dao.req_env))
        .await?;
    Ok(JsonData::data(json!({ "num": row })))
}

#[derive(Debug, Deserialize)]
pub struct TplConfigListParam {
    pub id: Option<u64>,
    pub tpl: Option<String>,
    pub app_info: Option<bool>,
    pub page: Option<PageParam>,
    pub count_num: Option<bool>,
}
//模板列表
pub(crate) async fn tpl_config_list(
    param: &TplConfigListParam,
    tpl_config: &SenderTplConfig,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let tpl_data = tpl_config
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
            tpl_config
                .count_config(param.id, Some(0), Some(0), param.tpl.as_deref())
                .await?,
        )
    } else {
        None
    };
    Ok(JsonData::data(json!({ "data": row ,"total":total})))
}
