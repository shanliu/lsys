use crate::dao::{SenderError, SenderResult};
use crate::model::{SenderConfigModel, SenderConfigModelRef, SenderConfigStatus, SenderType};
use lsys_core::now_time;

use sqlx::Pool;
use sqlx_model::{sql_format, Insert, Select, Update};

use sqlx_model::SqlQuote;

//短信任务记录
pub struct SenderConfig {
    db: Pool<sqlx::MySql>,
    send_type: SenderType,
}

impl SenderConfig {
    pub fn new(db: Pool<sqlx::MySql>, send_type: SenderType) -> Self {
        Self { db, send_type }
    }
    pub async fn find_by_id(&self, id: &u64) -> SenderResult<SenderConfigModel> {
        let data = sqlx_model::Select::type_new::<SenderConfigModel>()
            .fetch_one_by_where::<SenderConfigModel, _>(
                &sqlx_model::WhereOption::Where(sqlx_model::sql_format!(
                    "sender_type={} and id={}",
                    self.send_type,
                    id
                )),
                &self.db,
            )
            .await?;
        Ok(data)
    }
    pub async fn add(
        &self,
        app_id: Option<u64>,
        priority: i8,
        config_type: i8,
        config_data: String,
        user_id: u64,
        add_user_id: u64,
    ) -> SenderResult<u64> {
        let sender_type = self.send_type as i8;
        let app_id = app_id.unwrap_or_default();
        let time = now_time().unwrap_or_default();

        let add = sqlx_model::model_option_set!(SenderConfigModelRef, {
            app_id:app_id,
            sender_type:sender_type,
            priority:priority,
            config_type:config_type,
            user_id:user_id,
            add_user_id:add_user_id,
            add_time:time,
            status:SenderConfigStatus::Enable as i8,
            config_data:config_data,
        });
        Insert::<sqlx::MySql, SenderConfigModel, _>::new(add)
            .execute(&self.db)
            .await
            .map(|e| e.last_insert_id())
            .map_err(|e| e.into())
    }
    pub async fn del(&self, config: &SenderConfigModel, user_id: u64) -> SenderResult<u64> {
        let time = now_time().unwrap_or_default();
        let change = sqlx_model::model_option_set!(SenderConfigModelRef,{
            status:SenderConfigStatus::Delete as i8,
            delete_time:time,
            delete_user_id:user_id
        });
        let res = Update::<sqlx::MySql, SenderConfigModel, _>::new(change)
            .execute_by_pk(config, &self.db)
            .await;
        match res {
            Err(e) => Err(SenderError::Sqlx(e))?,
            Ok(mr) => {
                //清理缓存
                Ok(mr.rows_affected())
            }
        }
    }
    pub async fn list_data(
        &self,
        user_id: Option<u64>,
        id: Option<u64>,
        app_id: Option<u64>,
    ) -> SenderResult<Vec<SenderConfigModel>> {
        let sender_type = self.send_type as i8;
        let mut sqlwhere = vec![sql_format!(
            "sender_type={} and status ={}",
            sender_type,
            SenderConfigStatus::Enable
        )];
        if let Some(aid) = app_id {
            sqlwhere.push(sql_format!("app_id = {}  ", aid));
        }
        if let Some(uid) = id {
            sqlwhere.push(sql_format!("id={} ", uid));
        }
        if let Some(uid) = user_id {
            sqlwhere.push(sql_format!("user_id={} ", uid));
        }
        let sql = format!("{}  order by id desc", sqlwhere.join(" and "));
        Ok(Select::type_new::<SenderConfigModel>()
            .fetch_all_by_where::<SenderConfigModel, _>(
                &sqlx_model::WhereOption::Where(sql),
                &self.db,
            )
            .await?)
    }
}
