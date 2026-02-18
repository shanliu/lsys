use crate::dao::SenderResult;
use crate::model::{SenderLogModel, SenderLogStatus, SenderLogType, SenderType};
use lsys_core::db::OffsetPageParam;
use lsys_core::now_time;

use lsys_core::db::BatchInsert;
use lsys_core::db::{Insert, TableMeta, SqlExpr};
use lsys_core::sql_format;
use sqlx::Pool;

use lsys_core::db::SqlQuote;
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
        let mut batch = BatchInsert::<SenderLogModel>::with_capacity(tmp_dat.len());
        for (message_id, log_status, message) in tmp_dat.iter() {
            batch = batch.push(
                Insert::<SenderLogModel>::new()
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
        let sqlwhere = sql_format!(
            "sender_type={} and sender_message_id = {}  ",
            sender_type,
            message_id
        );
        let sql = sql_format!(
            "select count(*) as total from {} where {}",
            SenderLogModel::table_name(),
            SqlExpr(sqlwhere)
        );
        let query = sqlx::query_scalar::<_, i64>(&sql);
        let res = query.fetch_one(&self.db).await?;
        Ok(res)
    }
    pub async fn list_data(
        &self,
        message_id: u64,
        page: &OffsetPageParam,
    ) -> SenderResult<Vec<SenderLogModel>> {
        let sender_type = self.send_type as i8;
        let sql = sql_format!(
            "sender_type={} and sender_message_id = {} order by id desc {}",
            sender_type,
            message_id,
            page.page_query().limit_sql().unwrap_or_default()
        );
        let data = sqlx::query_as::<_, SenderLogModel>(&sql_format!(
            "select * from {} where {}",
            SenderLogModel::table_name(),
            SqlExpr(sql)
        ))
        .fetch_all(&self.db)
        .await?;
        Ok(data)
    }
}
