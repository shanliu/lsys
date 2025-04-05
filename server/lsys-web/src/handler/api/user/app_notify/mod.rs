use lsys_app_notify::model::NotifyDataStatus;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::common::UserAuthQueryDao;
use crate::common::{JsonData, JsonResult, LimitParam};
use crate::dao::access::api::user::CheckUserNotifyView;
use lsys_access::dao::AccessSession;
#[derive(Deserialize)]
pub struct DataListParam {
    pub app_id: Option<u64>,
    pub method: Option<String>,
    pub status: Option<i8>,
    pub limit: Option<LimitParam>,
    pub count_num: Option<bool>,
}

#[derive(Serialize)]
pub struct DataListRecord {
    pub id: u64,
    pub app_id: u64,
    pub method: String,
    pub call_url: String,
    pub status: i8,
    pub result: String,
    pub try_num: i8,
    pub publish_time: u64,
    pub next_time: u64,
}

pub async fn data_list(param: &DataListParam, req_dao: &UserAuthQueryDao) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &req_dao.req_env,
            Some(&auth_data),
            &CheckUserNotifyView {
                res_user_id: auth_data.user_id(),
            },
        )
        .await?;

    let status = if let Some(e) = param.status {
        Some(match NotifyDataStatus::try_from(e) {
            Ok(ts) => ts,
            Err(err) => return Err(err.into()),
        })
    } else {
        None
    };

    let res = req_dao
        .web_dao
        .app_notify
        .notify_dao
        .record
        .data_list(
            param.app_id,
            Some(auth_data.user_id()),
            param.method.as_deref(),
            status,
            param.limit.as_ref().map(|e| e.into()).as_ref(),
        )
        .await?;
    let next = res.1;
    let out = res
        .0
        .into_iter()
        .map(|e| DataListRecord {
            id: e.0.id,
            status: e.0.status,
            app_id: e.0.app_id,
            method: e.0.method,
            call_url: e.1,
            result: e.0.result,
            try_num: e.0.try_num,
            publish_time: e.0.publish_time,
            next_time: e.0.next_time,
        })
        .collect::<Vec<_>>();

    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .app_notify
                .notify_dao
                .record
                .data_count(
                    param.app_id,
                    Some(auth_data.user_id()),
                    param.method.as_deref(),
                    status,
                )
                .await?,
        )
    } else {
        None
    };
    Ok(JsonData::data(
        json!({ "data":out,"next":next, "total":count,}),
    ))
}
