use async_trait::async_trait;

use lsys_core::{now_time, AppCore};
use redis::aio::{Connection, ConnectionManager};
use redis::{
    AsyncCommands, ErrorKind, FromRedisValue, RedisError, RedisResult, ToRedisArgs, Value,
};
use serde::{Deserialize, Serialize};
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

//最外层的发送任务派发封装
//不包含具体的发送逻辑

//任务相关数据
#[derive(Serialize, Deserialize, Clone)]
pub struct TaskData {
    //执行发送任务的HOST
    pub host: String,
    //执行发送任务时间
    pub time: u64,
}
impl FromRedisValue for TaskData {
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

// 任务特征
pub trait TaskItem<I: FromRedisValue + ToRedisArgs + Eq + Hash + Send + Sync + Display>:
    Send
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

// 发送执行
// 具体的发送接口实现该特征
#[async_trait]
pub trait TaskExecutor<
    I: FromRedisValue + ToRedisArgs + Eq + Hash + Send + Sync + Display,
    T: TaskItem<I>,
>: Send + Sync + Clone
{
    async fn exec(&self, val: T) -> Result<(), String>;
}

// 任务获取结果
pub struct TaskRecord<
    I: FromRedisValue + ToRedisArgs + Eq + Hash + Send + Sync + Display,
    T: TaskItem<I>,
> {
    // 任务数据,传入 TaskExecutor 中完成具体发送任务
    pub result: Vec<T>,
    // 是否有下一页任务,返回TRUE将继续下一次获取发送任务
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

// 发送任务获取约束
#[async_trait]
pub trait TaskAcquisition<
    I: FromRedisValue + ToRedisArgs + Eq + Hash + Send + Sync + Display,
    T: TaskItem<I>,
>
{
    // @var tasking_record 为当前正在发送中的任务ID,时间,及所在HOST
    // @var limit 返回的最大发送任务量
    // @return 需发送的任务结果集
    async fn read_record(
        &self,
        tasking_record: &HashMap<I, TaskData>,
        limit: usize,
    ) -> Result<TaskRecord<I, T>, String>;
}

// 发送任务抽象实现
pub struct TaskDispatch<
    I: FromRedisValue + ToRedisArgs + Eq + Hash + Send + Sync + Display + Clone,
    T: TaskItem<I>,
> {
    //任务触发监听的REDIS KEY
    list_notify: String,
    //任务读取锁定Redis KEY
    read_lock_key: String,
    //任务读取锁定超时,大于等于check_timeout ,task_timeout
    read_lock_timeout: usize,
    //存放执行中任务的REDIS key
    task_list_key: String,
    //执行中任务的数量的REDIS KEY
    exec_num_key: String,
    //是否定时检测遗漏发送任务
    pub is_check: bool,
    //定时检测遗漏发送任务时间
    pub check_timeout: usize,
    //任务最大执行时间,超过此时间在被再次执行
    pub task_timeout: usize,
    //同时执行任务数量
    pub task_size: usize,
    //每次获取记录数量,等于 同时执行任务数量
    pub read_size: usize,
    marker_i: PhantomData<I>,
    marker_t: PhantomData<T>,
}

impl<
        I: FromRedisValue + ToRedisArgs + Eq + Hash + Send + Sync + Display + Clone,
        T: TaskItem<I>,
    > TaskDispatch<I, T>
{
    /// * `list_notify` - 任务触发监听的REDIS KEY
    /// * `read_lock_key` - 任务读取锁定Redis KEY
    /// * `task_list_key` - 存放执行中任务的REDIS key
    /// * `exec_num_key` - 执行中任务的数量的REDIS KEY
    /// * `task_size` - 同时发送任务数量,默认等于CPU数量2倍
    /// * `task_timeout` - 任务最大执行时间
    /// * `is_check` - 是否定时检测遗漏发送任务
    /// * `check_timeout` - 当使用任务检测时的时间间隔，大于等于任务最大执行时间
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        list_notify: String,
        read_lock_key: String,
        task_list_key: String,
        exec_num_key: String,
        task_size: Option<usize>,
        task_timeout: usize,
        is_check: bool,
        check_timeout: usize,
    ) -> Self {
        let task_size = task_size.unwrap_or_else(num_cpus::get);
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
            task_size,
            read_size: task_size,
            marker_i: PhantomData,
            marker_t: PhantomData,
        }
    }
}

