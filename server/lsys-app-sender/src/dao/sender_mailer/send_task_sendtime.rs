use crate::model::SenderMailBodyModel;
use crate::model::SenderMailBodyStatus;
use lsys_core::db::ModelTableName;
use lsys_core::db::SqlQuote;
use lsys_core::TaskNotify;
use lsys_core::{now_time, sql_format};
use lsys_core::{TimeOutTaskExec, TimeOutTaskExecutor, TimeOutTaskNextTime};
use redis::AsyncCommands;
use sqlx::MySql;
use sqlx::Pool;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tracing::debug;
use tracing::error;
use tracing::warn;

pub struct MailTaskSendTimeNotify {
    last_run_time_key: String,
    redis: deadpool_redis::Pool,
    db: Pool<MySql>,
    task_notify: Arc<TaskNotify>,
}

impl MailTaskSendTimeNotify {
    pub fn new(
        last_run_time_key: impl ToString,
        db: Pool<MySql>,
        redis: deadpool_redis::Pool,
        task_notify: Arc<TaskNotify>,
    ) -> Self {
        Self {
            db,
            task_notify,
            redis,
            last_run_time_key: last_run_time_key.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl TimeOutTaskExec for MailTaskSendTimeNotify {
    async fn exec(
        &self,
        max_lock_time: usize,
        mut _expire_call: impl FnMut() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send,
    ) -> Result<(), String> {
        if let Err(err) = self.task_notify.notify().await {
            warn!("mail exec notify fail:{:?}", err);
        } else {
            let mut redis = self.redis.get().await.map_err(|e| e.to_string())?;
            if let Err(err) = redis
                .set_ex::<_, _, ()>(
                    self.last_run_time_key.as_str(),
                    now_time().unwrap_or_default(),
                    max_lock_time as u64,
                )
                .await
            {
                error!("save mail last runtime fail fail:{:?}", err);
            }
        }
        Ok(())
    }
}
#[async_trait::async_trait]
impl TimeOutTaskNextTime for MailTaskSendTimeNotify {
    async fn next_time(&self, max_lock_time: usize) -> Result<Option<u64>, String> {
        let ntime = now_time().unwrap_or_default();

        let mut redis = self.redis.get().await.map_err(|e| e.to_string())?;
        let last_time = match redis
            .get::<_, Option<String>>(self.last_run_time_key.as_str())
            .await
        {
            Ok(t) => t.and_then(|s| s.parse::<u64>().ok()),
            Err(err) => {
                error!("get mail last runtime fail fail:{:?}", err);
                None
            }
        };
        let ctime = last_time
            .map(|s| {
                if s < ntime {
                    ntime
                } else if s > ntime {
                    s
                } else {
                    s + 1
                }
            })
            .unwrap_or(ntime);
        debug!("sms check sendtime start time is:{}", ctime);
        let timeout_res = sqlx::query_scalar::<_, u64>(&sql_format!(
            "select expected_time from  {}  where 
                status={} and expected_time >={} and expected_time <{} order by expected_time asc limit 1",
            SenderMailBodyModel::table_name(),
            SenderMailBodyStatus::Init as i8,
            ctime,
            (ctime + max_lock_time as u64)
        ))
        .fetch_one(&self.db)
        .await;
        match timeout_res {
            Ok(dat) => Ok(Some(dat)),
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(err) => Err(err.to_string()),
        }
    }
}
#[async_trait::async_trait]
impl TimeOutTaskExecutor for MailTaskSendTimeNotify {
    type Exec = Self;
    type NextTime = Self;
}
