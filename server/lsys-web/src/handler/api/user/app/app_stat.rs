use crate::common::JsonData;
use crate::common::UserAuthQueryDao;
use crate::common::{JsonResponse, JsonResult};
use crate::dao::access::api::system::user::CheckUserAppView;
use crate::dao::access::RbacAccessCheckEnv;
use chrono::{Duration, NaiveDate};
use lsys_access::dao::AccessSession;
use lsys_app::model::{AppNotifyDataStatus, AppRequestStatus, AppStatus};
use lsys_core::now_time;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::convert::TryInto;

// 计算3个月的天数（基于当前时间）
fn get_max_days() -> u64 {
    let now = now_time().unwrap_or_default();
    // 计算3个月前的时间戳（约90天，实际根据月份天数计算）
    let three_months_seconds = 3 * 30 * 86400; // 先用30天作为估算
    let three_months_ago = now.saturating_sub(three_months_seconds);

    // 更精确计算：获取实际天数差
    (now - three_months_ago) / 86400
}

#[derive(Deserialize)]
pub struct AppStatParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub days: u64,
}

#[derive(Serialize)]
pub struct AppStatData {
    pub notify_data: NotifyStatGroup,
    pub oauth_access: Vec<DailyStatData>,
    pub sub_app: SubAppStatGroup,
    pub request: RequestStatGroup,
}

#[derive(Serialize)]
pub struct NotifyStatGroup {
    pub success: Vec<NotifyStatData>, // status=2 的数据
    pub all: Vec<NotifyStatData>,     // 全部状态的数据
}

#[derive(Serialize)]
pub struct SubAppStatGroup {
    pub enable: Vec<StatusStatData>, // status=2 的数据
    pub all: Vec<StatusStatData>,    // 全部状态的数据
}

#[derive(Serialize)]
pub struct RequestStatGroup {
    pub processed: Vec<StatusStatData>, // status=2+3 的数据
    pub all: Vec<StatusStatData>,       // 全部状态的数据
}

#[derive(Serialize, Clone)]
pub struct NotifyStatData {
    pub date: String,
    pub notify_type: u8,
    pub status: i8,
    pub total: i64,
}

#[derive(Serialize, Clone)]
pub struct DailyStatData {
    pub date: String,
    pub total: i64,
}

#[derive(Serialize, Clone)]
pub struct StatusStatData {
    pub date: String,
    pub status: i8,
    pub total: i64,
}

fn build_date_range(days: u64, end_ts: u64) -> Vec<String> {
    if days == 0 {
        return Vec::new();
    }
    let default_date = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
    const UNIX_EPOCH_DAYS_FROM_CE: i32 = 719_163;
    let end_days = (end_ts / 86_400) as i64;
    let end_days_from_ce = UNIX_EPOCH_DAYS_FROM_CE as i64 + end_days;
    let end_date = end_days_from_ce
        .try_into()
        .ok()
        .and_then(NaiveDate::from_num_days_from_ce_opt)
        .unwrap_or(default_date);
    let offset_days = days.saturating_sub(1) as i64;
    let start_date = if offset_days > 0 {
        end_date
            .checked_sub_signed(Duration::days(offset_days))
            .unwrap_or(end_date)
    } else {
        end_date
    };

    (0..days)
        .filter_map(|index| {
            start_date
                .checked_add_signed(Duration::days(index as i64))
                .map(|date| date.format("%Y-%m-%d").to_string())
        })
        .collect()
}

fn fill_daily_stats(date_range: &[String], stats: &[DailyStatData]) -> Vec<DailyStatData> {
    let mut totals: HashMap<String, i64> = HashMap::new();
    for item in stats {
        totals.insert(item.date.clone(), item.total);
    }
    date_range
        .iter()
        .map(|date| DailyStatData {
            date: date.clone(),
            total: *totals.get(date).unwrap_or(&0),
        })
        .collect()
}

fn unique_status_order(stats: &[StatusStatData]) -> Vec<i8> {
    let mut seen = HashSet::new();
    let mut order = Vec::new();
    for item in stats {
        if seen.insert(item.status) {
            order.push(item.status);
        }
    }
    order
}

fn unique_notify_type_order(stats: &[NotifyStatData]) -> Vec<u8> {
    let mut seen = HashSet::new();
    let mut order = Vec::new();
    for item in stats {
        if seen.insert(item.notify_type) {
            order.push(item.notify_type);
        }
    }
    order
}

fn unique_notify_status_order(stats: &[NotifyStatData]) -> Vec<i8> {
    let mut seen = HashSet::new();
    let mut order = Vec::new();
    for item in stats {
        if seen.insert(item.status) {
            order.push(item.status);
        }
    }
    order
}

fn fill_status_stats_for_statuses(
    date_range: &[String],
    stats: &[StatusStatData],
    statuses: &[i8],
) -> Vec<StatusStatData> {
    let mut data: HashMap<i8, HashMap<String, i64>> = HashMap::new();
    for item in stats {
        data.entry(item.status)
            .or_default()
            .insert(item.date.clone(), item.total);
    }

    let mut result = Vec::new();
    for status in statuses {
        let map = data.get(status);
        for date in date_range {
            let total = map
                .and_then(|inner| inner.get(date))
                .copied()
                .unwrap_or(0);
            result.push(StatusStatData {
                date: date.clone(),
                status: *status,
                total,
            });
        }
    }
    result
}

