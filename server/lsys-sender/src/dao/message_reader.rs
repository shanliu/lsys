use std::{collections::HashMap, sync::Arc};

use crate::{
    dao::SenderResult,
    model::{SenderMessageCancelModel, SenderType},
};

use lsys_core::{now_time, AppCore, FluentMessage};
use parking_lot::Mutex;
use sqlx::{FromRow, MySql, Pool};
use sqlx_model::{sql_format, ModelTableField, ModelTableName, Select, SqlExpr};

use lsys_core::{TaskData, TaskItem};
use sqlx_model::SqlQuote;

//统一任务消息读取实现

pub struct MessageReader<BM, MM>
where
    for<'t> BM:
        FromRow<'t, sqlx::mysql::MySqlRow> + Send + Unpin + ModelTableName + ModelTableField<MySql>,
    for<'t> MM:
        FromRow<'t, sqlx::mysql::MySqlRow> + Send + Unpin + ModelTableName + ModelTableField<MySql>,
{
    db: Pool<MySql>,
    id_generator: Arc<Mutex<snowflake::SnowflakeIdGenerator>>,
    marker_task: std::marker::PhantomData<BM>,
    marker_message: std::marker::PhantomData<MM>,
    send_type: SenderType,
}

impl<BM, MM> MessageReader<BM, MM>
where
    for<'r> BM:
        FromRow<'r, sqlx::mysql::MySqlRow> + Send + Unpin + ModelTableName + ModelTableField<MySql>,
    for<'t> MM:
        FromRow<'t, sqlx::mysql::MySqlRow> + Send + Unpin + ModelTableName + ModelTableField<MySql>,
{
    pub fn new(
        db: Pool<sqlx::MySql>,
        app_core: Arc<AppCore>,
        send_type: SenderType,
        _fluent: Arc<FluentMessage>,
    ) -> Self {
        let id_generator = Arc::new(Mutex::new(app_core.create_snowflake_id_generator()));
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
        let mut app_res = Select::type_new::<BM>()
            .fetch_all_by_where::<BM, _>(
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
    //读取邮件任务数据
    pub async fn read_message<TD: TaskItem<u64>>(
        &self,
        record: &TD,
        sending_data: &[u64],
        status: i8,
        limit: u16,
    ) -> SenderResult<Vec<MM>> {
        let mut sql_vec = vec![];
        sql_vec.push(sql_format!(
            "sender_body_id={} and status = {} and id not in (select id from {} where sender_body_id={} and sender_type={})",
            record.to_task_pk(),
            status,
            SqlExpr(SenderMessageCancelModel::table_name()),
            record.to_task_pk(),
            self.send_type as i8,
        ));

        if !sending_data.is_empty() {
            sql_vec.push(sql_format!(" id not in ({})", sending_data));
        }
        Ok(Select::type_new::<MM>()
            .fetch_all_by_where::<MM, _>(
                &sqlx_model::WhereOption::Where(format!(
                    "{} order by id asc limit {}",
                    sql_vec.join(" and "),
                    limit
                )),
                &self.db,
            )
            .await?)
    }
    pub async fn find_message_by_snid_vec(&self, ids: &[u64]) -> SenderResult<Vec<MM>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        Ok(Select::type_new::<MM>()
            .fetch_all_by_where::<MM, _>(
                &sqlx_model::WhereOption::Where(sql_format!("snid in ({})", ids)),
                &self.db,
            )
            .await?)
    }
    lsys_core::impl_dao_fetch_one_by_one!(
        db,
        find_message_by_id,
        u64,
        MM,
        SenderResult<MM>,
        id,
        "id={id}"
    );
    lsys_core::impl_dao_fetch_one_by_one!(
        db,
        find_body_by_id,
        u64,
        BM,
        SenderResult<BM>,
        id,
        "id={id}"
    );
    pub async fn find_body_by_id_vec(&self, ids: &[u64]) -> SenderResult<Vec<BM>> {
        Ok(Select::type_new::<BM>()
            .fetch_all_by_where::<BM, _>(
                &sqlx_model::WhereOption::Where(sql_format!("id in ({})", ids)),
                &self.db,
            )
            .await?)
    }
}
