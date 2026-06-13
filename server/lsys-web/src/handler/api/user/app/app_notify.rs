use crate::common::JsonData;
use crate::common::ToCursorPageParam;
use crate::common::{JsonResponse, JsonResult, LimitParam};
use crate::common::{RequestDao, UserAuthQueryDao};
use crate::dao::WebDao;
use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::user::CheckUserNotifyView;
use lsys_access::dao::AccessSession;
use lsys_app::model::AppNotifyDataStatus;
use lsys_core::api_utils::{JsonPageData, PageCursorValue, PageTotalRowValue};
use lsys_core::db::{CursorPageSort, TotalParam};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct NotifyDataListParam {
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u64")]
    pub app_id: Option<u64>,
    pub notify_method: Option<String>,
    pub notify_key: Option<String>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_i8")]
    pub status: Option<i8>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub attr_callback_data: Option<bool>,
    pub limit: Option<LimitParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
}

#[derive(Serialize)]
pub struct NotifyDataListRecord {
    pub id: u64,
    pub app_id: u64,
    pub notify_method: String,
    pub notify_type: u8,
    pub notify_key: String,
    pub notify_payload: String,
    pub call_url: String,
    pub status: i8,
    pub result: String,
    pub try_num: u8,
    pub try_max: u8,
    pub publish_time: u64,
    pub next_time: u64,
}

pub async fn notify_data_list(
    param: &NotifyDataListParam,
    req_dao: &RequestDao,
    auth_dao: &UserAuthQueryDao,
    web_dao: &WebDao,
) -> JsonResult<JsonResponse> {
    let auth_data = auth_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await?;
    web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserNotifyView {
                res_user_id: auth_data.user_id(),
            },
        )
        .await?;

    let status = if let Some(e) = param.status {
        Some(match AppNotifyDataStatus::try_from(e) {
            Ok(ts) => vec![ts],
            Err(err) => return Err(err.into()),
        })
    } else {
        Some(vec![
            AppNotifyDataStatus::Init,
            AppNotifyDataStatus::Succ,
            AppNotifyDataStatus::Fail,
        ])
    };

    let res = web_dao
        .web_app
        .app_dao
        .app_notify
        .record
        .data_list(
            param.app_id,
            Some(auth_data.user_id()),
            param.notify_method.as_deref(),
            param.notify_key.as_deref(),
            status.as_deref(),
            param.attr_callback_data.unwrap_or(false),
            &param.limit.to_u64_cursor_page_param(CursorPageSort::Desc),
        )
        .await?;
    let cursor = PageCursorValue::from(&res.1);
    let out = res
        .0
        .into_iter()
        .map(|e| NotifyDataListRecord {
            id: e.0.id,
            status: e.0.status,
            app_id: e.0.app_id,
            notify_method: e.0.notify_method,
            notify_type: e.0.notify_type,
            notify_key: e.0.notify_key,
            notify_payload: e.0.notify_payload,
            call_url: e.1,
            result: e.0.result,
            try_num: e.0.try_num,
            try_max: e.0.try_max,
            publish_time: e.0.publish_time,
            next_time: e.0.next_time,
        })
        .collect::<Vec<_>>();

    let count = if param.count_num.unwrap_or(false) {
        Some(
            web_dao
                .web_app
                .app_dao
                .app_notify
                .record
                .data_count(
                    param.app_id,
                    Some(auth_data.user_id()),
                    param.notify_method.as_deref(),
                    param.notify_key.as_deref(),
                    status.as_deref(),
                    &TotalParam::default(),
                )
                .await
                .map(PageTotalRowValue::from)?,
        )
    } else {
        None
    };
    Ok(JsonResponse::data(JsonData::body(JsonPageData::cursor(
        out, cursor, count,
    ))))
}

#[derive(Deserialize)]
pub struct NotifyDataDelParam {
    #[serde(default, deserialize_with = "crate::common::deserialize_u64")]
    pub id: u64,
}

pub async fn notify_data_del(
    param: &NotifyDataDelParam,
    req_dao: &RequestDao,
    auth_dao: &UserAuthQueryDao,
    web_dao: &WebDao,
) -> JsonResult<JsonResponse> {
    let auth_data = auth_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await?;

    let notify = web_dao
        .web_app
        .app_dao
        .app_notify
        .record
        .find_data_by_id(&param.id)
        .await?;

    let app = web_dao
        .web_app
        .app_dao
        .app
        .find_by_id(notify.app_id)
        .await?;
    web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserNotifyView {
                res_user_id: app.user_id,
            },
        )
        .await?;

    web_dao
        .web_app
        .app_dao
        .app_notify
        .remove_notify(&notify, auth_data.user_id(), Some(&req_dao.req_env))
        .await?;
    Ok(JsonResponse::default())
}
