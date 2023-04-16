use std::{collections::HashMap, sync::Arc};

use crate::dao::SenderResult;

use lsys_core::{now_time, AppCore, FluentMessage, PageParam};
use parking_lot::Mutex;
use snowflake::SnowflakeIdGenerator;
use sqlx::{FromRow, MySql, Pool};
use sqlx_model::{sql_format, ModelTableField, ModelTableName, Select, SqlExpr};

use super::TaskData;
use sqlx_model::SqlQuote;

//统一任务消息读取实现

pub struct MessageReader<M>
where
    for<'t> M:
        FromRow<'t, sqlx::mysql::MySqlRow> + Send + Unpin + ModelTableName + ModelTableField<MySql>,
{
    db: Pool<MySql>,
    id_generator: Arc<Mutex<SnowflakeIdGenerator>>,
    marker: std::marker::PhantomData<M>,
}

impl<M> MessageReader<M>
where
    for<'r> M:
        FromRow<'r, sqlx::mysql::MySqlRow> + Send + Unpin + ModelTableName + ModelTableField<MySql>,
{
    pub fn new(db: Pool<sqlx::MySql>, app_core: Arc<AppCore>, _fluent: Arc<FluentMessage>) -> Self {
        //TODO  这个生成ID 库有BUG...
        let machine_id = app_core.config.get_int("snowflake_machine_id").unwrap_or(1);
        let node_id = app_core
            .config
            .get_int("snowflake_node_id")
            .unwrap_or_else(|_| {
                crc32fast::hash(
                    hostname::get()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .as_bytes(),
                )
                .into()
            });
        //TODO  这个生成ID 库有BUG...
        let id_generator = Arc::new(Mutex::new(SnowflakeIdGenerator::new(
            machine_id as i32,
            node_id as i32,
        )));
        Self {
            id_generator,
            db,
            marker: std::marker::PhantomData::default(),
        }
    }
    //读取邮件任务数据
    pub async fn read(
        &self,
        tasking_record: &HashMap<u64, TaskData>,
        status: i8,
        limit: usize,
    ) -> SenderResult<(Vec<M>, bool)> {
        let mut sql_vec = vec![];
        sql_vec.push(sql_format!(
            "expected_time<={} and status = {}",
            now_time().unwrap_or_default(),
            status
        ));
        let ids = tasking_record.keys().copied().collect::<Vec<u64>>();
        if !ids.is_empty() {
            sql_vec.push(sql_format!(" id not in ({})", ids));
        }
        let mut app_res = Select::type_new::<M>()
            .fetch_all_by_where::<M, _>(
                &sqlx_model::WhereOption::Where(format!(
                    "{} order by id asc limit {}",
                    sql_vec.join(" and "),
                    limit + 1
                )),
                &self.db,
            )
            .await?;
        let next = if app_res.len() > limit {
            app_res.pop();
            true
        } else {
            false
        };
        Ok((app_res, next))
    }
    lsys_core::impl_dao_fetch_one_by_one!(
        db,
        find_message_by_id,
        u64,
        M,
        SenderResult<M>,
        id,
        "id={id}"
    );
    pub async fn message_count(
        &self,
        user_id: &Option<u64>,
        app_id: &Option<u64>,
        tpl_id: &Option<String>,
        status: &Option<i8>,
        sql_where: Option<String>,
    ) -> SenderResult<i64> {
        let mut sqlwhere = vec![];
        if let Some(aid) = app_id {
            sqlwhere.push(sql_format!("app_id = {}  ", aid));
        }
        if let Some(uid) = user_id {
            sqlwhere.push(sql_format!("user_id={} ", uid));
        }
        if let Some(t) = tpl_id {
            sqlwhere.push(sql_format!("tpl_id={} ", t));
        }
        if let Some(s) = status {
            sqlwhere.push(sql_format!("status={} ", *s));
        }
        if let Some(s) = sql_where {
            sqlwhere.push(s);
        }
        let sql = sql_format!(
            "select count(*) as total from {} {}",
            M::table_name(),
            SqlExpr(if sqlwhere.is_empty() {
                "".to_string()
            } else {
                format!("where {}", sqlwhere.join(" and "))
            })
        );
        let query = sqlx::query_scalar::<_, i64>(&sql);
        let res = query.fetch_one(&self.db).await?;
        Ok(res)
    }
    pub async fn message_list(
        &self,
        user_id: &Option<u64>,
        app_id: &Option<u64>,
        tpl_id: &Option<String>,
        status: &Option<i8>,
        sql_where: Option<String>,
        page: &Option<PageParam>,
    ) -> SenderResult<Vec<M>> {
        let mut sqlwhere = vec![];
        if let Some(aid) = app_id {
            sqlwhere.push(sql_format!("app_id = {}  ", aid));
        }
        if let Some(uid) = user_id {
            sqlwhere.push(sql_format!("user_id={} ", uid));
        }
        if let Some(t) = tpl_id {
            sqlwhere.push(sql_format!("tpl_id={} ", t));
        }
        if let Some(s) = status {
            sqlwhere.push(sql_format!("status={} ", *s));
        }
        if let Some(s) = sql_where {
            sqlwhere.push(s);
        }
        let mut sql = format!("{}  order by id desc", sqlwhere.join(" and "));
        if let Some(pdat) = page {
            sql += format!(" limit {} offset {}", pdat.limit, pdat.offset).as_str();
        }
        let data = Select::type_new::<M>()
            .fetch_all_by_where::<M, _>(&sqlx_model::WhereOption::Where(sql), &self.db)
            .await?;
        Ok(data)
    }
    pub fn message_id(&self) -> u64 {
        self.id_generator.lock().real_time_generate() as u64
    }
}
