// 基于REDIS,实现多节点并行执行不同任务
// 使用示例
// 短信,邮件的从数据库中获取待发送记录,由多个主机同时进行非重复的批量发送
use async_trait::async_trait;
use redis::aio::MultiplexedConnection;
use redis::{
    AsyncCommands, ErrorKind, FromRedisValue, RedisError, RedisResult, ToRedisArgs, Value,
};
use serde::{Deserialize, Serialize};

use crate::{now_time, AppCore, AppCoreError, IntoFluentMessage};
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::marker::PhantomData;
use std::str::from_utf8;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::Mutex;
use tokio::task::{AbortHandle, JoinSet};
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

use super::{TaskNotify, TaskNotifyConfig};

//最外层的任务派发封装

//任务相关数据
#[derive(Serialize, Deserialize, Clone)]
pub struct TaskData {
    //执行任务的HOST
    pub host: String,
    //开始执行任务时间
    pub time: u64,
}
impl FromRedisValue for TaskData {
    fn from_redis_value(val: &Value) -> RedisResult<Self> {
        let valstr = match *val {
            Value::BulkString(ref bytes) => from_utf8(bytes)?.to_string(),
            _ => {
                return Err(RedisError::from((
                    ErrorKind::TypeError,
                    "Response was of incompatible type",
                    format!(
                        "Response type not string compatible. (response was {:?})",
                        val
                    ),
                )))
            }
        };
        match serde_json::from_str::<TaskData>(&valstr) {
            Ok(data) => Ok(data),
            Err(err) => Err(RedisError::from((
                ErrorKind::TypeError,
                "Response was of incompatible type",
                format!(
                    "Response type parse error:{}. (response was {:?})",
                    err, val
                ),
            ))),
        }
    }
}
impl ToRedisArgs for TaskData {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        out.write_arg(serde_json::to_string(self).unwrap_or_default().as_bytes())
    }
}

// 任务TRAIT
pub trait TaskItem<I: FromRedisValue + ToRedisArgs + Eq + Hash + Send + Sync + Display>:
    Send + 'static
{
    fn to_task_pk(&self) -> I;
    fn to_task_data(&self) -> TaskData {
        TaskData {
            host: hostname::get()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            time: now_time().unwrap_or_default(),
        }
    }
}

// 任务执行
// 具体的任务接口实现该特征
#[async_trait]
pub trait TaskExecutor<
    I: FromRedisValue + ToRedisArgs + Eq + Hash + Send + Sync + Display,
    T: TaskItem<I>,
>: Send + Sync + 'static
{
    async fn exec(&self, val: T) -> Result<(), String>;
}

// 任务获取结果
pub struct TaskRecord<
    I: FromRedisValue + ToRedisArgs + Eq + Hash + Send + Sync + Display,
    T: TaskItem<I>,
> {
    // 任务数据,传入 TaskExecutor 中完成具体任务
    pub result: Vec<T>,
    // 是否有下一页任务,返回TRUE将继续下一次获取任务
    pub next: bool,
    marker_i: PhantomData<I>,
}
// 新建任务获取结果
impl<I: FromRedisValue + ToRedisArgs + Eq + Hash + Send + Sync + Display, T: TaskItem<I>>
    TaskRecord<I, T>
{
    pub fn new(result: Vec<T>, next: bool) -> Self {
        Self {
            result,
            next,
            marker_i: PhantomData,
        }
    }
}

// 执行任务获取接口
#[async_trait]
pub trait TaskAcquisition<
    I: FromRedisValue + ToRedisArgs + Eq + Hash + Send + Sync + Display,
    T: TaskItem<I>,
>
{
    // @var tasking_record 为当前正在执行中的任务ID,时间,及所在HOST
    // @var limit 返回的最大执行任务量
    // @return 需要待执行任务列表,返回有结果时,将立即加入执行任务中
    async fn read_exec_task(
        &self,
        tasking_record: &HashMap<I, TaskData>,
        limit: usize,
    ) -> Result<TaskRecord<I, T>, String>;
}

