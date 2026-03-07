// 日志写入 + 查询

use lsys_core::db::{CursorPageData, CursorPageParam, Insert, SqlQuote, TableMeta};
use lsys_core::fluent_message;
use lsys_core::sql_format;
use lsys_core::utils::{
    now_time, string_clear, StringClear, STRING_CLEAR_FORMAT, STRING_CLEAR_XSS,
};
use sqlx::{MySql, Pool};
use tracing::error;

use crate::dao::result::{WebError, WebResult};
use crate::model::*;

use super::WebFileCollector;

impl WebFileCollector {
    /// 构建日志查询的 WHERE 子句
    fn build_log_where(script_id: u64, request_id: Option<&str>, level: Option<u8>) -> String {
        let mut clauses: Vec<String> = vec![sql_format!("script_id={}", script_id)];
        if let Some(rid) = request_id {
            let rid = rid.trim();
            if !rid.is_empty() {
                clauses.push(sql_format!("request_id={}", rid));
            }
        }
        if let Some(lv) = level {
            clauses.push(sql_format!("level={}", lv));
        }
        clauses.join(" AND ")
    }

    /// 内部构建 Insert 语句（内部会对 message 做统一清理）
    fn build_log_insert<'a, S: ToString>(
        request_id: &'a str,
        script_id: u64,
        user_id: u64,
        app_id: u64,
        level: u8,
        message: S,
        add_time: u64,
    ) -> Insert<'a, sqlx::MySql, CollectorLogModel> {
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
        let now = now_time().map_err(|e| WebError::Message(fluent_message!("time-error", e)))?;
        let res =
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
        let now = now_time().unwrap_or(0);
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
        let where_str = Self::build_log_where(script_id, request_id, level);
        let query_limit = page.page_query("id");
        let suff_sql = query_limit.build_query_sql(Some(&where_str));

        let sql = format!(
            "SELECT * FROM {} {}",
            CollectorLogModel::table_name().sql_quote(),
            suff_sql
        );

        let mut data = sqlx::query_as::<_, CollectorLogModel>(&sql)
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
    ) -> WebResult<u64> {
        let where_str = Self::build_log_where(script_id, request_id, level);
        let sql = format!(
            "SELECT COUNT(*) FROM {} WHERE {}",
            CollectorLogModel::table_name().sql_quote(),
            where_str
        );

        let count = sqlx::query_scalar::<_, i64>(&sql)
            .fetch_one(&self.db)
            .await?;
        Ok(count as u64)
    }
}
