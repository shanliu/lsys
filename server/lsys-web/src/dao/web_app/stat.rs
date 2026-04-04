// stat 统计数据操作相关封装
use crate::dao::{WebApp, WebResult};
use lsys_app::model::{AppModel, AppNotifyDataModel, AppOAuthClientAccessModel, AppRequestModel};
use lsys_core::utils::now_time;
use lsys_core::db::TableMeta;
use serde::Serialize;
use sqlx::FromRow;

#[derive(FromRow, Serialize, Debug, Clone)]
pub struct DailyStat {
    pub date: String,
    pub total: i64,
}

#[derive(FromRow, Serialize, Debug, Clone)]
pub struct StatusDailyStat {
    pub date: String,
    pub status: i8,
    pub total: i64,
}

#[derive(FromRow, Serialize, Debug, Clone)]
pub struct NotifyTypeDailyStat {
    pub date: String,
    pub notify_type: u8,
    pub status: i8,
    pub total: i64,
}

impl WebApp {
    /// 统计 AppNotifyDataModel 按 notify_type & status 各状态总数
    /// 指定倒数天数内每天的总数汇总
    pub async fn app_notify_data(
        &self,
        app_id: u64,
        days: u64,
    ) -> WebResult<Vec<NotifyTypeDailyStat>> {
        let now = now_time().unwrap_or_default();
        let start_time = now.saturating_sub(days * 86400);

        let sql = format!(
            "SELECT DATE_FORMAT(FROM_UNIXTIME(create_time),'%Y-%m-%d') as date, 
                    notify_type, 
                    status, 
                    COUNT(*) as total
             FROM {}
             WHERE app_id = ? 
               AND create_time >= ?
               AND create_time <= ?
             GROUP BY DATE_FORMAT(FROM_UNIXTIME(create_time),'%Y-%m-%d'), notify_type, status
             ORDER BY date ASC, notify_type, status",
            AppNotifyDataModel::table_name()
        );

        let result = sqlx::query_as::<_, NotifyTypeDailyStat>(&sql)
            .bind(app_id)
            .bind(start_time)
            .bind(now)
            .fetch_all(&self.db)
            .await?;

        Ok(result)
    }

    /// 统计 AppOAuthClientAccessModel 指定倒数天数内每天的总数汇总
    pub async fn app_oauth_client_access(
        &self,
        app_id: u64,
        days: u64,
    ) -> WebResult<Vec<DailyStat>> {
        let now = now_time().unwrap_or_default();
        let start_time = now.saturating_sub(days * 86400);

        let sql = format!(
            "SELECT DATE_FORMAT(FROM_UNIXTIME(add_time),'%Y-%m-%d') as date, 
                    COUNT(*) as total
             FROM {}
             WHERE app_id = ? 
               AND add_time >= ?
               AND add_time <= ?
             GROUP BY DATE_FORMAT(FROM_UNIXTIME(add_time),'%Y-%m-%d')
             ORDER BY date ASC",
            AppOAuthClientAccessModel::table_name()
        );

        let result = sqlx::query_as::<_, DailyStat>(&sql)
            .bind(app_id)
            .bind(start_time)
            .bind(now)
            .fetch_all(&self.db)
            .await?;

        Ok(result)
    }

    /// 统计 AppModel parent_app_id=(参数 app_id) & status (Enable 跟全部)
    /// 指定倒数天数内每天的总数汇总
    pub async fn app_day_stat(&self, app_id: u64, days: u64) -> WebResult<Vec<StatusDailyStat>> {
        let now = now_time().unwrap_or_default();
        let start_time = now.saturating_sub(days * 86400);

        let sql = format!(
            "SELECT DATE_FORMAT(FROM_UNIXTIME(change_time),'%Y-%m-%d') as date, 
                    status, 
                    COUNT(*) as total
             FROM {}
             WHERE parent_app_id = ? 
               AND change_time >= ?
               AND change_time <= ?
             GROUP BY DATE_FORMAT(FROM_UNIXTIME(change_time),'%Y-%m-%d'), status
             ORDER BY date ASC, status",
            AppModel::table_name()
        );

        let result = sqlx::query_as::<_, StatusDailyStat>(&sql)
            .bind(app_id)
            .bind(start_time)
            .bind(now)
            .fetch_all(&self.db)
            .await?;

        Ok(result)
    }

    /// 统计 AppRequestModel parent_app_id=(参数 app_id) & status ((Approved+Rejected) 跟全部)
    /// 指定倒数天数内每天的总数汇总
    pub async fn app_request(&self, app_id: u64, days: u64) -> WebResult<Vec<StatusDailyStat>> {
        let now = now_time().unwrap_or_default();
        let start_time = now.saturating_sub(days * 86400);

        let sql = format!(
            "SELECT DATE_FORMAT(FROM_UNIXTIME(request_time),'%Y-%m-%d') as date, 
                    status, 
                    COUNT(*) as total
             FROM {}
             WHERE parent_app_id = ? 
               AND request_time >= ?
               AND request_time <= ?
             GROUP BY DATE_FORMAT(FROM_UNIXTIME(request_time),'%Y-%m-%d'), status
             ORDER BY date ASC, status",
            AppRequestModel::table_name()
        );

        let result = sqlx::query_as::<_, StatusDailyStat>(&sql)
            .bind(app_id)
            .bind(start_time)
            .bind(now)
            .fetch_all(&self.db)
            .await?;

        Ok(result)
    }
}
