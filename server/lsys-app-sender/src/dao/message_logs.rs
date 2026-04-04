use crate::dao::SenderResult;
use crate::model::{SenderLogModel, SenderLogStatus, SenderLogType, SenderType};
use lsys_core::db::OffsetPageParam;
use lsys_core::utils::now_time;

use lsys_core::db::BatchInsert;
use lsys_core::db::{Insert, QueryBuilderExt, TableMeta};
use sqlx::{MySql, Pool, QueryBuilder};
use tracing::warn;

//发送任务日志相关操作实现
pub struct MessageLogs {
    db: Pool<sqlx::MySql>,
    send_type: SenderType,
}

impl MessageLogs {
    pub fn new(db: Pool<sqlx::MySql>, send_type: SenderType) -> Self {
        Self { db, send_type }
    }
    pub(crate) async fn add_exec_log(
        &self,
        app_id: u64,
        log_data: &[(u64, SenderLogStatus, &str)],
        executor_type: &str,
    ) {
        if log_data.is_empty() {
            return;
        }
        let send_time = now_time().unwrap_or_default();
        let app_id = app_id.to_owned();
        let log_type = SenderLogType::Send as i8;
        let sender_type = self.send_type as i8;
        let executor_type = executor_type.to_owned();
        let tmp_dat = log_data
            .iter()
            .map(|(a, b, c)| (*a, (*b as i8), c.to_string()))
            .collect::<Vec<(u64, i8, String)>>();
        let mut batch = BatchInsert::<_,SenderLogModel>::with_capacity(tmp_dat.len());
        for (message_id, log_status, message) in tmp_dat.iter() {
            batch = batch.push(
                Insert::<_,SenderLogModel>::new()
                    .set(SenderLogModel::SENDER_MESSAGE_ID, *message_id)
                    .set(SenderLogModel::APP_ID, app_id)
                    .set(SenderLogModel::SENDER_TYPE, sender_type)
                    .set(SenderLogModel::LOG_TYPE, log_type)
                    .set(SenderLogModel::STATUS, *log_status)
                    .set(SenderLogModel::EXECUTOR_TYPE, &executor_type)
                    .set(SenderLogModel::MESSAGE, message)
                    .set(SenderLogModel::CREATE_TIME, send_time),
            );
        }
        let tmp = batch.execute(&self.db).await;
        if let Err(ie) = tmp {
            warn!(
                "sms[{}:{}] is send ,add history fail : {:?}",
                app_id, executor_type, ie
            );
        }
    }
    pub async fn list_count(&self, message_id: u64) -> SenderResult<i64> {
        let sender_type = self.send_type as i8;
        let sql = format!(
            "select count(*) as total from {} where sender_type=? and sender_message_id = ?",
            SenderLogModel::table_name()
        );
        let res = sqlx::query_scalar::<_, i64>(&sql)
            .bind(sender_type)
            .bind(message_id)
            .fetch_one(&self.db)
            .await?;
        Ok(res)
    }
    pub async fn list_data(
        &self,
        message_id: u64,
        page: &OffsetPageParam,
    ) -> SenderResult<Vec<SenderLogModel>> {
        let sender_type = self.send_type as i8;
        let mut qb = QueryBuilder::<MySql>::new(format!(
            "select * from {}",
            SenderLogModel::table_name(),
        ));
        qb.push_where().field_eq("sender_type", sender_type);
        qb.push_and().field_eq("sender_message_id", message_id);
        qb.push(" order by id desc");
        page.push_limit(&mut qb);
        let data = qb.build_query_as::<SenderLogModel>()
            .fetch_all(&self.db)
            .await?;
        Ok(data)
    }
}
