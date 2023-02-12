use async_trait::async_trait;

use lsys_core::{now_time, AppCore};
use redis::aio::Connection;
use redis::{
    AsyncCommands, ErrorKind, FromRedisValue, RedisError, RedisResult, ToRedisArgs, Value,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::marker::PhantomData;
use std::str::from_utf8;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

#[async_trait]
pub trait TaskExecutioner<
    I: FromRedisValue + ToRedisArgs + Eq + Hash + Send + Sync + Display,
    T: TaskItem<I>,
>: Clone + Send + Sync + 'static
{
    async fn exec(&self, val: T) -> Result<(), TaskError>;
}

#[async_trait]
pub trait TaskAcquisition<
    I: FromRedisValue + ToRedisArgs + Eq + Hash + Send + Sync + Display,
    T: TaskItem<I>,
>
{
    async fn read_record(
        &self,
        tasking_record: &HashMap<I, TaskValue>,
        limit: usize,
    ) -> Result<TaskRecord<I, T>, TaskError>;
}

pub trait TaskItem<I: FromRedisValue + ToRedisArgs + Eq + Hash + Send + Sync + Display>:
    Send + 'static
{
    fn to_task_pk(&self) -> I;
    fn to_task_value(&self) -> TaskValue;
}

pub enum TaskError {
    Sqlx(sqlx::Error),
    Redis(String),
    Exec(String),
}
impl Display for TaskError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskError::Sqlx(e) => write!(f, "{:?}", e),
            TaskError::Redis(e) => write!(f, "{:?}", e),
            TaskError::Exec(e) => write!(f, "{:?}", e),
        }
    }
}
pub struct TaskRecord<
    I: FromRedisValue + ToRedisArgs + Eq + Hash + Send + Sync + Display,
    T: TaskItem<I>,
> {
    pub result: Vec<T>,
    pub next: bool,
    marker_i: PhantomData<I>,
}

