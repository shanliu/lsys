// 日志写入 + 查询

use lsys_core::db::{
    CursorPageData, CursorPageParam, Insert, QueryBuilderExt, TableMeta, TotalParam, TotalRow,
    WhereClause,
};
use lsys_core::utils::{
    now_time, string_clear, StringClear, STRING_CLEAR_FORMAT, STRING_CLEAR_XSS,
};
use sqlx::{MySql, Pool, QueryBuilder};
use tracing::error;

use crate::dao::result::WebResult;
use crate::model::*;

use super::WebFileCollector;

impl WebFileCollector {
    /// 构建日志查询的 WHERE 子句
    fn build_log_where<'a, 'args>(
        wb: &mut WhereClause<'a, 'args, MySql>,
        script_id: u64,
        request_id: Option<&str>,
        level: Option<u8>,
    ) {
        wb.and().field_eq("script_id", script_id);
        if let Some(rid) = request_id {
            let rid = rid.trim();
            if !rid.is_empty() {
                wb.and().field_eq("request_id", rid.to_owned());
            }
        }
        if let Some(lv) = level {
            wb.and().field_eq("level", lv);
        }
    }

    /// 内部构建 Insert 语句（内部会对 message 做统一清理）
    fn build_log_insert<S: ToString>(
        request_id: &str,
        script_id: u64,
        user_id: u64,
        app_id: u64,
        level: u8,
        message: S,
        add_time: u64,
    ) -> Insert<sqlx::MySql, CollectorLogModel> {
        let msg = string_clear(
            message.to_string(),
            StringClear::Option(STRING_CLEAR_XSS | STRING_CLEAR_FORMAT),
            None,
        );

        Insert::<_, CollectorLogModel>::new()
            .set(CollectorLogModel::REQUEST_ID, request_id)
            .set(CollectorLogModel::SCRIPT_ID, script_id)
            .set(CollectorLogModel::USER_ID, user_id)
            .set(CollectorLogModel::APP_ID, app_id)
            .set(CollectorLogModel::LEVEL, level)
            .set(CollectorLogModel::MESSAGE, msg)
            .set(CollectorLogModel::ADD_TIME, add_time)
    }

    /// 写入采集日志
    pub async fn add_log(
        &self,
        request_id: &str,
        script_id: u64,
        user_id: u64,
        app_id: u64,
        level: u8,
        message: &str,
    ) -> WebResult<u64> {
        let now = now_time().unwrap_or_default();
        let res: sqlx::mysql::MySqlQueryResult =
            Self::build_log_insert(request_id, script_id, user_id, app_id, level, message, now)
                .execute(&self.db)
                .await?;
        Ok(res.last_insert_id())
    }

    /// 静态写入日志（用于 callback / handler 等无 &self 场景）
    pub async fn add_log_raw(
        db: &Pool<MySql>,
        request_id: &str,
        script_id: u64,
        user_id: u64,
        app_id: u64,
        level: u8,
        message: &str,
    ) {
        let now = now_time().unwrap_or_default();
        match Self::build_log_insert(request_id, script_id, user_id, app_id, level, message, now)
            .execute(db)
            .await
        {
            Ok(_) => {}
            Err(e) => {
                error!(
                    "collector add_log_raw: failed to insert log request_id={} script_id={} level={} err={}",
                    request_id,
                    script_id,
                    level,
                    e
                );
            }
        }
    }

    /// 查询采集日志（分页）
    pub async fn list_logs(
        &self,
        script_id: u64,
        request_id: Option<&str>,
        level: Option<u8>,
        page: &CursorPageParam<u64>,
    ) -> WebResult<(Vec<CollectorLogModel>, CursorPageData<u64>)> {
        let query_limit = page.page_query("id");
        let mut qb = QueryBuilder::<MySql>::new(format!(
            "SELECT * FROM {}",
            CollectorLogModel::table_name()
        ));
        {
            let mut wb = WhereClause::new(&mut qb);
            Self::build_log_where(&mut wb, script_id, request_id, level);
            if query_limit.has_cursor() {
                query_limit.push_where(wb.and());
            }
        }
        query_limit.push_order_by(&mut qb);
        query_limit.push_limit(&mut qb);

        let mut data = qb
            .build_query_as::<CollectorLogModel>()
            .fetch_all(&self.db)
            .await?;

        let next = query_limit.finalize(&mut data, |d, c| d.id == *c, |d| d.id);
        Ok((data, next))
    }

    /// 日志总数
    pub async fn count_logs(
        &self,
        script_id: u64,
        request_id: Option<&str>,
        level: Option<u8>,
        total_param: &TotalParam,
    ) -> WebResult<TotalRow> {
        let query = total_param.total_count_query();
        let mut qb = if query.is_threshold_mode() {
            QueryBuilder::<MySql>::new(format!(
                "SELECT count(*) FROM (SELECT 1 FROM {}",
                CollectorLogModel::table_name()
            ))
        } else {
            QueryBuilder::<MySql>::new(format!(
                "SELECT count(*) FROM {}",
                CollectorLogModel::table_name()
            ))
        };
        {
            let mut wb = WhereClause::new(&mut qb);
            Self::build_log_where(&mut wb, script_id, request_id, level);
        }
        if query.is_threshold_mode() {
            query.push_limit(&mut qb);
            qb.push(") as t");
        }

        let count = qb
            .build_query_scalar()
            .fetch_one(&self.db)
            .await
            .unwrap_or(0i64) as u64;

        Ok(query.finalize(count))
    }
}