impl<
        I: FromRedisValue + ToRedisArgs + Eq + Hash + Send + Sync + Display + Clone,
        T: TaskItem<I> + 'static, // 实在不想细细折腾，直接 'static ，毕竟T也没打算带用带引用
    > TaskDispatch<I, T>
{
    /// 通知发送模块进行发送操作
    /// * `redis` - 存放发送任务的RDIS
    pub async fn notify(&self, redis: &mut Connection) -> Result<(), RedisError> {
        redis.lpush(&self.list_notify, 1).await
    }
    /// 获得发送中任务信息
    /// * `redis` - 存放发送任务的RDIS
    pub async fn task_data(
        &self,
        redis: &mut Connection,
    ) -> Result<HashMap<I, TaskData>, RedisError> {
        let redis_data_opt: Result<Option<HashMap<I, TaskData>>, _> =
            redis.hgetall(&self.task_list_key).await;
        redis_data_opt.map(|data| data.unwrap_or_default())
    }
    // 任务执行
    #[allow(clippy::too_many_arguments)]
    async fn run_task<E: TaskExecutor<I, T> + 'static>(
        task_set: &mut JoinSet<()>,
        task_ing: &mut Vec<(I, AbortHandle)>,
        v: T,
        redis_conn: Arc<Mutex<ConnectionManager>>,
        exec_num_key: String,
        task_list_key: String,
        task_executor: E,
        run_size: &mut usize,
    ) {
        //任务大小减一
        //把run_size 放到这里减,方便后期扩展,如启动任务失败时可不加
        *run_size -= 1;
        let pk = v.to_task_pk();
        debug!("add async task start [{}]:{}", task_list_key, pk);
        //并行发送任务
        let abort = task_set.spawn(async move {
            let pk = v.to_task_pk();
            debug!("async task start [{}]:{}", task_list_key, pk);
            let res = task_executor.exec(v).await;
            debug!("async task end [{}]:{}", task_list_key, pk);
            if let Err(err) = res {
                //任务执行失败,执行次数减一,PANIC捕捉有问题,不处理...
                warn!("exec fail on {} error:{}", pk, err);
                let mut redis = redis_conn.lock().await;
                match redis.hincr(&exec_num_key, &pk, -1).await {
                    Ok(()) => {}
                    Err(err) => {
                        warn!("remove task num error:{}", err);
                    }
                };
            }
        });
        debug!("add async task end :{}", pk);
        task_ing.push((pk, abort));
    }
    /// 获得发送中任务信息
    /// * `app_core` - 公共APP句柄,用于创建REDIS
    /// * `task_reader` - 任务读取实现
    /// * `task_executor` - 任务发送实现
    pub async fn dispatch<R: TaskAcquisition<I, T>, E: TaskExecutor<I, T> + 'static>(
        &self,
        app_core: Arc<AppCore>,
        task_reader: &R,
        task_executor: E,
    ) {
        if self.task_size == 0 {
            info!("task not runing [task size is zero]");
            return;
        }
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
        let max_size = self.task_size;
        debug!(
            "Concurrent send max[{}]:{} task",
            task_list_key, self.task_size
        );
        //从channel 中拿数据并发送
        tokio::spawn(async move {
            //连接REDIS
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
            let mut run_size = max_size;
            let mut task_empty;
            let mut task_set = JoinSet::new(); //进行中任务,没法将任务数据在这关联,所以用 task_ing 关联
            let mut task_ing = vec![]; //任务数据跟任务处理关联数组

            'task_main: loop {
                debug!("start send task:{}", task_list_key);

                //从channel 获取任务,不阻塞
                task_empty = match channel_receiver.try_recv() {
                    Ok(v) => {
                        //获取到任务,执行任务
                        Self::run_task(
                            &mut task_set,
                            &mut task_ing,
                            v,
                            redis_conn.clone(),
                            exec_num_key.clone(),
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
                //未获取到任务,且还有闲置发送
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
                                        redis_conn.clone(),
                                        exec_num_key.clone(),
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
                                                redis_conn.clone(),
                                                exec_num_key.clone(),
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
                        let v = r.to_task_data();
                        match redis.hset(&self.task_list_key, &i, v.clone()).await {
                            //必须添加成功到发送中才进行发送
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
                    //有下一页数据,通知其他执行服务器继续
                    if task_data.next {
                        if let Err(err) = self.notify(&mut redis).await {
                            warn!("notify next task fail:{}", err);
                        }
                    }
                    //把数据添加到发送channel
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