pub struct TaskDispatchConfig {
    pub notify_config: Arc<TaskNotifyConfig>,
    pub task_size: usize,       //同时任务任务数量,默认等于CPU数量2倍
    pub is_timeout_check: bool, //是否定时检测遗漏执行任务
    pub task_timeout: usize,    //任务最大执行时间(任务检测时的时间间隔)
    read_lock_timeout: usize,
    read_size: usize,
    read_lock_key: String, // 任务读取锁定Redis KEY
    task_list_key: String, //存放执行中任务的REDIS key
}

impl TaskDispatchConfig {
    pub fn new(
        notify_config: Arc<TaskNotifyConfig>,
        task_timeout: usize,
        is_timeout_check: bool,
        task_size: Option<usize>,
    ) -> Self {
        let read_lock_key = format!("td-{}-read-lock", notify_config.task_name);
        let task_list_key = format!("td-{}-task-list", notify_config.task_name);
        let task_size = task_size.unwrap_or_else(num_cpus::get);
        let task_timeout = if task_timeout == 0 { 300 } else { task_timeout };
        let read_lock_timeout = task_timeout;
        Self {
            notify_config,
            task_size,
            is_timeout_check,
            task_timeout,
            read_lock_timeout,
            read_lock_key,
            task_list_key,
            read_size: task_size,
        }
    }
    pub fn read_lock_key(&self) -> &str {
        &self.read_lock_key
    }
    pub fn task_list_key(&self) -> &str {
        &self.task_list_key
    }
}

// 任务派发执行抽象实现
pub struct TaskDispatch<
    I: FromRedisValue + ToRedisArgs + Eq + Hash + Send + Sync + Display + Clone,
    T: TaskItem<I>,
> {
    config: Arc<TaskDispatchConfig>,
    notify: Arc<TaskNotify>,
    //任务触发监听的REDIS KEY
    //  list_notify: String,
    //任务读取锁定Redis KEY
    //read_lock_key: String,
    //任务读取锁定超时,大于等于task_timeout
    //read_lock_timeout: usize,
    //存放执行中任务的REDIS key
    //  pub(super) task_list_key: String,
    //是否定时检测遗漏执行任务
    //  is_timeout_check: bool,
    //定时检测遗漏执行任务时间,超过此时间在被再次执行(任务检测时的时间间隔)
    //pub(super) task_timeout: usize,
    //同时执行任务数量
    // task_size: usize,
    //每次获取记录数量,等于 同时执行任务数量
    //   pub read_size: usize,
    marker_i: PhantomData<I>,
    marker_t: PhantomData<T>,
    redis: deadpool_redis::Pool,
}

impl<
        I: FromRedisValue + ToRedisArgs + Eq + Hash + Send + Sync + Display + Clone,
        T: TaskItem<I>,
    > TaskDispatch<I, T>
{
    pub fn new(
        redis: deadpool_redis::Pool,
        notify: Arc<TaskNotify>,
        config: Arc<TaskDispatchConfig>,
    ) -> Self {
        // let task_size = config.task_size.unwrap_or_else(num_cpus::get);
        // let task_timeout = if config.task_timeout == 0 {
        //     300
        // } else {
        //     config.task_timeout
        // };
        // let read_lock_timeout = task_timeout;
        Self {
            // list_notify: config.list_notify.to_string(),
            // read_lock_key: config.read_lock_key.to_string(),
            // read_lock_timeout,
            // task_list_key: config.task_list_key.to_string(),
            // is_timeout_check: config.is_timeout_check,
            // task_timeout,
            // task_size,
            // read_size: task_size,
            marker_i: PhantomData,
            marker_t: PhantomData,
            redis,
            config,
            notify,
        }
    }
}

