use crate::dao::SenderResult;
use crate::model::{SenderLogModel, SenderLogModelRef, SenderLogStatus, SenderLogType, SenderType};
use lsys_core::{now_time, PageParam};

use lsys_core::db::{Insert, ModelTableName, SqlExpr};
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
    pub async fn add_exec_log(
        &self,
        app_id: &u64,
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
        let mut idata = Vec::with_capacity(log_data.len());
        let tmp_dat = log_data
            .iter()
            .map(|(a, b, c)| (*a, (*b as i8), c.to_string()))
            .collect::<Vec<(u64, i8, String)>>();
        for (message_id, log_status, message) in tmp_dat.iter() {
            idata.push(lsys_core::model_option_set!(SenderLogModelRef,{
                sender_message_id:message_id,
                app_id:app_id,
                sender_type:sender_type,
                log_type:log_type,
                status:log_status,
                executor_type:executor_type,
                message:message,
                create_time:send_time,
            }));
        }
        let tmp = Insert::<SenderLogModel, _>::new_vec(idata)
            .execute(&self.db)
            .await;
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
        page: Option<&PageParam>,
    ) -> SenderResult<Vec<SenderLogModel>> {
        let sender_type = self.send_type as i8;
        let mut sql = sql_format!(
            "sender_type={} and sender_message_id = {} order by id desc ",
            sender_type,
            message_id
        );
        if let Some(pdat) = page {
            sql += format!(" limit {} offset {}", pdat.limit, pdat.offset).as_str();
        }
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
