use std::collections::HashMap;

use lsys_app_sender::{dao::SenderTplConfig, model::SenderTplConfigStatus};
use lsys_user::dao::auth::{SessionData, SessionTokenData, UserSession};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{
    dao::RequestAuthDao, handler::access::AccessAppSenderSmsConfig, JsonData, JsonResult, PageParam,
};

#[derive(Debug, Deserialize)]
pub struct TplConfigDelParam {
    pub app_config_id: u64,
}

pub(crate) async fn tpl_config_del<
    't,
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    param: TplConfigDelParam,
    tpl_config: &SenderTplConfig,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let config = tpl_config
        .find_by_id(&param.app_config_id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    if SenderTplConfigStatus::Delete.eq(config.status) {
        return Ok(JsonData::data(json!({ "num": 0 })));
    }
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
    let row = tpl_config
        .del_config(
            &config,
            &req_auth.user_data().user_id,
            Some(&req_dao.req_env),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::data(json!({ "num": row })))
}

#[derive(Debug, Deserialize)]
pub struct TplConfigListParam {
    pub user_id: Option<u64>,
    pub id: Option<u64>,
    pub app_id: Option<u64>,
    pub tpl: Option<String>,
    pub app_info: Option<bool>,
    pub page: Option<PageParam>,
    pub count_num: Option<bool>,
}
//模板列表
pub(crate) async fn tpl_config_list<
    't,
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    param: TplConfigListParam,
    tpl_config: &SenderTplConfig,
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
    let tpl_data = tpl_config
        .list_config(
            &param.id,
            &Some(param.user_id.unwrap_or(req_auth.user_data().user_id)),
            &param.app_id,
            &param.tpl,
            &Some(param.page.unwrap_or_default().into()),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let app_data = if param.app_info.unwrap_or(false) && !tpl_data.is_empty() {
        req_dao
            .web_dao
            .app
            .app_dao
            .app
            .find_by_ids(&tpl_data.iter().map(|e| e.0.app_id).collect::<Vec<_>>())
            .await
            .map_err(|e| req_dao.fluent_json_data(e))?
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
                .count_config(
                    &param.id,
                    &Some(param.user_id.unwrap_or(req_auth.user_data().user_id)),
                    &param.app_id,
                    &param.tpl,
                )
                .await
                .map_err(|e| req_dao.fluent_json_data(e))?,
        )
    } else {
        None
    };
    Ok(JsonData::data(json!({ "data": row ,"total":total})))
}
