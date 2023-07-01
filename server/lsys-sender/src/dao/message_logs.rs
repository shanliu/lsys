use crate::dao::SenderResult;
use crate::model::{SenderLogModel, SenderLogModelRef, SenderLogStatus, SenderLogType, SenderType};
use lsys_core::{now_time, PageParam, RequestEnv};

use sqlx::Pool;
use sqlx_model::{sql_format, Insert, ModelTableName, Select, SqlExpr};

use sqlx_model::SqlQuote;
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
    pub async fn add_init_log(
        &self,
        app_id: &u64,
        message_ids: &[u64],
        env_data: Option<&RequestEnv>,
    ) {
        if message_ids.is_empty() {
            return;
        }
        let send_time = now_time().unwrap_or_default();
        let log_type = SenderLogType::Init as i8;
        let sender_type = self.send_type as i8;
        let status = SenderLogStatus::Succ as i8;
        let mut idata = Vec::with_capacity(message_ids.len());
        let tmp = "".to_string();
        let user_ip = env_data
            .as_ref()
            .map(|e| {
                e.request_ip
                    .as_ref()
                    .map(|e| e.to_owned())
                    .unwrap_or("".to_string())
            })
            .unwrap_or("".to_string());
        let request_id = env_data
            .as_ref()
            .map(|e| {
                e.request_id
                    .as_ref()
                    .map(|e| e.to_owned())
                    .unwrap_or("".to_string())
            })
            .unwrap_or("".to_string());
        for id in message_ids {
            idata.push(sqlx_model::model_option_set!(SenderLogModelRef, {
                message_id:id,
                app_id:app_id,
                sender_type:sender_type,
                log_type:log_type,
                status:status ,
                send_note:tmp,
                message:tmp,
                create_time:send_time,
                user_id:0,
                user_ip:user_ip,
                request_id:request_id
            }));
        }
        let tmp = Insert::<sqlx::MySql, SenderLogModel, _>::new_vec(idata)
            .execute(&self.db)
            .await;
        if let Err(ie) = tmp {
            warn!("sms add log fail {} :{}", app_id, ie);
        }
    }
    pub async fn add_finish_log(
        &self,
        app_id: u64,
        message_id: u64,
        status: &SenderLogStatus,
        send_note: &str,
        message: &str,
    ) {
        let send_time = now_time().unwrap_or_default();
        let log_type = SenderLogType::Send as i8;
        let sender_type = self.send_type as i8;
        let log_status = *status as i8;
        let send_note = send_note.to_owned();
        let message = message.to_owned();
        let idata: SenderLogModelRef<'_> = sqlx_model::model_option_set!(SenderLogModelRef,{
            message_id:message_id,
            app_id:app_id,
            sender_type:sender_type,
            log_type:log_type,
            status:log_status,
            send_note:send_note,
            message:message,
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
    pub async fn add_cancel_log(
        &self,
        app_id: u64,
        message_id: u64,
        user_id: &u64,
        env_data: Option<&RequestEnv>,
    ) {
        let send_time = now_time().unwrap_or_default();
        let log_type = SenderLogType::Cancel as i8;
        let sender_type = self.send_type as i8;
        let send_note = "".to_owned();
        let log = "cancal send".to_string();
        let user_ip = env_data
            .as_ref()
            .map(|e| {
                e.request_ip
                    .as_ref()
                    .map(|e| e.to_owned())
                    .unwrap_or("".to_string())
            })
            .unwrap_or("".to_string());
        let request_id = env_data
            .as_ref()
            .map(|e| {
                e.request_id
                    .as_ref()
                    .map(|e| e.to_owned())
                    .unwrap_or("".to_string())
            })
            .unwrap_or("".to_string());
        let idata = sqlx_model::model_option_set!(SenderLogModelRef,{
            app_id:app_id,
            message_id:message_id,
            sender_type:sender_type,
            log_type:log_type,
            status: SenderLogStatus::MessageCancel as i8,
            send_note:send_note,
            message:log,
            create_time:send_time,
            user_id:*user_id,
            user_ip:user_ip,
            request_id:request_id
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
