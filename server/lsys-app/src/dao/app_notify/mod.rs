mod record;
mod sender;
mod task;

pub use record::*;
pub use sender::*;
pub use task::*;

use std::{sync::Arc, time::Duration};

use lsys_core::{
    fluent_message, AppCore, RequestEnv, TaskDispatch, TaskDispatchConfig, TaskNotify,
    TaskNotifyConfig,
};
use lsys_logger::dao::ChangeLoggerDao;
use sqlx::Pool;

use crate::{
    dao::{AppError, AppResult, AppSecret},
    model::{AppNotifyDataModel, AppNotifyTryTimeMode, AppNotifyType},
};

use super::App;

const NOTIFY_REDIS_PREFIX: &str = "notify-task";

pub struct AppNotify {
    db: Pool<sqlx::MySql>,
    pub record: Arc<AppNotifyRecord>,
    task: TaskDispatch<u64, AppAppNotifyTaskItem>,
    task_notify: Arc<TaskNotify>,
    app_secret: Arc<AppSecret>,
}

pub struct NotifyConfig {
    pub task_size: Option<usize>,    //最大同时回调任务数量
    pub task_timeout: Option<usize>, //任务执行超时
}

impl AppNotify {
    pub fn new(
        redis: deadpool_redis::Pool,
        db: Pool<sqlx::MySql>,
        config: &NotifyConfig,
        logger: Arc<ChangeLoggerDao>,
        app_secret: Arc<AppSecret>,
    ) -> Self {
        let record = Arc::new(AppNotifyRecord::new(db.clone(), logger));

        let task_timeout = match config.task_timeout {
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
        let task_notify_config = Arc::new(TaskNotifyConfig::new(NOTIFY_REDIS_PREFIX));
        let task_notify = Arc::new(TaskNotify::new(redis.clone(), task_notify_config.clone()));
        let display_config = Arc::new(TaskDispatchConfig::new(
            task_notify_config,
            task_timeout,
            true,
            config.task_size,
        ));
        let task = TaskDispatch::new(redis, task_notify.clone(), display_config);
        Self {
            db,
            record,
            task,
            task_notify,
            app_secret,
        }
    }
    pub fn sender_create(
        &self,
        notify_method: impl ToString,
        notify_type: AppNotifyType,
        try_max: u8,
        try_mode: AppNotifyTryTimeMode,
        try_delay: u16,
        clear_init_status: bool,
    ) -> AppNotifySender {
        AppNotifySender::new(
            self.db.clone(),
            self.record.clone(),
            self.task_notify.clone(),
            notify_method,
            notify_type,
            try_max,
            try_mode,
            try_delay,
            clear_init_status,
        )
    }
    //删除回调
    pub async fn remove_notify(
        &self,
        data: &AppNotifyDataModel,
        del_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> AppResult<()> {
        if self.task.task_data().await?.iter().any(|e| *e.0 == data.id) {
            return Err(AppError::System(fluent_message!(
                "del-notify-data-bad-status"
            )));
        }
        self.record.del(data, del_user_id, env_data).await
    }
    //后台发送任务，内部循环不退出
    pub async fn task(&self, app_core: Arc<AppCore>, app: Arc<App>) -> AppResult<()> {
        let acquisition = AppAppNotifyTaskAcquisition::new(self.db.clone());
        self.task
            .dispatch(
                app_core,
                &acquisition,
                Arc::new(AppNotifyTask::new(
                    self.db.clone(),
                    app,
                    self.app_secret.clone(),
                    self.record.clone(),
                    self.task_notify.clone(),
                    vec![(
                        AppNotifyType::Http,
                        Box::new(AppNotifyRequestReqwest::new(Duration::from_secs(5))?),
                    )],
                )),
            )
            .await;
        Ok(())
    }
}
