// 基于REDIS,多个节点下,仅有一个节点在执行任务
// 使用示例
// 子应用秘钥超时,仅在一个节点进行超时回调任务添加

use crate::{now_time, AppCore, AppCoreError, IntoFluentMessage};
use futures_util::{FutureExt, StreamExt};
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, RedisError};
use std::pin::Pin;
use std::time::Duration;
use std::{future::Future, sync::Arc};
use tokio::{
    sync::mpsc::{self, Receiver, Sender},
    time::sleep,
};
use tracing::{debug, error, info, span, trace, warn, Level};

// 任务派发执行抽象实现
#[async_trait::async_trait]
pub trait TimeOutTaskExec: Send + Sync + 'static {
    async fn exec(
        &self,
        max_lock_time: usize,
        //执行延迟锁定时间,当timeout 任务超过 TimeOutTaskConfig.max_lock_time 时,需拆分任务分阶段调用该函数
        mut expire_call: impl FnMut() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send,
    ) -> Result<(), String>;
}
#[async_trait::async_trait]
pub trait TimeOutTaskNextTime: Send + Sync + 'static {
    async fn next_time(&self, max_lock_time: usize) -> Result<Option<u64>, String>;
}
pub trait TimeOutTaskExecutor {
    type Exec: TimeOutTaskExec;
    type NextTime: TimeOutTaskNextTime;
}

/// 任务派发配置
/// * `lock_key` - 锁定Redis KEY
/// * `max_lock_time` - 任务最大连续执行时间,最小值60秒
pub struct TimeOutTaskConfig {
    lock_key: String,
    max_lock_time: usize,
}

impl TimeOutTaskConfig {
    pub fn new(lock_key: impl ToString, max_lock_time: usize) -> TimeOutTaskConfig {
        let max_lock_time = if max_lock_time > 60 {
            max_lock_time
        } else {
            60
        };
        Self {
            lock_key: lock_key.to_string(),
            max_lock_time,
        }
    }
}

// 任务派发执行抽象实现
pub struct TimeOutTaskNotify {
    redis: deadpool_redis::Pool,
    config: TimeOutTaskConfig,
    notify_key: String,
}

impl TimeOutTaskNotify {
    pub fn new(redis: deadpool_redis::Pool, config: TimeOutTaskConfig) -> Self {
        let notify_key = format!("tm-notify-lock-{}", config.lock_key);
        TimeOutTaskNotify {
            redis,
            config,
            notify_key,
        }
    }
    /// 发送超时重置通知
    /// * `task` - 任务数据
    pub async fn notify_timeout(&self, timeout: u64) -> Result<(), AppCoreError> {
        let ntime = now_time().unwrap_or_default();
        if timeout <= ntime || ntime - timeout < self.config.max_lock_time as u64 {
            debug!("notify timeout publish :{}", self.notify_key);
            let mut conn = self.redis.get().await?;
            let _: () = conn
                .publish::<&String, String, ()>(&self.notify_key, timeout.to_string())
                .await?;
        }
        Ok(())
    }
}

// 任务派发执行抽象实现
pub struct TimeOutTask<T: TimeOutTaskExecutor> {
    app_core: Arc<AppCore>,
    notfiy: Arc<TimeOutTaskNotify>,
    lock_key: String,
    task_exec: Arc<T::Exec>,
    task_next_time: Arc<T::NextTime>,
}

impl<T: TimeOutTaskExecutor> TimeOutTask<T> {
    pub fn new(
        app_core: Arc<AppCore>,
        notfiy: Arc<TimeOutTaskNotify>,
        task_exec: Arc<T::Exec>,
        task_next_time: Arc<T::NextTime>,
    ) -> Self {
        Self {
            lock_key: format!("tm-task-lock-{}", notfiy.config.lock_key),
            notfiy,
            app_core,
            task_exec,
            task_next_time,
        }
    }
}

