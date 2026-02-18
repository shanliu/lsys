use lsys_core::{now_time, string_clear, RequestEnv, StringClear, STRING_CLEAR_FORMAT};

use lsys_core::db::{CursorPageData, CursorPageParam, Insert, SqlExpr, TableMeta};
use lsys_core::{db_option_executor, sql_format};
use sqlx::{MySql, Pool, Transaction};
use tracing::{debug, warn};

use super::LoggerResult;
use crate::model::ChangeLogModel;
use lsys_core::db::SqlQuote;

pub trait ChangeLogData {
    fn log_type() -> &'static str; //日志类型
    fn message(&self) -> String; //保持时转换显示消息,不在显示时反序列化,防止结构改变时反序列化失败
    fn encode(&self) -> String; //更改是相关数据
}

pub struct ChangeLoggerDao {
    db: Pool<MySql>,
}

impl ChangeLoggerDao {
    pub fn new(db: Pool<MySql>) -> Self {
        Self { db }
    }
    pub async fn add<T: ChangeLogData>(
        &self,
        data: &T,
        source_id: Option<u64>,   //相关记录ID
        add_user_id: Option<u64>, //当前操作用户ID
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) {
        let add_user_id = add_user_id.unwrap_or_default();
        let source_id = source_id.unwrap_or_default();
        let log_data = data.encode();
        let message = data.message();
        let log_type = T::log_type().to_string();
        let time = env_data
            .map(|e| e.request_time)
            .unwrap_or_else(|| now_time().unwrap_or_default());
        let user_ip: String = env_data
            .map(|e| {
                e.request_ip
                    .as_ref()
                    .map(|e| e.to_string())
                    .unwrap_or_default()
            })
            .unwrap_or_default()
            .chars()
            .take(46)
            .collect();
        let request_id: String = env_data
            .as_ref()
            .map(|e| {
                e.request_id
                    .as_ref()
                    .map(|e| e.to_string())
                    .unwrap_or_default()
            })
            .unwrap_or_default()
            .chars()
            .take(64)
            .collect();
        let request_user_agent: String = env_data
            .as_ref()
            .map(|e| {
                e.request_user_agent
                    .as_ref()
                    .map(|e| e.to_string())
                    .unwrap_or_default()
            })
            .unwrap_or_default()
            .chars()
            .take(254)
            .collect();
        let device_id: String = env_data
            .as_ref()
            .map(|e| {
                e.device_id
                    .as_ref()
                    .map(|e| e.to_string())
                    .unwrap_or_default()
            })
            .unwrap_or_default()
            .chars()
            .take(64)
            .collect();

        let res = db_option_executor!(
            db,
            {
                Insert::<ChangeLogModel>::new()
                    .set(ChangeLogModel::LOG_TYPE, log_type)
                    .set(ChangeLogModel::MESSAGE, message)
                    .set(ChangeLogModel::LOG_DATA, log_data)
                    .set(ChangeLogModel::SOURCE_ID, source_id)
                    .set(ChangeLogModel::ADD_USER_ID, add_user_id)
                    .set(ChangeLogModel::ADD_USER_IP, user_ip)
                    .set(ChangeLogModel::REQUEST_ID, request_id)
                    .set(ChangeLogModel::ADD_TIME, time)
                    .set(ChangeLogModel::DEVICE_ID, device_id)
                    .set(ChangeLogModel::REQUEST_USER_AGENT, request_user_agent)
                    .execute(db.as_executor())
                    .await
            },
            transaction,
            &self.db
        );
        match res {
            Err(err) => warn!("add log fail:{}", err),
            Ok(r) => debug!("add log id:{}", r.last_insert_id()),
        };
    }
    fn list_where(&self, log_type: Option<&str>, add_user_id: Option<u64>) -> Option<Vec<String>> {
        let mut sqlwhere = vec![];
        if let Some(tmp) = log_type {
            let tmp = string_clear(tmp, StringClear::Option(STRING_CLEAR_FORMAT), Some(33));
            if tmp.is_empty() {
                return None;
            }
            sqlwhere.push(sql_format!("log_type = {}  ", tmp));
        }
        if let Some(uid) = add_user_id {
            sqlwhere.push(sql_format!("add_user_id={} ", uid));
        }
        Some(sqlwhere)
    }
    pub async fn list_data(
        &self,
        log_type: Option<&str>,
        add_user_id: Option<u64>,
        limit: &CursorPageParam<u64>,
    ) -> LoggerResult<(Vec<ChangeLogModel>, CursorPageData<u64>)> {
        let sqlwhere = match self.list_where(log_type, add_user_id) {
            Some(t) => t,
            None => return Ok((vec![], CursorPageData::default())),
        };
        let query_limit = limit.page_query("id");
        let where_str = sqlwhere.join(" and ");
        let suff_sql = query_limit.build_query_sql(if sqlwhere.is_empty() {
            None
        } else {
            Some(&where_str)
        });

        let mut data = sqlx::query_as::<_, ChangeLogModel>(&sql_format!(
            "select * from {} {}",
            ChangeLogModel::table_name(),
            SqlExpr(suff_sql)
        ))
        .fetch_all(&self.db)
        .await?;

        let next = query_limit.finalize(&mut data, |c, d| *d == c.id, |c| c.id);
        Ok((data, next))
    }
    pub async fn list_count(
        &self,
        log_type: Option<&str>,
        add_user_id: Option<u64>,
    ) -> LoggerResult<i64> {
        let sqlwhere = match self.list_where(log_type, add_user_id) {
            Some(t) => t,
            None => return Ok(0),
        };
        let where_sql = if sqlwhere.is_empty() {
            "".to_string()
        } else {
            format!("where {} ", sqlwhere.join(" and "))
        };
        return Ok(sqlx::query_scalar::<_, i64>(&sql_format!(
            "select count(*) as total from {} {}",
            ChangeLogModel::table_name(),
            SqlExpr(where_sql)
        ))
        .fetch_one(&self.db)
        .await?);
    }
}
