use std::{
    // error::Error,
    sync::Arc,
};

use lsys_app::dao::App;
use lsys_core::{AppCore, TaskDispatch};
use lsys_logger::dao::ChangeLoggerDao;
use sqlx::Pool;
use tracing::warn;

use super::{
    NotifyRecord, NotifyResult, NotifyTask, NotifyTaskAcquisition, NotifyTaskItem,
    NOTIFY_MIN_DELAY_TIME,
};

const NOTIFY_REDIS_PREFIX: &str = "notify-task";
pub trait NotifyData {
    fn to_string(&self) -> String;
    fn method() -> String;
    fn app_id(&self) -> u64;
}
pub struct NotifyDao {
    app_core: Arc<AppCore>,
    db: Pool<sqlx::MySql>,
    app: Arc<App>,
    pub record: Arc<NotifyRecord>,
    task: TaskDispatch<u64, NotifyTaskItem>,
    redis: deadpool_redis::Pool,
    max_try: u16,
}

impl NotifyDao {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        redis: deadpool_redis::Pool,
        db: Pool<sqlx::MySql>,
        app_core: Arc<AppCore>,
        app: Arc<App>,
        logger: Arc<ChangeLoggerDao>,
        max_try: Option<u16>,
        task_size: Option<usize>,
        task_timeout: Option<usize>,
        is_check: bool,
    ) -> Self {
        let record = Arc::new(NotifyRecord::new(db.clone(), logger));

        let task_timeout = match task_timeout {
            Some(t) => {
                if t == 0 {
                    NOTIFY_MIN_DELAY_TIME as usize
                } else if (t as u64) < NOTIFY_MIN_DELAY_TIME {
                    t
                } else {
                    NOTIFY_MIN_DELAY_TIME as usize
                }
            }
            None => NOTIFY_MIN_DELAY_TIME as usize,
        };

        let task = TaskDispatch::new(
            &format!("{}-notify", NOTIFY_REDIS_PREFIX),
            &format!("{}-notify-read-lock", NOTIFY_REDIS_PREFIX),
            &format!("{}-notify-run-task", NOTIFY_REDIS_PREFIX),
            task_size,
            task_timeout,
            is_check,
            task_timeout,
        );
        Self {
            app_core,
            db,
            record,
            task,
            redis,
            app,
            max_try: max_try.unwrap_or(5),
        }
    }
    pub async fn add_data<T: NotifyData>(&self, data: T) -> NotifyResult<u64> {
        self.add(&T::method(), data.app_id(), &data.to_string())
            .await
    }
    pub async fn add(&self, method: &str, app_id: u64, data: &str) -> NotifyResult<u64> {
        let id = self.record.add(method, app_id, data).await?;
        let mut redis = self.redis.get().await?;
        if let Err(err) = self.task.notify(&mut redis).await {
            warn!("add notify task fail :{}", err)
        }
        Ok(id)
    }

    //后台发送任务，内部循环不退出
    pub async fn task(&self) -> NotifyResult<()> {
        let acquisition = NotifyTaskAcquisition::new(self.db.clone());
        self.task
            .dispatch(
                self.app_core.clone(),
                &acquisition,
                NotifyTask::new(
                    self.db.clone(),
                    self.app.clone(),
                    self.record.clone(),
                    self.max_try,
                ),
            )
            .await;
        Ok(())
    }
}
