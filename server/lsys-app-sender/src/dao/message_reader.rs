use std::{collections::HashMap, sync::Arc};

use crate::{
    dao::SenderResult,
    model::{SenderMessageCancelModel, SenderType},
};

use lsys_core::db::{QueryBuilderExt, TableMeta};
use lsys_core::app_core::AppCore;
use parking_lot::Mutex;
use sqlx::{FromRow, MySql, Pool, QueryBuilder};

use lsys_core::task_dispatch::{TaskData, TaskItem};
use lsys_core::utils::now_time;

//统一任务消息读取实现

pub struct MessageReader<BM, MM>
where
    for<'t> BM: FromRow<'t, sqlx::mysql::MySqlRow> + Send + Unpin + TableMeta,
    for<'t> MM: FromRow<'t, sqlx::mysql::MySqlRow> + Send + Unpin + TableMeta,
{
    db: Pool<MySql>,
    id_generator: Arc<Mutex<snowflake::SnowflakeIdGenerator>>,
    marker_task: std::marker::PhantomData<BM>,
    marker_message: std::marker::PhantomData<MM>,
    send_type: SenderType,
}

impl<BM, MM> MessageReader<BM, MM>
where
    for<'r> BM: FromRow<'r, sqlx::mysql::MySqlRow> + Send + Unpin + TableMeta,
    for<'t> MM: FromRow<'t, sqlx::mysql::MySqlRow> + Send + Unpin + TableMeta,
{
    pub fn new(db: Pool<sqlx::MySql>, app_core: Arc<AppCore>, send_type: SenderType) -> Self {
        let id_generator = Arc::new(Mutex::new(lsys_core::app_core::create_snowflake_id_generator(app_core.as_ref())));
        Self {
            id_generator,
            db,
            marker_task: std::marker::PhantomData,
            marker_message: std::marker::PhantomData,
            send_type,
        }
    }
    pub fn message_id(&self) -> u64 {
        self.id_generator.lock().real_time_generate() as u64
    }
    //读取邮件任务数据
    pub async fn read_task(
        &self,
        tasking_record: &HashMap<u64, TaskData>,
        status: i8,
        limit: usize,
    ) -> SenderResult<(Vec<BM>, bool)> {
        let mut qb: QueryBuilder<'_, MySql> = QueryBuilder::new(format!(
            "select * from {}",
            BM::table_name(),
        ));
        qb.push_where().field_lte("expected_time", now_time().unwrap_or_default() as i64);
        qb.push_and().field_eq("status", status);
        let ids = tasking_record.keys().copied().collect::<Vec<u64>>();
        if !ids.is_empty() {
            qb.push_and().field_not_in("id", &ids);
        }
        qb.push(" ORDER BY id ASC LIMIT ").push_bind((limit + 1) as i64);

        let mut app_res = qb.build_query_as::<BM>()
            .fetch_all(&self.db)
            .await?;

        let next = if app_res.len() > limit {
            app_res.pop();
            true
        } else {
            false
        };
        Ok((app_res, next))
    }
    pub async fn read_message<TD: TaskItem<u64>>(
        &self,
        record: &TD,
        sending_data: &[u64],
        status: i8,
        limit: u16,
    ) -> SenderResult<Vec<MM>> {
        let mut qb: QueryBuilder<'_, MySql> = QueryBuilder::new(format!(
            "select * from {}",
            MM::table_name(),
        ));
        qb.push_where().field_eq("sender_body_id", record.to_task_pk());
        qb.push_and().field_eq("status", status);
        qb.push_and().push(format!("id not in (select id from {}", SenderMessageCancelModel::table_name()));
        qb.push_where().field_eq("sender_body_id", record.to_task_pk());
        qb.push_and().field_eq("sender_type", self.send_type as i8);
        qb.push(")");

        if !sending_data.is_empty() {
            qb.push_and().field_not_in("id", sending_data);
        }
        qb.push(" ORDER BY id ASC LIMIT ").push_bind(limit as i64);

        Ok(qb.build_query_as::<MM>()
            .fetch_all(&self.db)
            .await?)
    }
    pub async fn find_message_by_snid_vec(&self, ids: &[u64]) -> SenderResult<Vec<MM>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let mut qb: QueryBuilder<'_, MySql> = QueryBuilder::new(format!(
            "select * from {}",
            MM::table_name()
        ));
        qb.push_where().field_in_copied("snid", ids);
        Ok(qb.build_query_as::<MM>()
            .fetch_all(&self.db)
            .await?)
    }
    pub async fn find_message_by_id(&self, id: &u64) -> SenderResult<MM> {
        use lsys_core::db::utils::Fetch;
        Ok(Fetch::<MySql, MM>::one(
            &self.db,
            |qb| { qb.field_eq("id", *id); },
        ).await?)
    }
    pub async fn find_body_by_id(&self, id: &u64) -> SenderResult<BM> {
        use lsys_core::db::utils::Fetch;
        Ok(Fetch::<MySql, BM>::one(
            &self.db,
            |qb| { qb.field_eq("id", *id); },
        ).await?)
    }
    pub async fn find_body_by_id_vec(&self, ids: &[u64]) -> SenderResult<Vec<BM>> {
        let mut qb: QueryBuilder<'_, MySql> = QueryBuilder::new(format!(
            "select * from {}",
            BM::table_name()
        ));
        qb.push_where().field_in_copied("id", ids);
        Ok(qb.build_query_as::<BM>()
            .fetch_all(&self.db)
            .await?)
    }
}
