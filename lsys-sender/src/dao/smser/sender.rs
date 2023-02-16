use std::{collections::HashSet, sync::Arc};

use lsys_core::{now_time, AppCore};

use tracing::warn;

use crate::dao::task::TaskExecutioner;

use super::{super::task::Task, SmsTaskAcquisition, SmsTaskItem, SmserTask, SmserTaskExecutioner};

const SMSER_REDIS_PREFIX: &str = "sender-sms-";

pub struct SmsSender<A: SmsTaskAcquisition<T>, T: Send + Sync + 'static + Clone> {
    app_core: Arc<AppCore>,
    redis: deadpool_redis::Pool,
    task: Task<u64, SmsTaskItem<T>>,
    acquisition: A,
}

impl<A: SmsTaskAcquisition<T>, T: Send + Sync + 'static + Clone> SmsSender<A, T> {
    //短信发送
    pub fn new(
        app_core: Arc<AppCore>,
        redis: deadpool_redis::Pool,
        task_size: Option<usize>,
        task_timeout: usize,
        is_check: bool,
        acquisition: A,
    ) -> Self {
        let task = Task::new(
            format!("{}-notify", SMSER_REDIS_PREFIX),
            format!("{}-read-lock", SMSER_REDIS_PREFIX),
            format!("{}-run-task", SMSER_REDIS_PREFIX),
            format!("{}-run-num", SMSER_REDIS_PREFIX),
            task_size,
            task_timeout,
            is_check,
            task_timeout,
        );
        Self {
            app_core,
            redis,
            task,
            acquisition,
        }
    }
    //发送模板消息
    #[allow(clippy::too_many_arguments)]
    pub async fn send(
        &self,
        app_id: Option<u64>,
        mobiles: &[(&str, &str)],
        tpl_id: &str,
        tpl_var: &str,
        send_time: &Option<u64>,
        user_id: &Option<u64>,
        cancel_key: &Option<String>,
    ) -> Result<u64, String> {
        let mobiles = mobiles
            .iter()
            .collect::<HashSet<_>>()
            .into_iter()
            .map(|e| (e.0.to_owned(), e.1.to_owned()))
            .collect::<Vec<(String, String)>>();

        let nt = now_time().unwrap_or_default();
        let send_time = send_time.unwrap_or(nt);
        let send_time = if send_time < nt { nt } else { send_time };
        self.acquisition
            .sms_record()
            .send_check(app_id, tpl_id, &mobiles, send_time)
            .await?;
        let id = self
            .acquisition
            .sms_record()
            .add(&mobiles, tpl_id, tpl_var, &send_time, user_id, cancel_key)
            .await?;
        let mut redis = self.redis.get().await.map_err(|e| e.to_string())?;
        if let Err(err) = self.task.notify(&mut redis).await {
            warn!("sms is add [{}] ,but send fail :{}", id, err)
        }
        Ok(id)
    }
    //取消发送
    pub async fn cancal(&self, cancel_key: &str, user_id: &u64) -> Result<u64, String> {
        let data = self
            .acquisition
            .sms_record()
            .cancel_data(cancel_key)
            .await?;
        let mut redis = self.redis.get().await.map_err(|e| e.to_string())?;
        let mut succ = 0;
        for tmp in data {
            let tdata = self
                .task
                .task_data(&mut redis)
                .await
                .map_err(|e| e.to_string())?;
            if tdata.get(&tmp.id).is_none() {
                self.acquisition
                    .sms_record()
                    .cancel_id(&tmp, user_id)
                    .await?;
                succ += 1;
            }
        }
        Ok(succ)
    }
    //后台发送任务，内部循环不退出
    pub async fn task<ST: Send + Sync + 'static + Clone, SE: SmserTaskExecutioner<ST>>(
        &self,
        se: SE,
    ) where
        SmserTask<ST, SE>: TaskExecutioner<u64, SmsTaskItem<T>>,
    {
        self.task
            .dispatch(
                self.app_core.clone(),
                &self.acquisition,
                SmserTask::<_, _>::new(self.acquisition.sms_record().to_owned(), se),
            )
            .await;
    }
}
