use lsys_core::{now_time, RequestEnv};
use sqlx::{MySql, Pool, Transaction};
use sqlx_model::{executor_option, model_option_set, Insert};
use tracing::{debug, warn};

use crate::model::{ChangeLogModel, ChangeLogModelRef};

pub trait ChangeLogData {
    fn log_type<'t>() -> &'t str;
    fn format(&self) -> String;
    fn encode(&self) -> String;
}

pub struct ChangeLogger {
    db: Pool<MySql>,
}

impl ChangeLogger {
    pub fn new(db: Pool<MySql>) -> Self {
        Self { db }
    }
    pub async fn add<'t, T: ChangeLogData>(
        &self,
        data: &T,
        source_id: &Option<u64>,
        user_id: &Option<u64>,
        add_user_id: &Option<u64>,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) {
        let user_id = user_id.unwrap_or_default();
        let add_user_id = add_user_id.unwrap_or_default();
        let source_id = source_id.unwrap_or_default();
        let log_data = data.encode();
        let log_type = T::log_type().to_string();
        let time = env_data
            .map(|e| e.request_time)
            .unwrap_or_else(|| now_time().unwrap_or_default());
        let user_ip = env_data
            .map(|e| {
                e.request_ip
                    .as_ref()
                    .map(|e| e.to_string())
                    .unwrap_or_default()
            })
            .unwrap_or_default()
            .chars()
            .take(40)
            .collect();
        let request_id = env_data
            .as_ref()
            .map(|e| {
                e.request_id
                    .as_ref()
                    .map(|e| e.to_string())
                    .unwrap_or_default()
            })
            .unwrap_or_default()
            .chars()
            .take(32)
            .collect();
        let request_user_agent = env_data
            .as_ref()
            .map(|e| {
                e.request_user_agent
                    .as_ref()
                    .map(|e| e.to_string())
                    .unwrap_or_default()
            })
            .unwrap_or_default()
            .chars()
            .take(254)
            .collect();

        let new_data = model_option_set!(ChangeLogModelRef, {
            log_type: log_type,
            log_data:log_data,
            user_id:user_id,
            source_id:source_id,
            add_user_id:add_user_id,
            user_ip:user_ip,
            request_id:request_id,
            add_time:time,
            request_user_agent:request_user_agent,
        });
        let res = executor_option!(
            {
                Insert::<sqlx::MySql, ChangeLogModel, _>::new(new_data)
                    .execute(db)
                    .await
            },
            transaction,
            &self.db,
            db
        );
        match res {
            Err(err) => warn!("add log fail:{}", err),
            Ok(r) => debug!("add log id:{}", r.last_insert_id()),
        };
    }
}