impl<T: TimeOutTaskExecutor> TimeOutTask<T> {
    /// 监听外部超时重置操作
    pub async fn listen(self, channel_buffer: Option<usize>) {
        let channel_buffer = channel_buffer.unwrap_or(10);
        let (task_tx, task_rx) = mpsc::channel::<()>(channel_buffer);
        let (timeout_tx, timeout_rx) = mpsc::channel::<()>(channel_buffer);
        let listen_next_time = tokio::spawn({
            let lock_key = self.lock_key.clone();
            let max_lock_time = self.notfiy.config.max_lock_time;
            let redis = self.notfiy.redis.clone();
            let task_tx = task_tx.clone();
            async move {
                Self::listen_next_time(
                    redis,
                    timeout_rx,
                    task_tx,
                    &lock_key,
                    max_lock_time,
                    &self.task_next_time,
                )
                .await;
            }
        });
        let listen_timeout_task = tokio::spawn({
            let lock_key = self.lock_key.clone();
            let max_lock_time = self.notfiy.config.max_lock_time;
            let redis = self.notfiy.redis.clone();
            let timeout_tx = timeout_tx.clone();
            async move {
                Self::listen_timeout_task(
                    redis,
                    task_rx,
                    timeout_tx,
                    &lock_key,
                    max_lock_time,
                    &self.task_exec,
                )
                .await;
            }
        });
        let listen_loop_check = tokio::spawn({
            let lock_key = self.lock_key.clone();
            let max_lock_time = self.notfiy.config.max_lock_time;
            let redis = self.notfiy.redis.clone();
            let task_tx = task_tx.clone();
            async move {
                Self::listen_loop_check(redis, &lock_key, max_lock_time, task_tx).await;
            }
        });
        let listen_redis_sub = tokio::spawn({
            let lock_key = self.lock_key.clone();
            let app_core = self.app_core.clone();
            let notify_key = self.notfiy.notify_key.clone();
            let redis = self.notfiy.redis.clone();
            let timeout_tx = timeout_tx.clone();
            async move {
                Self::listen_redis_sub(app_core, redis, &lock_key, &notify_key, timeout_tx).await;
            }
        });
        drop(timeout_tx);
        drop(task_tx);
        let _ = tokio::join!(
            listen_next_time,
            listen_timeout_task,
            listen_loop_check,
            listen_redis_sub,
        );
    }
    async fn listen_lock_check(
        check_type: &str,
        redis: &mut MultiplexedConnection,
        lock_key: &str,
    ) -> Result<bool, RedisError> {
        match redis.get::<&str, Option<String>>(lock_key).await {
            Ok(host) => match host {
                Some(host) => {
                    if host == hostname::get().unwrap_or_default().to_string_lossy() {
                        return Ok(true);
                    }
                    warn!(
                        "{} listen_lock_check {} not self, lock host is:{}",
                        check_type, lock_key, host
                    );
                }
                None => {
                    warn!("{} listen_lock_check {} is empty", check_type, lock_key);
                }
            },
            Err(err) => {
                warn!("{} listen_lock_check fail :{}", check_type, err);
                return Err(err);
            }
        }
        Ok(false)
    }
    //监听下一个任务时间处理
    async fn listen_next_time(
        redis: deadpool_redis::Pool,
        mut ntc_rx: Receiver<()>,
        task_tx: Sender<()>,
        lock_key: &str,
        max_lock_time: usize,
        executor: &T::NextTime,
    ) {
        info!("timeout_task next_time listen start :{}", lock_key);
        let mut timeout_exec: Option<(tokio::task::JoinHandle<()>, u64)> = None;
        while let Some(()) = ntc_rx.recv().await {
            let _ = span!(Level::INFO, "listen_next_time").enter();
            match executor.next_time(max_lock_time).await {
                Ok(next_time_data) => {
                    let ntime = match next_time_data {
                        Some(t) => t,
                        None => {
                            debug!("listen_next_time not next time");
                            continue;
                        }
                    };
                    let nwtime = now_time().unwrap_or_default();
                    let mut addtime = ntime.saturating_sub(nwtime);
                    let mut is_change = true;
                    if let Some((handle, timeout)) = timeout_exec.take() {
                        //执行前[cancel or no cancel] 执行后 =none
                        if timeout < ntime {
                            addtime = timeout.saturating_sub(nwtime);
                            is_change = false;
                        }
                        if !handle.is_finished() {
                            handle.abort();
                            let result = handle.await;
                            if let Err(err) = result {
                                if !err.is_cancelled() {
                                    warn!("listen_next_time cancel fail :{}", err);
                                } else {
                                    debug!("listen_next_time is cancel");
                                }
                            }
                        }
                    }
                    if addtime == 0 {
                        debug!("listen_next_time now exec");
                        if let Err(err) = task_tx.send(()).await {
                            warn!("listen_next_time send task fail :{}", err);
                        }
                        continue;
                    }
                    if is_change {
                        let mut conn = loop {
                            match redis.get().await {
                                Ok(conn) => break conn,
                                Err(err) => {
                                    warn!("listen_next_time redis get fail :{}", err);
                                    sleep(Duration::from_secs(1)).await;
                                }
                            }
                        };
                        if !Self::listen_lock_check("listen_next_time", &mut conn, lock_key)
                            .await
                            .unwrap_or(false)
                        {
                            continue;
                        }
                        if let Err(err) = conn
                            .expire::<&str, i64>(lock_key, addtime as i64 + max_lock_time as i64)
                            .await
                        {
                            warn!(
                                "listen_next_time redis lock{} expire fail :{}",
                                lock_key, err
                            );
                            continue;
                        }
                    }
                    debug!("listen_next_time add timeout handle on {} seconds", addtime);
                    let handle = tokio::spawn({
                        let tmp_lock_key = lock_key.to_string();
                        let tmp_task_tx = task_tx.clone();
                        let tmp_redis = redis.clone();
                        async move {
                            sleep(Duration::from_secs((addtime + 1) as u64)).await;
                            let mut conn = loop {
                                match tmp_redis.get().await {
                                    Ok(conn) => break conn,
                                    Err(err) => {
                                        warn!("listen_next_time redis get fail :{}", err);
                                        sleep(Duration::from_secs(1)).await;
                                    }
                                }
                            };
                            if !Self::listen_lock_check(
                                "listen_next_time_handle",
                                &mut conn,
                                &tmp_lock_key,
                            )
                            .await
                            .unwrap_or(false)
                            {
                                return;
                            }
                            if let Err(err) = tmp_task_tx.send(()).await {
                                warn!("listen_next_time  timeout ok, send task fail :{}", err);
                            }
                        }
                    });
                    timeout_exec = Some((handle, nwtime + addtime));
                }
                Err(err) => {
                    warn!("listen_next_time get fail :{}", err);
                }
            }
        }
    }
    async fn listen_timeout_task(
        redis: deadpool_redis::Pool,
        mut task_rx: Receiver<()>,
        ntc_tx: Sender<()>,
        lock_key: &str,
        max_lock_time: usize,
        executor: &T::Exec,
    ) {
        info!("timeout_task task listen start:{}", lock_key);
        while let Some(()) = task_rx.recv().await {
            let _ = span!(Level::INFO, "listen_timeout_task").enter();
            let mut conn = loop {
                match redis.get().await {
                    Ok(conn) => break conn,
                    Err(err) => {
                        warn!("listen_timeout_task redis get fail :{}", err);
                        sleep(Duration::from_secs(1)).await;
                    }
                }
            };

            match executor
                .exec(max_lock_time, || {
                    let mut conn_exec = conn.clone();
                    let lock_key = lock_key.to_string();
                    async move {
                        if let Ok(ttl) = conn_exec.ttl::<&str, usize>(&lock_key).await {
                            if ttl < max_lock_time / 2 {
                                let _ = conn_exec
                                    .expire::<&str, i64>(&lock_key, max_lock_time as i64)
                                    .await;
                            }
                        }
                    }
                    .boxed()
                })
                .await
            {
                Ok(()) => {
                    if !Self::listen_lock_check("listen_timeout_task", &mut conn, lock_key)
                        .await
                        .unwrap_or(false)
                    {
                        drop(conn);
                        sleep(Duration::from_secs(1)).await;
                        continue;
                    }
                }
                Err(err) => {
                    warn!("listen_timeout_task exec task fail :{}", err);
                    continue;
                }
            }
            if let Err(err) = ntc_tx.send(()).await {
                warn!("listen_timeout_task send next time task fail :{}", err);
            }
        }
    }