impl<
        I: FromRedisValue + ToRedisArgs + Eq + Hash + Send + Sync + Display + Clone,
        T: TaskItem<I>, // 实在不想细细折腾，直接 'static ，毕竟T也没打算带用带引用
    > TaskDispatch<I, T>
{
    // 通知执行模块进行任务执行操作
    // * `redis` - 存放任务的RDIS
    // pub async fn notify(&self) -> Result<(), AppCoreError> {
    //     debug!("notify send :{}", self.list_notify);
    //     let mut redis = self.redis.get().await?;
    //     self._notify(&mut redis).await?;
    //     Ok(())
    // }
    // async fn _notify(&self, redis: &mut MultiplexedConnection) -> Result<(), RedisError> {
    //     if let Ok(len) = redis.llen::<&str, i64>(&self.list_notify).await {
    //         if len > 1 {
    //             return Ok(());
    //         }
    //     }
    //     redis.lpush(&self.list_notify, 1).await
    // }

    /// 获得执行中任务信息
    /// * `redis` - 存放执行任务的RDIS
    pub async fn task_data(&self) -> Result<HashMap<I, TaskData>, AppCoreError> {
        let mut redis = self.redis.get().await?;
        self._task_data(&mut redis).await
    }
    async fn _task_data(
        &self,
        redis: &mut MultiplexedConnection,
    ) -> Result<HashMap<I, TaskData>, AppCoreError> {
        let redis_data_opt: Result<Option<HashMap<I, TaskData>>, _> =
            redis.hgetall(self.config.task_list_key()).await;
        Ok(redis_data_opt.map(|data| data.unwrap_or_default())?)
    }

    // 任务执行处理函数
    async fn run_task<E: TaskExecutor<I, T>>(
        task_set: &mut JoinSet<()>,
        task_ing: &mut Vec<(I, AbortHandle)>,
        v: T,
        task_list_key: String,
        task_executor: Arc<E>,
        run_size: &mut usize,
    ) {
        //任务大小减一
        //把run_size 放到这里减,方便后期扩展,如启动任务失败时可不加
        *run_size -= 1;
        let pk = v.to_task_pk();
        debug!("add async task start [{}]:{}", task_list_key, pk);
        //并行执行任务
        let abort = task_set.spawn(async move {
            let pk = v.to_task_pk();
            debug!("async task start [{}]:{}", task_list_key, pk);
            if let Err(err) = task_executor.exec(v).await {
                info!("async task exec fail :{}", err);
            }
            debug!("async task end [{}]:{}", task_list_key, pk);
        });
        debug!("add async task end :{}", pk);
        task_ing.push((pk, abort));
    }
    /// 从 TaskAcquisition 获取任务,并通过 TaskExecutor 执行
    /// * `app_core` - 公共APP句柄,用于创建REDIS
    /// * `task_reader` - 任务读取实现(返回需要立即执行的任务)
    /// * `task_executor` - 任务执行实现
    pub async fn dispatch<R: TaskAcquisition<I, T>, E: TaskExecutor<I, T>>(
        &self,
        app_core: Arc<AppCore>,
        task_reader: &R,
        task_executor: Arc<E>,
    ) {
        if self.config.task_size == 0 {
            info!("task not runing [task size is zero]");
            return;
        }

        let (channel_sender, mut channel_receiver) =
            tokio::sync::mpsc::channel::<T>(self.config.task_size);
        let task_list_key = self.config.task_list_key().to_owned();
        let redis_client = match app_core.create_redis_client().await {
            Ok(redis_client) => redis_client,
            Err(err) => {
                error!(
                    "create redis fail:{}",
                    err.to_fluent_message().default_format()
                );
                return;
            }
        };
        let task_redis_client = redis_client.clone();
        let max_size = self.config.task_size;
        debug!(
            "Concurrent exec max[{}]:{} task",
            task_list_key, self.config.task_size
        );
        //从 本地channel 中拿数据并执行
        tokio::spawn(async move {
            //连接REDIS
            let conn = loop {
                match task_redis_client.get_connection_manager().await {
                    Ok(conn) => break conn,
                    Err(err) => {
                        warn!("task redis get fail :{}", err);
                        sleep(Duration::from_secs(1)).await;
                    }
                }
            };
            let redis_conn = Arc::new(Mutex::new(conn));
            let mut run_size = max_size;
            let mut task_empty;
            let mut task_set = JoinSet::new(); //进行中任务,没法将任务数据在这关联,所以用 task_ing 关联
            let mut task_ing = vec![]; //任务数据跟任务处理关联数组

            'task_main: loop {
                debug!("start exec task:{}", task_list_key);

                //从channel 获取任务,不阻塞
                task_empty = match channel_receiver.try_recv() {
                    Ok(v) => {
                        //获取到任务,执行任务
                        Self::run_task(
                            &mut task_set,
                            &mut task_ing,
                            v,
                            task_list_key.clone(),
                            task_executor.clone(),
                            &mut run_size,
                        )
                        .await;
                        false
                    }
                    Err(err) => {
                        if err != TryRecvError::Empty {
                            error!("task channel error:{}", err);
                            sleep(Duration::from_secs(1)).await;
                            false
                        } else {
                            //未获取到任务,下一步阻塞等待
                            true
                        }
                    }
                };
                //未获取到任务,且还有闲置 执行任务的task
                if task_empty && run_size > 0 {
                    //查找已完成任务列表
                    let mut finsih_pk = Vec::with_capacity(max_size);
                    task_ing = task_ing
                        .into_iter()
                        .filter(|(pk, abt)| {
                            if abt.is_finished() {
                                finsih_pk.push(pk.to_owned());
                                false
                            } else {
                                true
                            }
                        })
                        .collect::<Vec<(I, AbortHandle)>>();
                    //未查找到已完成任务,可能上一次已处理完.重新进入等待
                    if !finsih_pk.is_empty() {
                        //存在处理完任务,进行REDIS解锁及增加空闲任务数量
                        //REDIS一次性全部解锁完在释放,省的多次获取锁
                        let mut redis = redis_conn.lock().await;
                        for pk in finsih_pk {
                            run_size += 1;
                            match redis.hdel(&task_list_key, &pk).await {
                                Ok(()) => {
                                    debug!("clear runing task:{}", pk);
                                }
                                Err(err) => {
                                    warn!("clear runing task fail:{}", err);
                                }
                            }
                        }
                    }

                    'recv: loop {
                        if task_set.is_empty() {
                            //异步阻塞等待任务
                            match channel_receiver.recv().await {
                                Some(v) => {
                                    Self::run_task(
                                        &mut task_set,
                                        &mut task_ing,
                                        v,
                                        task_list_key.clone(),
                                        task_executor.clone(),
                                        &mut run_size,
                                    )
                                    .await;
                                    break 'recv;
                                }
                                None => {
                                    //异步阻塞未获取任务,可能发生错误,重新回到不阻塞 try_recv 去获取详细
                                    info!("channel no task ");
                                    continue 'task_main;
                                }
                            }
                        } else {
                            //有进行中任务,监听任务完成跟新增
                            tokio::select! {
                                res = channel_receiver.recv() => {
                                    match res {
                                        Some(v) => {
                                            Self::run_task(
                                                &mut task_set,
                                                &mut task_ing,
                                                v,

                                                task_list_key.clone(),
                                                task_executor.clone(),
                                                &mut run_size,
                                            )
                                            .await;
                                            break 'recv;
                                        }
                                        None => {
                                            //异步阻塞未获取任务,可能发生错误,重新回到不阻塞 try_recv 去获取详细
                                            info!("channel no task ");
                                            continue 'task_main;
                                        }
                                    }
                                }
                                res = task_set.join_next()=> {
                                    if let Some(res)=res{//理论上不会NONE
                                        if let Err(err) = res {
                                            //有任务PANIC了,非稳定版没法捕捉到任务ID,等TOKIO升级后在修改...
                                            error!("task error[select]:{:?}", err);
                                        }
                                        //查找已完成任务列表
                                        let mut finsih_pk = Vec::with_capacity(max_size);
                                        task_ing = task_ing
                                            .into_iter()
                                            .filter(|(pk, abt)| {
                                                if abt.is_finished() {
                                                    finsih_pk.push(pk.to_owned());
                                                    false
                                                } else {
                                                    true
                                                }
                                            })
                                            .collect::<Vec<(I, AbortHandle)>>();
                                        //未查找到已完成任务,可能上一次已处理完.重新进入等待
                                        if !finsih_pk.is_empty() {
                                            //存在处理完任务,进行REDIS解锁及增加空闲任务数量
                                            //REDIS一次性全部解锁完在释放,省的多次获取锁
                                            let mut redis = redis_conn.lock().await;
                                            for pk in finsih_pk {
                                                run_size += 1;
                                                match redis.hdel(&task_list_key, &pk).await {
                                                    Ok(()) => {
                                                        debug!("clear runing task[select]:{}", pk);
                                                    }
                                                    Err(err) => {
                                                        warn!("clear runing task fail[select]:{}", err);
                                                    }
                                                }
                                            }
                                        }
                                    }else{
                                        warn!("[task] select task set is empty");//理论上,永远不会进入这里
                                    }
                                    continue 'recv;
                                },
                            };
                        }
                    }
                }
                //任务处理已满,需等待进行中任务处理完在继续
                if run_size == 0 && !task_set.is_empty() {
                    while let Some(res) = task_set.join_next().await {
                        if let Err(err) = res {
                            //有任务PANIC了,非稳定版没法捕捉到任务ID,等TOKIO升级后在修改...
                            error!("task error:{:?}", err);
                        }
                        //查找已完成任务列表
                        let mut finsih_pk = Vec::with_capacity(max_size);
                        task_ing = task_ing
                            .into_iter()
                            .filter(|(pk, abt)| {
                                if abt.is_finished() {
                                    finsih_pk.push(pk.to_owned());
                                    false
                                } else {
                                    true
                                }
                            })
                            .collect::<Vec<(I, AbortHandle)>>();
                        //未查找到已完成任务,可能上一次已处理完.重新进入等待
                        if finsih_pk.is_empty() {
                            continue;
                        }
                        //存在处理完任务,进行REDIS解锁及增加空闲任务数量
                        //REDIS一次性全部解锁完在释放,省的多次获取锁
                        let mut redis = redis_conn.lock().await;
                        for pk in finsih_pk {
                            run_size += 1;
                            match redis.hdel(&task_list_key, &pk).await {
                                Ok(()) => {
                                    debug!("clear runing task:{}", pk);
                                }
                                Err(err) => {
                                    warn!("clear runing task fail:{}", err);
                                }
                            }
                        }
                        break;
                        //退出任务完成检测,进入任务处理流程
                    }
                }
            }
        });
        debug!("connect redis {}", self.config.task_list_key());
        let list_notify_key = self.config.notify_config.list_notify_key();
        loop {
            //监听 list_notify 通知,获取发送任务加入到发送列表(本地channel 及 redis执行任务列表)
            debug!("listen task:{}", self.config.task_list_key());
            match redis_client.get_multiplexed_async_connection().await {
                Ok(mut redis) => {
                    let block: Result<Option<()>, _> = redis
                        .blpop(list_notify_key, self.config.task_timeout as f64)
                        .await;
                    match block {
                        Ok(a) => {
                            if a.is_none() {
                                if !self.config.is_timeout_check {
                                    continue;
                                }
                                info!("timeout check task:{}", self.config.task_list_key());
                            } else {
                                debug!("read task data:{}", self.config.task_list_key());
                            }
                        }
                        Err(err) => {
                            warn!("read pop error[{}]:{}", self.config.task_list_key(), err);
                            sleep(Duration::from_secs(1)).await;
                            continue;
                        }
                    };

                    match redis.ltrim(list_notify_key, 0, -1).await {
                        Ok(()) => {
                            debug!("clear list succ :{}", self.config.task_list_key(),);
                        }
                        Err(err) => {
                            warn!("clear list error:{}:{}", self.config.task_list_key(), err);
                        }
                    };
                    match redis.set_nx(&self.config.read_lock_key, 1).await {
                        //确保读取待执行任务,全局只有一个在执行
                        Ok(()) => {
                            debug!("lock read succ :{}", self.config.task_list_key(),);
                        }
                        Err(err) => {
                            warn!("lock read error:{}", err);
                            sleep(Duration::from_secs(1)).await;
                            continue;
                        }
                    };
                    match redis
                        .expire(
                            &self.config.read_lock_key,
                            self.config.read_lock_timeout as i64,
                        )
                        .await
                    {
                        Ok(()) => {}
                        Err(err) => {
                            match redis.del(&self.config.read_lock_key).await {
                                Ok(()) => {}
                                Err(err) => {
                                    warn!("expire set fail,delete lock fail:{}", err);
                                }
                            };
                            warn!("set expire error:{}", err);
                            continue;
                        }
                    };
                    //完成读取锁定

                    //获取当前执行任务中数据
                    let redis_data = match self._task_data(&mut redis).await {
                        Ok(data) => data,
                        Err(err) => {
                            warn!(
                                "get run task data error:{}",
                                err.to_fluent_message().default_format()
                            );
                            match redis.del(&self.config.read_lock_key).await {
                                Ok(()) => {}
                                Err(err) => {
                                    warn!("get run task data fail,and read lock error:{}", err);
                                }
                            };
                            continue;
                        }
                    };
                    let nt = now_time().unwrap_or_default();
                    //在执行且未超时任务数据
                    let mut filter_data = HashMap::new();
                    //过滤掉超时的任务中的数据
                    for (k, v) in redis_data {
                        //执行开始+超时 < 当前时间
                        if v.time + (self.config.task_timeout as u64) < nt {
                            match redis.hdel(self.config.task_list_key(), &k).await {
                                Ok(()) => {}
                                Err(err) => {
                                    warn!("time out clean runing task fail:{}", err);
                                }
                            }
                            continue;
                        }
                        filter_data.insert(k, v);
                    }
                    debug!(
                        "on task data:{} total:{}",
                        self.config.task_list_key(),
                        filter_data.len()
                    );
                    let task_data = match task_reader
                        .read_exec_task(&filter_data, self.config.read_size)
                        .await
                    {
                        Ok(data) => data,
                        Err(err) => {
                            warn!(
                                "read task:{} record error:{}",
                                self.config.task_list_key(),
                                err
                            );
                            match redis.del(&self.config.read_lock_key).await {
                                Ok(()) => {}
                                Err(err) => {
                                    warn!("read task fail ,del read lock error:{}", err);
                                }
                            };
                            continue;
                        }
                    };
                    //数据读取完成，解读取锁定
                    match redis.del(&self.config.read_lock_key).await {
                        Ok(()) => {}
                        Err(err) => {
                            warn!(
                                "read task:{} fail ,del read lock error:{}",
                                self.config.task_list_key(),
                                err
                            );
                        }
                    };
                    if task_data.result.is_empty() {
                        //无任务重新监听
                        info!("not task:{} record data ", self.config.task_list_key());
                        continue;
                    }
                    //添加任务中的数据
                    let mut add_task = Vec::with_capacity(task_data.result.len());

                    for r in task_data.result {
                        let i = r.to_task_pk();
                        let v = r.to_task_data();
                        match redis.hset(self.config.task_list_key(), &i, v.clone()).await {
                            //必须添加成功到任务列表中才进行执行
                            Ok(()) => add_task.push(r),
                            Err(err) => {
                                warn!(
                                    "set run task:{} error[{}]:{}",
                                    self.config.task_list_key(),
                                    i,
                                    err
                                );
                                continue;
                            }
                        };
                    }
                    //有下一页数据,通知其他执行服务器继续
                    if task_data.next {
                        if let Err(err) = self.notify._notify(&mut redis).await {
                            warn!(
                                "notify next task:{} fail:{:?}",
                                self.config.task_list_key(),
                                err
                            );
                        }
                    }
                    //把数据添加到任务的channel
                    for tmp in add_task {
                        let pk = tmp.to_task_pk();
                        if let Err(err) = channel_sender.send(tmp).await {
                            warn!(
                                "add task:{} fail ,remove task fail:{}",
                                self.config.task_list_key(),
                                err
                            );
                            match redis.hdel(self.config.task_list_key(), &pk).await {
                                Ok(()) => {}
                                Err(err) => {
                                    warn!(
                                        "add task:{} fail ,remove task fail:{}",
                                        self.config.task_list_key(),
                                        err
                                    );
                                }
                            };
                        } else {
                            debug!("exec task:{} add:{}", self.config.task_list_key(), pk);
                        }
                    }
                    debug!("listen next exec task :{}", self.config.task_list_key());
                }
                Err(err) => {
                    warn!(
                        "task:{} connect redis fail,try listening:{}",
                        self.config.task_list_key(),
                        err
                    );
                    sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }
}
