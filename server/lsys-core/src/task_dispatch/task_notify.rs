//要使用定时任务时,时间到后定时触发任务派发
use redis::{AsyncCommands, RedisError, aio::MultiplexedConnection};
use tracing::{debug, info};

use std::sync::Arc;

use crate::app_core::{AppCore, AppCoreError};
use crate::timeout_task::{TimeOutTask, TimeOutTaskExecutor, TimeOutTaskNotify};
use crate::utils::now_time;

use super::TaskDispatchConfig;
pub struct TaskNotifyConfig {
    pub task_name: String,   //执行任务名
    list_notify_key: String, //任务触发监听的REDIS KEY
}

impl TaskNotifyConfig {
    pub fn new(task_name: impl ToString) -> Self {
        let task_name = task_name.to_string();
        let list_notify_key = format!("td-{}-list-notify", task_name);
        Self {
            task_name,
            list_notify_key,
        }
    }
    pub fn list_notify_key(&self) -> &str {
        &self.list_notify_key
    }
}

// 任务派发执行抽象实现
pub struct TaskNotify {
    redis: deadpool_redis::Pool,
    config: Arc<TaskNotifyConfig>,
}

impl TaskNotify {
    pub fn new(redis: deadpool_redis::Pool, config: Arc<TaskNotifyConfig>) -> Self {
        TaskNotify { redis, config }
    }
    pub async fn notify(&self) -> Result<(), AppCoreError> {
        debug!("notify send :{}", self.config.list_notify_key());
        let mut redis = self.redis.get().await?;
        self._notify(&mut redis).await?;
        Ok(())
    }
    pub(super) async fn _notify(
        &self,
        redis: &mut MultiplexedConnection,
    ) -> Result<(), RedisError> {
        if let Ok(len) = redis.llen::<&str, i64>(self.config.list_notify_key()).await
            && len > 1
        {
            debug!(
                "notify skipped, queue already has {} pending signals: {}",
                len,
                self.config.list_notify_key()
            );
            return Ok(());
        }
        info!("notify: sending dispatch signal to [{}]", self.config.list_notify_key());
        redis.lpush(self.config.list_notify_key(), 1).await
    }
}

pub struct TaskTimeOutNotify {
    app_core: Arc<AppCore>,
    notify: Arc<TaskNotify>,
    display_config: Arc<TaskDispatchConfig>,
    time_out_notify: Arc<TimeOutTaskNotify>,
}

impl TaskTimeOutNotify {
    pub fn new(
        app_core: Arc<AppCore>,
        notify: Arc<TaskNotify>,
        time_out_notify: Arc<TimeOutTaskNotify>,
        display_config: Arc<TaskDispatchConfig>,
    ) -> Self {
        Self {
            app_core,
            notify,
            time_out_notify,
            display_config,
        }
    }
    /// 在某个时间完成通知任务派发
    /// run_time 时间戳
    pub async fn notify_at_time(&self, run_time: u64) -> Result<bool, AppCoreError> {
        if run_time == 0 || run_time <= (now_time().unwrap_or_default() + 1) {
            self.notify.notify().await.map(|_| true)
        } else if run_time
            < (now_time().unwrap_or_default() + self.display_config.task_timeout as u64)
        {
            debug!(
                "notify at time {} :{} ",
                run_time,
                self.display_config.task_list_key()
            );
            self.time_out_notify
                .notify_timeout(
                    (now_time().unwrap_or_default() + self.display_config.task_timeout as u64)
                        - run_time,
                )
                .await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    /// 通知执行模块进行任务执行操作
    /// * `redis` - 存放任务的RDIS
    pub async fn listen_timeout<E: TimeOutTaskExecutor>(
        &self,
        exec: Arc<E::Exec>,
        task_next_time: Arc<E::NextTime>,
        channel_buffer: Option<usize>,
        cancel_token: tokio_util::sync::CancellationToken,
    ) -> Result<(), AppCoreError> {
        let sub_app_timeout_task = TimeOutTask::<E>::new(
            self.app_core.clone(),
            self.time_out_notify.clone(),
            exec,
            task_next_time,
        );
        sub_app_timeout_task.listen(channel_buffer, cancel_token).await;
        Ok(())
    }
}
