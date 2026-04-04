use std::sync::Arc;

use crate::dao::{SenderError, SenderResult};
use crate::model::{SenderConfigModel, SenderConfigStatus, SenderType};
use lsys_core::utils::{now_time, RequestEnv};

use lsys_core::db::{Insert, QueryBuilderExt, TableMeta, Update};
use lsys_logger::dao::ChangeLoggerDao;
use sqlx::{MySql, Pool};

use super::logger::LogSenderConfig;

//发送限制等配置

//发送配置
pub struct SenderConfig {
    db: Pool<sqlx::MySql>,
    send_type: SenderType,
    logger: Arc<ChangeLoggerDao>,
}

impl SenderConfig {
    pub fn new(db: Pool<sqlx::MySql>, logger: Arc<ChangeLoggerDao>, send_type: SenderType) -> Self {
        Self {
            db,
            send_type,
            logger,
        }
    }
    pub async fn find_by_id(&self, id: u64) -> SenderResult<SenderConfigModel> {
        let mut qb = sqlx::QueryBuilder::<MySql>::new(format!(
            "select * from {}",
            SenderConfigModel::table_name(),
        ));
        qb.push_where().field_eq("sender_type", self.send_type as i8);
        qb.push_and().field_eq("id", id);
        let data = qb.build_query_as::<SenderConfigModel>()
            .fetch_one(&self.db)
            .await?;

        Ok(data)
    }
    #[allow(clippy::too_many_arguments)]
    pub async fn add(
        &self,
        app_id: Option<u64>,
        priority: i8,
        config_type: i8,
        config_data: &str,
        user_id: u64,
        add_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        let sender_type = self.send_type as i8;
        let app_id = app_id.unwrap_or_default();
        let time = now_time().unwrap_or_default();
        let config_data = config_data.to_string();
        let id = Insert::<_,SenderConfigModel>::new()
            .set(SenderConfigModel::APP_ID, app_id)
            .set(SenderConfigModel::SENDER_TYPE, sender_type)
            .set(SenderConfigModel::PRIORITY, priority)
            .set(SenderConfigModel::CONFIG_TYPE, config_type)
            .set(SenderConfigModel::USER_ID, user_id)
            .set(SenderConfigModel::CHANGE_USER_ID, add_user_id)
            .set(SenderConfigModel::CHANGE_TIME, time)
            .set(SenderConfigModel::STATUS, SenderConfigStatus::Enable as i8)
            .set(SenderConfigModel::CONFIG_DATA, &config_data)
            .execute(&self.db)
            .await
            .map(|e| e.last_insert_id())?;

        self.logger
            .add(
                &LogSenderConfig {
                    action: "add",
                    app_id,
                    user_id,
                    priority,
                    sender_type,
                    config_type,
                    config_data: &config_data,
                },
                Some(id),
                Some(add_user_id),
                None,
                env_data,
            )
            .await;

        Ok(id)
    }
    pub async fn del(
        &self,
        config: &SenderConfigModel,
        user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        if SenderConfigStatus::Delete.eq(config.status) {
            return Ok(0);
        }
        let time = now_time().unwrap_or_default();
        let res = Update::<_,SenderConfigModel>::new()
            .set(SenderConfigModel::STATUS, SenderConfigStatus::Delete as i8)
            .set(SenderConfigModel::CHANGE_TIME, time)
            .set(SenderConfigModel::CHANGE_USER_ID, user_id)
            .execute(&self.db, |qb| {
                qb.push_where().field_eq("id", config.id);
            })
            .await;
        match res {
            Err(e) => Err(SenderError::Sqlx(e))?,
            Ok(mr) => {
                self.logger
                    .add(
                        &LogSenderConfig {
                            action: "del",
                            app_id: config.app_id,
                            priority: config.priority,
                            user_id: config.user_id,
                            sender_type: config.sender_type,
                            config_type: config.config_type,
                            config_data: &config.config_data,
                        },
                        Some(config.id),
                        Some(user_id),
                        None,
                        env_data,
                    )
                    .await;

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
        let mut qb = sqlx::QueryBuilder::<MySql>::new(format!(
            "select * from {}",
            SenderConfigModel::table_name(),
        ));
        qb.push_where().field_eq("sender_type", sender_type);
        qb.push_and().field_eq("status", SenderConfigStatus::Enable as i8);
        if let Some(aid) = app_id {
            qb.push_and().field_eq("app_id", aid);
        }
        if let Some(uid) = id {
            qb.push_and().field_eq("id", uid);
        }
        if let Some(uid) = user_id {
            qb.push_and().field_eq("user_id", uid);
        }
        qb.push(" order by id desc");
        Ok(qb.build_query_as::<SenderConfigModel>()
            .fetch_all(&self.db)
            .await?)
    }
}