    async fn listen_loop_check(
        redis: deadpool_redis::Pool,
        lock_key: &str,
        max_lock_time: usize,
        task_tx: Sender<()>,
    ) {
        info!(
            "timeout_task check listen start :{} {} seconds",
            lock_key, max_lock_time
        );
        loop {
            let _ = span!(Level::INFO, "listen_loop_check").enter();
            let mut conn = loop {
                match redis.get().await {
                    Ok(conn) => break conn,
                    Err(err) => {
                        warn!("listen_timeout_check redis get fail :{}", err);
                        sleep(Duration::from_secs(1)).await;
                    }
                }
            };
            let set_host = hostname::get()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            trace!(
                "listen_timeout_check start lock key[{}] val[{}]",
                lock_key,
                set_host
            );
            match conn
                .set_nx::<&str, String, bool>(lock_key, set_host.clone())
                .await
            {
                Ok(mut status) => {
                    debug!(
                        "listen_timeout_check lock key[{}] status: {}",
                        lock_key, status
                    );
                    let mut set_expire = true;
                    if !status {
                        match conn.ttl::<&str, i64>(lock_key).await {
                            Ok(ttl) => {
                                debug!("listen_timeout_check lock key[{}] ttl: {}", lock_key, ttl);
                                if ttl > 0 {
                                    set_expire = false
                                }
                            }
                            Err(err) => {
                                debug!(
                                    "listen_timeout_check lock key{} ttl fail: {}",
                                    lock_key, err
                                );
                            }
                        }
                    }
                    let other_host = match conn.get::<&str, String>(lock_key).await {
                        Ok(t) => {
                            if t == set_host {
                                status = true
                            }
                            format!("other host lock:{}", t)
                        }
                        Err(e) => format!("get lock host fail:{}", e),
                    };
                    if set_expire || status {
                        if let Err(err) = conn
                            .expire::<&str, i64>(lock_key, max_lock_time as i64)
                            .await
                        {
                            warn!(
                                "listen_timeout_check lock key[{}] set expire fail:{}",
                                lock_key, err
                            );
                        }
                    }

                    if status {
                        if let Err(err) = task_tx.send(()).await {
                            warn!("listen_loop_check send task fail :{}", err);
                        } else {
                            debug!("listen_loop_check send task succ");
                        }
                    } else {
                        debug!("listen_loop_check lock fail :{}", other_host);
                    }
                }
                Err(err) => {
                    info!(
                        "listen_timeout_check task lock key[{}] set fail:{}",
                        lock_key, err
                    );
                }
            }
            debug!(
                "listen_timeout_check lock key[{}] -> sleep {}",
                lock_key, max_lock_time
            );
            drop(conn);
            sleep(Duration::from_secs(if max_lock_time > 3 {
                max_lock_time - 3
            } else {
                max_lock_time
            } as u64))
            .await;
        }
    }