fn fill_notify_stats_for_types(
    date_range: &[String],
    stats: &[NotifyStatData],
    notify_types: &[u8],
    statuses: &[i8],
) -> Vec<NotifyStatData> {
    let mut data: HashMap<(u8, i8), HashMap<String, i64>> = HashMap::new();
    for item in stats {
        data.entry((item.notify_type, item.status))
            .or_default()
            .insert(item.date.clone(), item.total);
    }

    let mut result = Vec::new();
    for notify_type in notify_types {
        for status in statuses {
            let map = data.get(&(*notify_type, *status));
            for date in date_range {
                let total = map
                    .and_then(|inner| inner.get(date))
                    .copied()
                    .unwrap_or(0);
                result.push(NotifyStatData {
                    date: date.clone(),
                    notify_type: *notify_type,
                    status: *status,
                    total,
                });
            }
        }
    }
    result
}

fn fill_notify_stats_all_status(
    date_range: &[String],
    stats: &[NotifyStatData],
    notify_types: &[u8],
) -> Vec<NotifyStatData> {
    let mut data: HashMap<u8, HashMap<String, i64>> = HashMap::new();
    for item in stats {
        data.entry(item.notify_type)
            .or_default()
            .entry(item.date.clone())
            .and_modify(|total| *total += item.total)
            .or_insert(item.total);
    }

    let mut result = Vec::new();
    for notify_type in notify_types {
        let map = data.get(notify_type);
        for date in date_range {
            let total = map
                .and_then(|inner| inner.get(date))
                .copied()
                .unwrap_or(0);
            result.push(NotifyStatData {
                date: date.clone(),
                notify_type: *notify_type,
                status: 0,
                total,
            });
        }
    }
    result
}

/// 获取应用统计数据
pub async fn stat(param: &AppStatParam, req_dao: &UserAuthQueryDao) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    // 验证应用存在且用户有权限访问
    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .find_by_id(param.app_id)
        .await?;

    // 检查权限
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserAppView {
                res_user_id: app.user_id,
            },
        )
        .await?;

    // 限制天数最大为3个月
    let max_days = get_max_days();
    let days = if param.days > max_days {
        max_days
    } else {
        param.days
    };
    let end_ts = now_time().unwrap_or_default();
    let date_labels = build_date_range(days, end_ts);

    // 获取通知数据统计（一次查询）
    let notify_stats_raw = req_dao
        .web_dao
        .web_app
        .yaf_app_notify_data(app.id, days)
        .await?
        .into_iter()
        .map(|e| NotifyStatData {
            date: e.date,
            notify_type: e.notify_type,
            status: e.status,
            total: e.total,
        })
        .collect::<Vec<_>>();
    let mut notify_types = unique_notify_type_order(&notify_stats_raw);
    if notify_types.is_empty() {
        notify_types.push(0);
    }
    let mut notify_statuses = unique_notify_status_order(&notify_stats_raw);
    if notify_statuses.is_empty() {
        notify_statuses.push(AppNotifyDataStatus::Succ as i8);
    }
    let mut notify_data_all = fill_notify_stats_all_status(&date_labels, &notify_stats_raw, &notify_types);
    let notify_status_details =
        fill_notify_stats_for_types(&date_labels, &notify_stats_raw, &notify_types, &notify_statuses);
    notify_data_all.extend(notify_status_details);
    let notify_data_success = fill_notify_stats_for_types(
        &date_labels,
        &notify_stats_raw,
        &notify_types,
        &[AppNotifyDataStatus::Succ as i8],
    );

    // 获取OAuth客户端访问统计
    let oauth_access_raw = req_dao
        .web_dao
        .web_app
        .yaf_app_oauth_client_access(app.id, days)
        .await?
        .into_iter()
        .map(|e| DailyStatData {
            date: e.date,
            total: e.total,
        })
        .collect::<Vec<_>>();
    let oauth_access = fill_daily_stats(&date_labels, &oauth_access_raw);

    // 获取子应用统计（一次查询）
    let sub_app_stats_raw = req_dao
        .web_dao
        .web_app
        .yaf_app(app.id, days)
        .await?
        .into_iter()
        .map(|e| StatusStatData {
            date: e.date,
            status: e.status,
            total: e.total,
        })
        .collect::<Vec<_>>();
    let mut sub_app_statuses = unique_status_order(&sub_app_stats_raw);
    if sub_app_statuses.is_empty() {
        sub_app_statuses.push(AppStatus::Enable as i8);
    }
    let sub_app_all =
        fill_status_stats_for_statuses(&date_labels, &sub_app_stats_raw, &sub_app_statuses);
    let sub_app_enable = fill_status_stats_for_statuses(
        &date_labels,
        &sub_app_stats_raw,
        &[AppStatus::Enable as i8],
    );

    // 获取应用请求统计（一次查询）
    let request_stats_raw = req_dao
        .web_dao
        .web_app
        .yaf_app_request(app.id, days)
        .await?
        .into_iter()
        .map(|e| StatusStatData {
            date: e.date,
            status: e.status,
            total: e.total,
        })
        .collect::<Vec<_>>();
    let mut request_statuses = unique_status_order(&request_stats_raw);
    if request_statuses.is_empty() {
        request_statuses = vec![
            AppRequestStatus::Approved as i8,
            AppRequestStatus::Rejected as i8,
        ];
    }
    let request_all =
        fill_status_stats_for_statuses(&date_labels, &request_stats_raw, &request_statuses);
    let request_processed = fill_status_stats_for_statuses(
        &date_labels,
        &request_stats_raw,
        &[
            AppRequestStatus::Approved as i8,
            AppRequestStatus::Rejected as i8,
        ],
    );

    Ok(JsonResponse::data(JsonData::body(json!({
        "data": AppStatData {
            notify_data: NotifyStatGroup {
                success: notify_data_success,
                all: notify_data_all,
            },
            oauth_access,
            sub_app: SubAppStatGroup {
                enable: sub_app_enable,
                all: sub_app_all,
            },
            request: RequestStatGroup {
                processed: request_processed,
                all: request_all,
            },
        }
    }))))
}
