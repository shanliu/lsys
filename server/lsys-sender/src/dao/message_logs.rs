use crate::dao::SenderResult;
use crate::model::{SenderLogModel, SenderLogModelRef, SenderLogStatus, SenderLogType, SenderType};
use lsys_core::{now_time, PageParam};

use sqlx::Pool;
use sqlx_model::{sql_format, Insert, ModelTableName, Select, SqlExpr};

use sqlx_model::SqlQuote;
use tracing::warn;

//短信任务记录
pub struct MessageLogs {
    db: Pool<sqlx::MySql>,
    send_type: SenderType,
}

impl MessageLogs {
    pub fn new(db: Pool<sqlx::MySql>, send_type: SenderType) -> Self {
        Self { db, send_type }
    }
    pub async fn add_finish_log(
        &self,
        event_type: String,
        app_id: u64,
        message_id: u64,
        channel: String,
        res: &Result<(), String>,
    ) {
        let (log_status, err_msg) = match res {
            Ok(()) => (SenderLogStatus::Succ as i8, "".to_string()),
            Err(err) => (SenderLogStatus::Fail as i8, err.to_owned()),
        };
        let send_time = now_time().unwrap_or_default();
        let log_type = SenderLogType::Send as i8;
        let idata = sqlx_model::model_option_set!(SenderLogModelRef,{
            message_id:message_id,
            app_id:app_id,
            log_type:log_type,
            status:log_status,
            event_type:event_type,
            send_channel:channel,
            message:err_msg,
            create_time:send_time,
            user_id:0,
        });
        let tmp = Insert::<sqlx::MySql, SenderLogModel, _>::new(idata)
            .execute(&self.db)
            .await;
        if let Err(ie) = tmp {
            warn!("sms[{}] is send ,add history fail : {:?}", message_id, ie);
        }
    }
    pub async fn add_cancel_log(&self, app_id: u64, message_id: u64, user_id: &u64) {
        let send_time = now_time().unwrap_or_default();
        let log_type = SenderLogType::Cancel as i8;
        let event_type = "cancel".to_string();
        let sender_type = self.send_type as i8;
        let log = "cancal send".to_string();
        let idata = sqlx_model::model_option_set!(SenderLogModelRef,{
            app_id:app_id,
            message_id:message_id,
            sender_type:sender_type,
            log_type:log_type,
            status: SenderLogStatus::MessageCancel as i8,
            event_type:event_type,
            message:log,
            create_time:send_time,
            user_id:*user_id,
        });
        let tmp = Insert::<sqlx::MySql, SenderLogModel, _>::new(idata)
            .execute(&self.db)
            .await;
        if let Err(ie) = tmp {
            warn!("sms[{}] is cancel ,add history fail : {:?}", message_id, ie);
        }
    }
    pub async fn list_count(&self, message_id: &u64) -> SenderResult<i64> {
        let sender_type = self.send_type as i8;
        let sqlwhere = vec![sql_format!(
            "sender_type={} and message_id = {}  ",
            sender_type,
            message_id
        )];
        let sql = sql_format!(
            "select count(*) as total from {} where {}",
            SenderLogModel::table_name(),
            SqlExpr(sqlwhere.join(" and "))
        );
        let query = sqlx::query_scalar::<_, i64>(&sql);
        let res = query.fetch_one(&self.db).await?;
        Ok(res)
    }
    pub async fn list_data(
        &self,
        message_id: &u64,
        page: &Option<PageParam>,
    ) -> SenderResult<Vec<SenderLogModel>> {
        let sender_type = self.send_type as i8;
        let sqlwhere = vec![sql_format!(
            "sender_type={} and message_id = {}  ",
            sender_type,
            message_id
        )];
        let mut sql = format!("{}  order by id desc", sqlwhere.join(" and "));
        if let Some(pdat) = page {
            sql += format!(" limit {} offset {}", pdat.limit, pdat.offset).as_str();
        }
        let data = Select::type_new::<SenderLogModel>()
            .fetch_all_by_where::<SenderLogModel, _>(&sqlx_model::WhereOption::Where(sql), &self.db)
            .await?;
        Ok(data)
    }
}
