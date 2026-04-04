use async_trait::async_trait;
use lsys_core::db::{QueryBuilderExt, TableMeta};
use lsys_core::task_dispatch::{TaskAcquisition, TaskData, TaskItem, TaskRecord};
use lsys_core::utils::now_time;
use sqlx::{MySql, Pool, QueryBuilder};
use std::collections::HashMap;

use crate::model::{AppNotifyDataModel, AppNotifyDataStatus};

pub struct AppAppNotifyTaskItem(pub AppNotifyDataModel);
impl TaskItem<u64> for AppAppNotifyTaskItem {
    fn to_task_pk(&self) -> u64 {
        self.0.id
    }
}

pub struct AppAppNotifyTaskAcquisition {
    db: Pool<MySql>,
}
impl AppAppNotifyTaskAcquisition {
    pub fn new(db: Pool<MySql>) -> Self {
        Self { db }
    }
}
#[async_trait]
impl TaskAcquisition<u64, AppAppNotifyTaskItem> for AppAppNotifyTaskAcquisition {
    //复用父结构体方法实现
    async fn read_exec_task(
        &self,
        tasking_record: &HashMap<u64, TaskData>,
        limit: usize,
    ) -> Result<TaskRecord<u64, AppAppNotifyTaskItem>, String> {
        let ntime = now_time().unwrap_or_default();
        let init_status = AppNotifyDataStatus::Init as i8;

        let ids = tasking_record.keys().copied().collect::<Vec<u64>>();

        // Fetch send_data if there are active tasks
        let send_data = if !ids.is_empty() {
            let mut qb = QueryBuilder::<MySql>::new(format!(
                "select app_id,notify_method,notify_key from {}",
                AppNotifyDataModel::table_name(),
            ));
            qb.push_where().field_in_copied("id", &ids);
            qb.push(" group by app_id,notify_method,notify_key");
            qb.build_query_as::<(u64, String, String)>()
                .fetch_all(&self.db)
                .await
                .map_err(|e| e.to_string())?
        } else {
            vec![]
        };

        // Build main query using QueryBuilder
        let mut qb = QueryBuilder::<MySql>::new(format!(
            "select t1.* from {} as t1 INNER JOIN ( SELECT MIN(id) as min_id FROM {}",
            AppNotifyDataModel::table_name(),
            AppNotifyDataModel::table_name(),
        ));

        // t2 conditions (inner subquery)
        qb.push_where().field_lte("next_time", ntime);
        qb.push_and().field_eq("status", init_status);
        qb.push_and().push("try_num<=try_max");

        if !send_data.is_empty() {
            qb.push_and().push("NOT ((");
            for (i, e) in send_data.iter().enumerate() {
                if i > 0 {
                    qb.push(") or (");
                }
                qb.field_eq("app_id", e.0);
                qb.push_and().field_eq("notify_method", e.1.clone());
                qb.push_and().field_eq("notify_key", e.2.clone());
            }
            qb.push("))");
        }

        qb.push(" GROUP BY app_id,notify_method,notify_key ) as t2 ON t1.id = t2.min_id");

        // t1 conditions (outer query)
        qb.push_where().field_lte("t1.next_time", ntime);
        qb.push_and().field_eq("t1.status", init_status);
        qb.push_and().push("t1.try_num<t1.try_max");

        if !send_data.is_empty() {
            qb.push_and().push("NOT ((");
            for (i, e) in send_data.iter().enumerate() {
                if i > 0 {
                    qb.push(") or (");
                }
                qb.field_eq("t1.app_id", e.0);
                qb.push_and().field_eq("t1.notify_method", e.1.clone());
                qb.push_and().field_eq("t1.notify_key", e.2.clone());
            }
            qb.push("))");
        }

        qb.push(" ORDER BY t1.id ASC LIMIT ").push_bind((limit + 1) as i64);

        let mut app_res = qb
            .build_query_as::<AppNotifyDataModel>()
            .fetch_all(&self.db)
            .await
            .map_err(|e| e.to_string())?;
        let next = if app_res.len() > limit {
            app_res.pop();
            true
        } else {
            false
        };
        Ok(TaskRecord::new(
            app_res.into_iter().map(AppAppNotifyTaskItem).collect(),
            next,
        ))
    }
}