    async fn listen_redis_sub(
        app_core: Arc<AppCore>,
        redis: deadpool_redis::Pool,
        lock_key: &str,
        notify_key: &str,
        ntc_tx: Sender<()>,
    ) {
        info!(
            "timeout_task timeout change listen start:{} && {}",
            notify_key, notify_key
        );
        loop {
            let _ = span!(Level::INFO, "listen_redis_sub").enter();
            match app_core.create_redis_client().await {
                Ok(redis_client) => {
                    let con_res = redis_client.get_async_pubsub().await;
                    match con_res {
                        Ok(mut pubsub) => {
                            let res = pubsub.subscribe(notify_key).await;
                            if let Err(err) = res {
                                error!("listen_redis_sub listen  sub fail :{}", err);
                                sleep(Duration::from_secs(1)).await;
                                continue;
                            } else {
                                info!(
                                    "listen_redis_sub  listen remote channel succ:{}",
                                    notify_key
                                );
                            }
                            let mut pubsub_stream = pubsub.on_message();
                            loop {
                                match pubsub_stream.next().await {
                                    Some(msg) => match msg.get_payload::<String>() {
                                        Ok(pubsub_msg) => {
                                            debug!("listen_redis_sub msg:{}", pubsub_msg);
                                            let mut conn = loop {
                                                match redis.get().await {
                                                    Ok(conn) => break conn,
                                                    Err(err) => {
                                                        warn!(
                                                            "listen_redis_sub redis get fail :{}",
                                                            err
                                                        );
                                                        sleep(Duration::from_secs(1)).await;
                                                    }
                                                }
                                            };
                                            match Self::listen_lock_check(
                                                "listen_redis_sub",
                                                &mut conn,
                                                lock_key,
                                            )
                                            .await
                                            {
                                                Ok(status) => {
                                                    if !status {
                                                        continue;
                                                    }
                                                }
                                                Err(_) => {
                                                    break;
                                                }
                                            }
                                            if let Err(err) = ntc_tx.send(()).await {
                                                warn!(
                                                    "listen_redis_sub send next time task fail :{}",
                                                    err
                                                );
                                            }
                                        }
                                        Err(err) => {
                                            error!("read payload fail :{}", err);
                                            break;
                                        }
                                    },
                                    None => {
                                        debug!("listen_redis_sub none");
                                        break;
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            error!("clear conn redis:{}", err);
                            tokio::time::sleep(Duration::from_secs(1)).await;
                        }
                    }
                }
                Err(err) => {
                    warn!(
                        "create remote notify listen client fail:{}",
                        err.to_fluent_message().default_format()
                    );
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }
}
