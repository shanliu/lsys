use std::collections::HashMap;

use crate::common::JsonData;
use crate::{
    common::{JsonError, JsonResponse, JsonResult, RequestDao},
    dao::access::rest::CheckRestApp,
};
use lsys_app::model::AppModel;
use lsys_core::{str_time, IntoFluentMessage};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Debug, Deserialize)]
pub struct SendParam {
    pub to: Vec<String>,
    pub tpl_key: String,
    pub data: HashMap<String, String>,
    pub reply: Option<String>,
    pub send_time: Option<String>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u8")]
    pub max_try: Option<u8>,
}
pub async fn send(
    param: &SendParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, None, &CheckRestApp {})
        .await?;

    req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .cache()
        .exter_feature_check(app, &[crate::handler::APP_FEATURE_MAIL])
        .await?;

    let send_time = if let Some(ref t) = param.send_time {
        if t.is_empty() {
            None
        } else {
            Some(str_time(t).map_err(JsonError::Message)?.timestamp() as u64)
        }
    } else {
        None
    };
    // 字符串转时间对象
    let to = param.to.iter().map(|e| e.as_str()).collect::<Vec<_>>();
    let data = req_dao
        .web_dao
        .app_sender
        .mailer
        .app_send(
            app,
            &param.tpl_key,
            &to,
            &param
                .data
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect(),
            send_time,
            param.reply.as_deref(),
            param.max_try,
            Some(&req_dao.req_env),
        )
        .await?;
    let detail = data
        .into_iter()
        .map(|e| {
            json!({
                "snid":e.0.to_string(),
                "mail":e.1,
            })
        })
        .collect::<Vec<Value>>();
    Ok(JsonResponse::data(JsonData::body(json!( {
        "detail":detail
    }))))
}

#[derive(Debug, Deserialize)]
pub struct CancelParam {
    pub snid_data: Vec<String>,
}
pub async fn cancel(
    param: &CancelParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    let mut ids = Vec::with_capacity(param.snid_data.len());
    for e in param.snid_data.iter() {
        ids.push(e.parse::<u64>()?);
    }

    let data = req_dao
        .web_dao
        .app_sender
        .mailer
        .app_send_cancel(app, &ids, Some(&req_dao.req_env))
        .await?;
    let detail = data
        .into_iter()
        .map(|e| {
            json!({
                "snid":e.0.to_string(),
                "status":!e.1&&e.2.is_none(),
              //  "sending":e.1,
                "msg":e.2.map(|e|e.to_fluent_message().default_format())
            })
        })
        .collect::<Vec<Value>>();
    Ok(JsonResponse::data(JsonData::body(json!( {
        "detail":detail
    }))))
}
