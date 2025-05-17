use crate::common::{JsonData, JsonResponse, JsonResult, LimitParam, PageParam, UserAuthQueryDao};
use crate::dao::access::api::system::CheckAdminSmsMgr;
use lsys_access::dao::AccessSession;
use lsys_app_sender::model::SenderSmsMessageStatus;
use lsys_core::{now_time, IntoFluentMessage};
use serde::Deserialize;
use serde_json::json;
use tracing::warn;

#[derive(Debug, Deserialize)]
pub struct SmserMessageLogParam {
    pub message_id: String,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
    pub page: Option<PageParam>,
}

pub async fn smser_message_log(
    param: &SmserMessageLogParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminSmsMgr {})
        .await?;
    let message_id = param.message_id.parse::<u64>()?;
    let res = req_dao
        .web_dao
        .app_sender
        .smser
        .smser_dao
        .sms_record
        .message_log_list(message_id, param.page.as_ref().map(|e| e.into()).as_ref())
        .await?;
    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .app_sender
                .smser
                .smser_dao
                .sms_record
                .message_log_count(message_id)
                .await?,
        )
    } else {
        None
    };
    Ok(JsonResponse::data(JsonData::body(
        json!({ "data": res,"total":count}),
    )))
}

#[derive(Debug, Deserialize)]
pub struct SmserMessageBodyParam {
    pub message_id: String,
}

pub async fn smser_message_body(
    param: &SmserMessageBodyParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminSmsMgr {})
        .await?;
    let message_id = param.message_id.parse::<u64>()?;
    let msg = req_dao
        .web_dao
        .app_sender
        .smser
        .smser_dao
        .sms_record
        .find_message_by_id(message_id)
        .await?;
    let body = req_dao
        .web_dao
        .app_sender
        .smser
        .smser_dao
        .sms_record
        .find_body_by_id(msg.sender_body_id)
        .await?;
    req_dao
        .web_dao
        .app_sender
        .smser
        .smser_message_body(&msg, &body, &auth_data, Some(&req_dao.req_env))
        .await?;

    Ok(JsonResponse::data(JsonData::body(
        json!({ "body": body.tpl_var}),
    )))
}

#[derive(Debug, Deserialize)]
pub struct SmserMessageListParam {
    pub tpl_id: Option<String>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u64")]
    pub body_id: Option<u64>,
    pub snid: Option<String>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_i8")]
    pub status: Option<i8>,
    pub mobile: Option<String>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
    pub limit: Option<LimitParam>,
}

pub async fn smser_message_list(
    param: &SmserMessageListParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminSmsMgr {})
        .await?;

    let status = if let Some(e) = param.status {
        Some(SenderSmsMessageStatus::try_from(e)?)
    } else {
        None
    };

    let res = req_dao
        .web_dao
        .app_sender
        .smser
        .smser_dao
        .sms_record
        .message_list(
            Some(0),
            Some(0),
            param.tpl_id.as_deref(),
            param.body_id,
            param.snid.as_ref().and_then(|e| e.parse::<u64>().ok()),
            status,
            param.mobile.as_deref(),
            param.limit.as_ref().map(|e| e.into()).as_ref(),
        )
        .await?;
    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .app_sender
                .smser
                .smser_dao
                .sms_record
                .message_count(
                    Some(0),
                    Some(0),
                    param.tpl_id.as_deref(),
                    param.body_id,
                    param.snid.as_ref().and_then(|e| e.parse::<u64>().ok()),
                    status,
                    param.mobile.as_deref(),
                )
                .await?,
        )
    } else {
        None
    };

    if let Err(err) = req_dao
        .web_dao
        .app_sender
        .smser
        .smser_dao
        .add_status_query(
            &res.0
                .iter()
                .flat_map(|(m, b)| if b.is_some() { Some(m) } else { None })
                .collect::<Vec<_>>(),
        )
        .await
    {
        warn!(
            "query status fail:{}",
            err.to_fluent_message().default_format()
        );
    }
    let ntime = now_time().unwrap_or_default();
    let next = res.1;
    let res_data = req_dao
        .web_dao
        .app_sender
        .smser
        .smser_dao
        .task_is_run(res.0.iter().map(|t| (&t.0.id, t)).collect::<Vec<_>>())
        .await?;
    let res = res_data
        .into_iter()
        .map(|(e, s)| {
            let now_send = SenderSmsMessageStatus::Init.eq(e.0.status)
                && e.1
                    .as_ref()
                    .map(|t| t.expected_time <= ntime)
                    .unwrap_or(true);

            json!({
                "id":e.0.id,
                "snid":e.0.snid,
                "app_id":e.1.as_ref().map(|t|t.app_id),
                "mobile":format!("{}-{}",e.0.area,e.0.mobile),
                "tpl_id":e.1.as_ref().map(|t|t.tpl_id.to_owned()),
                "try_num":e.0.try_num,
                "max_try_num":e.1.as_ref().map(|t|t.max_try_num),
                "add_time":e.1.as_ref().map(|t|t.add_time),
                "status":e.0.status,
                "now_send":now_send,
                "on_task":s.is_some(),
                "expected_time":e.1.as_ref().map(|t|t.expected_time),
               "send_time":e.0.send_time
            })
        })
        .collect::<Vec<_>>();
    Ok(JsonResponse::data(JsonData::body(
        json!({ "data": res,"total":count,"next":next}),
    )))
}
#[derive(Debug, Deserialize)]
pub struct SmserMessageCancelParam {
    pub message_id: String,
}

pub async fn smser_message_cancel(
    param: &SmserMessageCancelParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminSmsMgr {})
        .await?;
    let message_id = param.message_id.parse::<u64>()?;
    let msg = req_dao
        .web_dao
        .app_sender
        .smser
        .smser_dao
        .sms_record
        .find_message_by_id(message_id)
        .await?;
    let body = req_dao
        .web_dao
        .app_sender
        .smser
        .smser_dao
        .sms_record
        .find_body_by_id(msg.sender_body_id)
        .await?;
    let mut res = req_dao
        .web_dao
        .app_sender
        .smser
        .send_cancel(&body, &[&msg], auth_data.user_id(), Some(&req_dao.req_env))
        .await?;
    let mut out = None;
    if !res.is_empty() {
        if let Some(err) = res.remove(0).2 {
            return Err(err.into());
        } else {
            out = Some(message_id.to_string())
        }
    }
    Ok(JsonResponse::data(JsonData::body(json!({
        "data":out
    }))))
}
