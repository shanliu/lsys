use lsys_core::{now_time, LimitParam, RequestEnv};

use sqlx::{MySql, Pool, Transaction};
use sqlx_model::{executor_option, model_option_set, sql_format, Insert, Select};
use tracing::{debug, warn};

use super::LoggerResult;
use crate::model::{ChangeLogModel, ChangeLogModelRef};
use sqlx_model::SqlQuote;

pub trait ChangeLogData {
    fn log_type<'t>() -> &'t str; //日志类型
    fn message(&self) -> String; //保持时转换显示消息,不在显示时反序列化,防止结构改变时反序列化失败
    fn encode(&self) -> String; //更改是相关数据
}

pub struct ChangeLogger {
    db: Pool<MySql>,
}

impl ChangeLogger {
    pub fn new(db: Pool<MySql>) -> Self {
        Self { db }
    }
    pub async fn add<'t, T: ChangeLogData>(
        &self,
        data: &T,
        source_id: &Option<u64>,
        user_id: &Option<u64>,
        add_user_id: &Option<u64>,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) {
        let user_id = user_id.unwrap_or_default();
        let add_user_id = add_user_id.unwrap_or_default();
        let source_id = source_id.unwrap_or_default();
        let log_data = data.encode();
        let message = data.message();
        let log_type = T::log_type().to_string();
        let time = env_data
            .map(|e| e.request_time)
            .unwrap_or_else(|| now_time().unwrap_or_default());
        let user_ip = env_data
            .map(|e| {
                e.request_ip
                    .as_ref()
                    .map(|e| e.to_string())
                    .unwrap_or_default()
            })
            .unwrap_or_default()
            .chars()
            .take(40)
            .collect();
        let request_id = env_data
            .as_ref()
            .map(|e| {
                e.request_id
                    .as_ref()
                    .map(|e| e.to_string())
                    .unwrap_or_default()
            })
            .unwrap_or_default()
            .chars()
            .take(32)
            .collect();
        let request_user_agent = env_data
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

        let new_data = model_option_set!(ChangeLogModelRef, {
            log_type: log_type,
            message:message,
            log_data:log_data,
            user_id:user_id,
            source_id:source_id,
            add_user_id:add_user_id,
            user_ip:user_ip,
            request_id:request_id,
            add_time:time,
            request_user_agent:request_user_agent,
        });
        let res = executor_option!(
            {
                Insert::<sqlx::MySql, ChangeLogModel, _>::new(new_data)
                    .execute(db)
                    .await
            },
            transaction,
            &self.db,
            db
        );
        match res {
            Err(err) => warn!("add log fail:{}", err),
            Ok(r) => debug!("add log id:{}", r.last_insert_id()),
        };
    }
    pub async fn list_data(
        &self,
        log_type: &Option<String>,
        user_id: &Option<u64>,
        add_user_id: &Option<u64>,
        limit: &Option<LimitParam>,
    ) -> LoggerResult<(Vec<ChangeLogModel>, Option<u64>)> {
        let mut sqlwhere = vec![];
        if let Some(tmp) = log_type {
            sqlwhere.push(sql_format!("log_type = {}  ", tmp));
        }
        if let Some(uid) = user_id {
            sqlwhere.push(sql_format!("user_id={} ", uid));
        }
        if let Some(uid) = add_user_id {
            sqlwhere.push(sql_format!("add_user_id={} ", uid));
        }
        let tmp = if let Some(page) = limit {
            if sqlwhere.is_empty() {
                format!(
                    " {} order by {} {} ",
                    page.where_sql("id", None),
                    page.order_sql("id"),
                    page.limit_sql(),
                )
            } else {
                format!(
                    "{} {} order by {} {} ",
                    sqlwhere.join(" and "),
                    page.where_sql("id", Some("and")),
                    page.order_sql("id"),
                    page.limit_sql(),
                )
            }
        } else {
            format!("{}  order by id desc", sqlwhere.join(" and "))
        };
        let sql = if !sqlwhere.is_empty()
            || limit
                .as_ref()
                .map(|e| e.pos())
                .unwrap_or_default()
                .is_some()
        {
            sqlx_model::WhereOption::Where(tmp)
        } else {
            sqlx_model::WhereOption::NoWhere(tmp)
        };
        let mut data = Select::type_new::<ChangeLogModel>()
            .fetch_all_by_where::<ChangeLogModel, _>(&sql, &self.db)
            .await?;
        let next = limit
            .as_ref()
            .map(|page| page.tidy(&mut data))
            .unwrap_or_default()
            .map(|e| e.id);
        Ok((data, next))
    }
}
