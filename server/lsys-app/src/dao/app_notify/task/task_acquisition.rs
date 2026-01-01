use async_trait::async_trait;
use lsys_core::db::{ModelTableName, SqlExpr};
use lsys_core::sql_format;
use lsys_core::{now_time, TaskAcquisition, TaskData, TaskItem, TaskRecord};
use sqlx::{MySql, Pool};
use std::collections::HashMap;

use crate::model::{AppNotifyDataModel, AppNotifyDataStatus};

use lsys_core::db::SqlQuote;

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
        let mut t1_sql_vec = vec![sql_format!(
            "t1.next_time<={} and t1.status = {} and t1.try_num<t1.try_max ",
            ntime,
            AppNotifyDataStatus::Init as i8
        )];
        let mut t2_sql_vec = vec![sql_format!(
            "next_time<={} and status = {} and try_num<=try_max ",
            ntime,
            AppNotifyDataStatus::Init as i8
        )];
        let ids = tasking_record.keys().copied().collect::<Vec<u64>>();
        if !ids.is_empty() {
            let send_data = sqlx::query_as::<_, (u64, String, String)>(&sql_format!(
                "select app_id,notify_method,notify_key from {} 
                 where id in ({}) group by app_id,notify_method,notify_key",
                AppNotifyDataModel::table_name(),
                ids
            ))
            .fetch_all(&self.db)
            .await
            .map_err(|e| e.to_string())?;
            if !send_data.is_empty() {
                t1_sql_vec.push(sql_format!(
                    " NOT (({}))",
                    SqlExpr(
                        send_data
                            .iter()
                            .map(|e| {
                                sql_format!(
                                    "t1.app_id={} and t1.notify_method ={} and t1.notify_key={}",
                                    e.0,
                                    e.1,
                                    e.2
                                )
                            })
                            .collect::<Vec<String>>()
                            .join(") or (")
                    )
                ));
                t2_sql_vec.push(sql_format!(
                    " NOT (({}))",
                    SqlExpr(
                        send_data
                            .iter()
                            .map(|e| {
                                sql_format!(
                                    "app_id={} and notify_method ={} and notify_key={}",
                                    e.0,
                                    e.1,
                                    e.2
                                )
                            })
                            .collect::<Vec<String>>()
                            .join(") or (")
                    )
                ));
            }
        }
        let mut app_res = sqlx::query_as::<_, AppNotifyDataModel>(&format!(
            "select t1.* from {} as t1
            INNER JOIN (
                SELECT MIN(id) as min_id
                FROM {}
                WHERE {}
                GROUP BY app_id,notify_method,notify_key
            ) as t2 ON t1.id = t2.min_id
                 where {} order by t1.id asc limit {}",
            AppNotifyDataModel::table_name(),
            AppNotifyDataModel::table_name(),
            t2_sql_vec.join(" and "),
            t1_sql_vec.join(" and "),
            limit + 1
        ))
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