impl<I: FromRedisValue + ToRedisArgs + Eq + Hash + Send + Sync + Display, T: TaskItem<I>>
    TaskRecord<I, T>
{
    pub fn new(result: Vec<T>, next: bool) -> Self {
        Self {
            result,
            next,
            marker_i: PhantomData::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TaskValue {
    pub host: String,
    pub time: u64,
}
impl FromRedisValue for TaskValue {
    fn from_redis_value(val: &Value) -> RedisResult<Self> {
        let valstr = match *val {
            Value::Data(ref bytes) => from_utf8(bytes)?.to_string(),
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
        match serde_json::from_str::<TaskValue>(&valstr) {
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
impl ToRedisArgs for TaskValue {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        out.write_arg(serde_json::to_string(self).unwrap_or_default().as_bytes())
    }
}

pub struct Task<I: FromRedisValue + ToRedisArgs + Eq + Hash + Send + Sync + Display, T: TaskItem<I>>
{
    list_notify: String,
    read_lock_key: String,
    read_lock_timeout: usize,
    task_list_key: String,
    exec_num_key: String,
    pub is_check: bool,
    pub check_timeout: usize,
    pub task_timeout: usize,
    pub task_size: usize,
    pub read_size: usize,
    marker_i: PhantomData<I>,
    marker_t: PhantomData<T>,
}

impl<I: FromRedisValue + ToRedisArgs + Eq + Hash + Send + Sync + Display, T: TaskItem<I>>
    Task<I, T>
{
    pub fn new(
        list_notify: String,
        read_lock_key: String,
        task_list_key: String,
        exec_num_key: String,
        task_timeout: usize,  //任务最大执行时间
        is_check: bool,       //是否进行定期任务检测
        check_timeout: usize, //当使用任务检测时的时间间隔，大于等于任务最大执行时间
    ) -> Self {
        let cpu_num = num_cpus::get();
        let task_timeout = if task_timeout == 0 {
            5 * 60
        } else {
            task_timeout
        };
        let check_timeout = if check_timeout < task_timeout {
            task_timeout
        } else {
            check_timeout
        };
        let read_lock_timeout = check_timeout;
        Self {
            list_notify,
            read_lock_key,
            read_lock_timeout,
            task_list_key,
            exec_num_key,
            is_check,
            check_timeout,
            task_timeout,
            task_size: cpu_num,
            read_size: cpu_num,
            marker_i: PhantomData::default(),
            marker_t: PhantomData::default(),
        }
    }
}

impl<I: FromRedisValue + ToRedisArgs + Eq + Hash + Send + Sync + Display, T: TaskItem<I>>
    Task<I, T>
{
    pub async fn notify(&self, redis: &mut Connection) -> Result<(), TaskError> {
        redis
            .lpush(&self.list_notify, 1)
            .await
            .map_err(|e| TaskError::Redis(e.to_string()))
    }
    pub async fn task_data(
        &self,
        redis: &mut Connection,
    ) -> Result<HashMap<I, TaskValue>, TaskError> {
        let redis_data_opt: Result<Option<HashMap<I, TaskValue>>, _> =
            redis.hgetall(&self.task_list_key).await;
        match redis_data_opt {
            Ok(data) => Ok(data.unwrap_or_default()),
            Err(err) => Err(TaskError::Redis(err.to_string())),
        }
    }
    pub async fn dispatch<R: TaskAcquisition<I, T>, E: TaskExecutioner<I, T>>(
        &self,
        app_core: Arc<AppCore>,
        task_reader: &R,
        task_executioner: E,
    ) {
        let (channel_sender, mut channel_receiver) =
            tokio::sync::mpsc::channel::<T>(self.task_size);
        let task_list_key = self.task_list_key.clone();
        let exec_num_key = self.exec_num_key.clone();
        let redis_client = match app_core.create_redis_client() {
            Ok(redis_client) => redis_client,
            Err(err) => {
                error!("create redis fail:{}", err);
                return;
            }
        };
        let task_redis_client = redis_client.clone();
        tokio::spawn(async move {
            let conn = loop {
                match task_redis_client.get_tokio_connection_manager().await {
                    Ok(conn) => break conn,
                    Err(err) => {
                        warn!("task redis get fail :{}", err);
                        sleep(Duration::from_secs(1)).await;
                    }
                }
            };
            let redis_conn = Arc::new(Mutex::new(conn));
            loop {
                debug!("start send task:{}", task_list_key);
                match channel_receiver.recv().await {
                    Some(v) => {
                        debug!("send task start[{}]:{}", task_list_key, v.to_task_pk());
                        let task_list_key = task_list_key.clone();
                        let exec_num_key = exec_num_key.clone();
                        let execer = task_executioner.clone();
                        let task_redis = redis_conn.clone();
                        tokio::spawn(async move {
                            let pk = v.to_task_pk();
                            debug!("async task start [{}]:{}", task_list_key, pk);
                            let res = execer.exec(v).await;
                            let mut redis = task_redis.lock().await;
                            if let Err(err) = res {
                                warn!("exec fail on {} error:{}", pk, err);
                                match redis.hincr(&exec_num_key, &pk, -1).await {
                                    Ok(()) => {}
                                    Err(err) => {
                                        warn!("remove task num error:{}", err);
                                    }
                                };
                            }
                            match redis.hdel(&task_list_key, &pk).await {
                                Ok(()) => {
                                    debug!("clear runing task:{}", pk);
                                }
                                Err(err) => {
                                    warn!("clear runing task fail:{}", err);
                                }
                            }
                            debug!("async task end [{}]:{}", task_list_key, pk);
                        });
                    }
                    None => {
                        if let Err(e) = channel_receiver.try_recv() {
                            error!("task channel error:{}", e);
                        } else {
                            warn!("task channel close");
                        }
                        sleep(Duration::from_secs(1)).await;
                    }
                };
            }
        });
        debug!("connect redis {}", self.task_list_key);
        loop {
            debug!("listen task:{}", self.task_list_key);
            // redis listen or timeout{
            // redis clean bad add log
            // redis lock bad go to listen
            // redis get task record bad del redis lock and go to listen
            // self.read_task.get_record(task_ing...) bad del redis lock and go to listen
            // redis set task record and del redis lock bad go to listen
            // next true self.notify() bad add log
            // add record data to self.task_channel_sender
            match redis_client.get_async_connection().await {
                Ok(mut redis) => {
                    let block: Result<Option<()>, _> =
                        redis.blpop(&self.list_notify, self.check_timeout).await;
                    match block {
                        Ok(a) => {
                            if a.is_none() {
                                if !self.is_check {
                                    continue;
                                }
                                info!("timeout check task:{}", self.task_list_key);
                            } else {
                                debug!("read task data:{}", self.task_list_key);
                            }
                        }
                        Err(err) => {
                            warn!("read pop error[{}]:{}", self.task_list_key, err);
                            sleep(Duration::from_secs(1)).await;
                            continue;
                        }
                    };

                    match redis.ltrim(&self.list_notify, 0, -1).await {
                        Ok(()) => {}
                        Err(err) => {
                            warn!("clear list error:{}", err);
                        }
                    };
                    match redis.set_nx(&self.read_lock_key, 1).await {
                        Ok(()) => {}
                        Err(err) => {
                            warn!("lock read error:{}", err);
                            continue;
                        }
                    };
                    match redis
                        .expire(&self.read_lock_key, self.read_lock_timeout)
                        .await
                    {
                        Ok(()) => {}
                        Err(err) => {
                            match redis.del(&self.read_lock_key).await {
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

                    //获取当前任务中数据
                    let redis_data = match self.task_data(&mut redis).await {
                        Ok(data) => data,
                        Err(err) => {
                            warn!("get run task data error:{}", err);
                            match redis.del(&self.read_lock_key).await {
                                Ok(()) => {}
                                Err(err) => {
                                    warn!("get run task data fail,and read lock error:{}", err);
                                }
                            };
                            continue;
                        }
                    };
                    let nt = now_time().unwrap_or_default();
                    let mut filter_data = HashMap::new();
                    //过滤掉超时的任务中的数据
                    for (k, v) in redis_data {
                        if v.time + (self.check_timeout as u64) < nt {
                            match redis.hdel(&self.task_list_key, &k).await {
                                Ok(()) => {}
                                Err(err) => {
                                    warn!("time out clean runing task fail:{}", err);
                                }
                            }
                            continue;
                        }
                        filter_data.insert(k, v);
                    }
                    debug!("on task data:{}", filter_data.len());
                    let task_data =
                        match task_reader.read_record(&filter_data, self.read_size).await {
                            Ok(data) => data,
                            Err(err) => {
                                warn!("read task record error:{}", err);
                                match redis.del(&self.read_lock_key).await {
                                    Ok(()) => {}
                                    Err(err) => {
                                        warn!("read task fail ,del read lock error:{}", err);
                                    }
                                };
                                continue;
                            }
                        };
                    //数据读取完成，解读取锁定
                    match redis.del(&self.read_lock_key).await {
                        Ok(()) => {}
                        Err(err) => {
                            warn!("read task fail ,del read lock error:{}", err);
                        }
                    };
                    if task_data.result.is_empty() {
                        //无任务重新监听
                        info!("not task record data ");
                        continue;
                    }
                    //添加任务中的数据
                    let mut add_task = Vec::with_capacity(task_data.result.len());

                    for r in task_data.result {
                        let i = r.to_task_pk();
                        let v = r.to_task_value();
                        match redis.hset(&self.task_list_key, &i, v.clone()).await {
                            Ok(()) => add_task.push(r),
                            Err(err) => {
                                warn!("set run task error[{}]:{}", i, err);
                                continue;
                            }
                        };
                        let add_res: Result<u64, _> = redis.hincr(&self.exec_num_key, &i, 1).await;
                        match add_res {
                            Ok(add_num) => {
                                if add_num > 1 {
                                    warn!("repeat exec on {}", i);
                                }
                            }
                            Err(err) => {
                                warn!("add task num error:{} on {}", err, i);
                            }
                        };
                    }
                    if task_data.next {
                        if let Err(err) = self.notify(&mut redis).await {
                            warn!("notify next task fail:{}", err);
                        }
                    }
                    for tmp in add_task {
                        let pk = tmp.to_task_pk();
                        if let Err(err) = channel_sender.send(tmp).await {
                            warn!("add task fail ,remove task fail:{}", err);
                            match redis.hdel(&self.task_list_key, &pk).await {
                                Ok(()) => {}
                                Err(err) => {
                                    warn!("add task fail ,remove task fail:{}", err);
                                }
                            };
                            match redis.hincr(&self.exec_num_key, &pk, -1).await {
                                Ok(()) => {}
                                Err(err) => {
                                    warn!("remove task num error:{}", err);
                                }
                            };
                        } else {
                            debug!("send task add[{}]:{}", self.task_list_key, pk);
                        }
                    }
                    debug!("listen next send task :{}", self.task_list_key);
                }
                Err(err) => {
                    warn!("connect redis fail{},try listening...", err);
                    sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }
}
