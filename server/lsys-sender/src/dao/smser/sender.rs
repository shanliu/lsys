use std::{
    collections::HashSet,
    sync::{atomic::AtomicU32, Arc},
};

use async_trait::async_trait;
use lsys_core::{now_time, AppCore, RequestEnv};

use tracing::warn;

use crate::{
    dao::{SenderError, SenderResult, TaskExecutioner},
    model::SenderSmsMessageModel,
};

use super::{
    super::TaskSender, SmsTaskAcquisition, SmsTaskItem, SmsTaskRecord, SmserTaskExecutioner,
};

const SMSER_REDIS_PREFIX: &str = "sender-sms-";

pub struct SmsSender<A: SmsTaskAcquisition<T>, T: Send + Sync + 'static + Clone> {
    redis: deadpool_redis::Pool,
    task: TaskSender<u64, SmsTaskItem<T>>,
    acquisition: A,
}

impl<A: SmsTaskAcquisition<T>, T: Send + Sync + 'static + Clone> SmsSender<A, T> {
    //短信发送
    pub fn new(
        redis: deadpool_redis::Pool,
        task_size: Option<usize>,
        task_timeout: usize,
        is_check: bool,
        acquisition: A,
    ) -> Self {
        let task = TaskSender::new(
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
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        let mobiles = mobiles
            .iter()
            .collect::<HashSet<_>>()
            .into_iter()
            .map(|e| (e.0.to_owned(), e.1.to_owned()))
            .collect::<Vec<(String, String)>>();

        let nt = now_time().unwrap_or_default();
        let sendtime = send_time.unwrap_or(nt);
        let sendtime = if sendtime < nt { nt } else { sendtime };
        self.acquisition
            .sms_record()
            .send_check(app_id, tpl_id, &mobiles, sendtime)
            .await?;
        let id = self
            .acquisition
            .sms_record()
            .add(
                &mobiles,
                &app_id.unwrap_or_default(),
                tpl_id,
                tpl_var,
                &sendtime,
                user_id,
                cancel_key,
                env_data,
            )
            .await?;
        if send_time
            .map(|e| e - 1 <= now_time().unwrap_or_default())
            .unwrap_or(true)
        {
            let mut redis = self.redis.get().await?;
            if let Err(err) = self.task.notify(&mut redis).await {
                warn!("sms is add [{}] ,but send fail :{}", id, err)
            }
        }
        Ok(id)
    }
    //通过ID取消发送
    pub async fn cancal_from_message(
        &self,
        msg: &SenderSmsMessageModel,
        user_id: &u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        let mut redis = self.redis.get().await?;
        let tdata = self.task.task_data(&mut redis).await?;
        if tdata.get(&msg.id).is_none() {
            self.acquisition
                .sms_record()
                .cancel_form_message(msg, user_id, env_data)
                .await?;
        }
        Ok(1)
    }
    //通过KEY取消发送
    pub async fn cancal_from_key(
        &self,
        cancel_key: &str,
        user_id: &u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        let data = self
            .acquisition
            .sms_record()
            .cancel
            .cancel_data(cancel_key)
            .await?;
        let mut redis = self.redis.get().await?;
        let mut succ = 0;
        for tmp in data {
            let tdata = self.task.task_data(&mut redis).await?;
            if tdata.get(&tmp.id).is_none() {
                self.acquisition
                    .sms_record()
                    .cancel_form_key(&tmp, user_id, env_data)
                    .await?;
                succ += 1;
            }
        }
        Ok(succ)
    }
    //后台发送任务，内部循环不退出
    pub async fn task(
        &self,
        app_core: Arc<AppCore>,
        sms_record: Arc<SmsTaskRecord>,
        se: Vec<Box<dyn SmserTaskExecutioner<T>>>,
    ) -> SenderResult<()> {
        self.task
            .dispatch(
                app_core,
                &self.acquisition,
                SmserTask::<_>::new(sms_record, se)?,
            )
            .await;
        Ok(())
    }
}

#[derive(Clone)]
pub struct SmserTask<T: Send + Sync> {
    inner: Arc<Vec<Box<dyn SmserTaskExecutioner<T>>>>,
    record: Arc<SmsTaskRecord>,
    i: Arc<AtomicU32>,
}

impl<T: Send + Sync + Clone> SmserTask<T> {
    pub fn new(
        record: Arc<SmsTaskRecord>,
        se: Vec<Box<dyn SmserTaskExecutioner<T>>>,
    ) -> SenderResult<SmserTask<T>> {
        if se.is_empty() {
            return Err(SenderError::System("can't set task is empty".to_string()));
        }
        Ok(SmserTask {
            inner: Arc::new(se),
            record,
            i: AtomicU32::new(0).into(),
        })
    }
}

#[async_trait]
impl<T: Send + Sync + 'static + Clone> TaskExecutioner<u64, SmsTaskItem<T>> for SmserTask<T> {
    async fn exec(&self, val: SmsTaskItem<T>) -> SenderResult<()> {
        let len = self.inner.len();
        let now = if self.i.load(std::sync::atomic::Ordering::Relaxed) as usize > len {
            self.i.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        } else {
            self.i.store(0, std::sync::atomic::Ordering::Relaxed);
            0
        } as usize;
        let now = if now > len { len } else { now };
        if let Some(tmp) = self.inner.get(now) {
            tmp.exec(val, &self.record).await?;
        }
        Ok(())
    }
}
