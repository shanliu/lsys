use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use async_trait::async_trait;
use deadpool_redis::PoolError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use snowflake::SnowflakeIdGenerator;
use tokio::sync::{mpsc, Mutex, RwLock,};
use tokio::time::Duration;

use futures_util::StreamExt;

use redis::{AsyncCommands, RedisError};
use tracing::{debug, error, info, warn};

use crate::AppCore;

#[derive(Debug)]
pub enum RemoteNotifyError {
    System(String),
    RedisPool(PoolError),
    Redis(RedisError),
    RemoteTimeOut,
}

impl Display for RemoteNotifyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl Error for RemoteNotifyError {}
impl From<RedisError> for RemoteNotifyError {
    fn from(err: RedisError) -> Self {
        RemoteNotifyError::Redis(err)
    }
}
impl From<PoolError> for RemoteNotifyError {
    fn from(err: PoolError) -> Self {
        RemoteNotifyError::RedisPool(err)
    }
}

//发送消息
#[derive(Serialize, Deserialize, Clone)]
pub struct MsgSendBody {
   pub  data: Value,
   pub  id: i64,
   pub msg_type: u8,
   pub from_host: String,
   pub target_host: Option<String>,
   pub ignore_local: bool,
   pub reply: bool,
}
//执行结果
#[derive(Serialize, Deserialize, Clone)]
pub struct MsgResultBody {
    pub  data: Result<Option<Value>, String>,
    pub  from_host: String,
    pub  reply_id: i64,
}

//消息内容
#[derive(Serialize, Deserialize, Clone)]
pub enum MsgBody {
    Send(MsgSendBody),
    Result(MsgResultBody),
}

#[async_trait]
pub trait RemoteTask:Sync+Send{
    fn msg_type(&self) -> u8;
    async fn run(&self, msg: MsgSendBody) -> Result<Option<Value>, String>;
}

pub struct RemoteNotify {
    channel_name: &'static str,
    app_core: Arc<AppCore>,
    hostname: String,
    redis: deadpool_redis::Pool,
    callback: Mutex<HashMap<i64, mpsc::Sender<MsgResultBody>>>,
    id_generator: Mutex<SnowflakeIdGenerator>,
    run_list: RwLock<Vec<Box<dyn RemoteTask>>>,
}

impl RemoteNotify {
    pub fn new(
        channel_name: &'static str,
        app_core: Arc<AppCore>,
        redis: deadpool_redis::Pool,
    ) -> Result<Self, RemoteNotifyError> {
        let hostname = hostname::get()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let id_generator = Mutex::new(app_core.create_snowflake_id_generator());
        Ok(Self {
            channel_name,
            app_core,
            redis,
            hostname,
            id_generator,
            callback: Mutex::new(HashMap::new()),
            run_list:RwLock::new(vec![]),
        })
    }
}

#[derive(PartialEq, Eq)]
//目标包含本机直接执行方式
pub enum LocalExecType {
    RemoteExec,  //目标包含本机的通过消息方式执行
    LocalExec,   //目标包含本机的直接执行
    IgnoreLocal, //目标包含本机的忽略本机执行
}
pub struct ReplyWait {
    pub max_node: usize,
    pub timeout: u64,
}

impl RemoteNotify {
   
    pub async fn call<T: Serialize>(
        &self,
        msg_type: u8,//类型，外部定义，别重复了
        data: T,//消息数据
        target_host: Option<String>,//执行的目标机器
        local_exe_type: LocalExecType,//执行目标包含本机的执行方式
        reply_wait: Option<ReplyWait>,//执行完是否等待结果
    ) -> Result<Vec<MsgResultBody>, RemoteNotifyError> {
        let mut out = vec![];
        if local_exe_type == LocalExecType::IgnoreLocal//本机忽略执行
            //执行目标仅为本机
            && match &target_host {
                Some(th) => self.hostname == *th, 
                None => false,                  
            }
        {
            //等于不做任何操作
            return Ok(out);
        }
        let msg_id = self.id_generator.lock().await.generate();
        let msg = MsgSendBody {
            data: serde_json::json!(data),
            msg_type,
            id: msg_id,
            from_host: self.hostname.clone(),
            target_host: target_host.to_owned(),
            ignore_local: match local_exe_type {
                LocalExecType::RemoteExec => false,
                LocalExecType::LocalExec | LocalExecType::IgnoreLocal => true,
            },
            reply: reply_wait.is_some(),
        };
        if local_exe_type==LocalExecType::RemoteExec//本机通过消息执行
            || match &target_host {
                Some(th) => self.hostname != *th,
                None => true,
            }
        {
            match  serde_json::to_string(&MsgBody::Send(msg.to_owned())){
                Ok(send_msg) => {
                    let mut redis = self
                        .redis
                        .get()
                        .await?;
                    let res: Result<(), _> = redis.publish(self.channel_name, send_msg).await;
                    if let Err(err) = res {
                        warn!("notify redis clear cache fail :{}", err);
                        return Err(RemoteNotifyError::Redis(err));
                    };
                },
                Err(err) => {
                    warn!("create notify message fail :{}", err);
                    return Err(RemoteNotifyError::System(err.to_string()));
                },
            };
           
        }
        if local_exe_type==LocalExecType::LocalExec//本机直接执行
            && match &target_host {
                Some(th) => self.hostname == *th,//目标为本机
                None => true,//目标为所有机器
            }
        {
            for tmp in self.run_list.read().await.iter() {
                if tmp.msg_type() == msg_type {
                    match tmp.run(msg).await {
                        Ok(data) => {
                            out.push(MsgResultBody {
                                data: Ok(data),
                                from_host: self.hostname.to_owned(),
                                reply_id: msg_id,
                            });
                        }
                        Err(err) => {
                            warn!("run task error:{}", err);
                            out.push(MsgResultBody {
                                data: Err(err),
                                from_host: self.hostname.to_owned(),
                                reply_id: msg_id,
                            });
                        }
                    }
                    break;
                }
            }
            //目标仅为本机，且在本机执行，直接返回本机执行结果
            if match &target_host {
                Some(th) => self.hostname == *th, //目标为本机
                None => false,                    //目标为所有机器
            } {
                return Ok(out);
            }
        }
        //有远程节点执行且要等待各节点的执行结果
        if let Some(wait) = reply_wait {
            let (tx, rx) = mpsc::channel(wait.max_node);
            self.callback.lock().await.insert(msg_id, tx);
            // 调用 foo，并传入信道的发送端，并使用 timeout 包裹它
            let ret = Self::receiver_data(rx, wait.timeout, wait.max_node).await;
            self.callback.lock().await.remove(&msg_id);
            return ret;
        }
        Ok(out)
    }
    async fn receiver_data(
        mut rx: mpsc::Receiver<MsgResultBody>,
        time: u64,       //超时
        max_node: usize, //节点数
    ) -> Result<Vec<MsgResultBody>, RemoteNotifyError> {
        // 创建一个空的向量，用来存储接收到的消息
        let mut messages = Vec::new();
        // 设置一个超时时间，比如 500 毫秒
        let timeout_duration = Duration::from_millis(time);
        let mut ret_num = 0;
        // 使用 loop 不断从信道接收数据
        loop {
            // 使用 timeout 包裹 rx.recv()，并使用 select! 宏等待结果或超时
            tokio::select! {
                // 如果成功接收到一条消息，就将其添加到向量中
                Ok(msg) = tokio::time::timeout(timeout_duration, rx.recv()) => {
                    match msg{
                        Some(msg_data) =>{
                            messages.push(msg_data);
                            ret_num+=1;
                            if ret_num>=max_node{
                                break;
                            }
                        },
                        None => {
                            continue;
                        },
                    }
                }
                // 如果超时了，就跳出循环
                else => {
                    if ret_num==0{
                        return Err(RemoteNotifyError::RemoteTimeOut)
                    }
                    break;
                }
            }
        }
        // 返回向量作为结果
        Ok(messages)
    }
    async fn listen_run(&self, msg: MsgBody) -> Result<(), String> {
        match msg {
            MsgBody::Send(send) => {
                if send.ignore_local
                    && match &send.target_host {
                        Some(thost) => *thost != self.hostname,
                        None => true,
                    }
                {//该消息已被标记为本机忽略
                    info!("ignore target self msg :{}", send.data.to_string());
                    return Ok(());
                }
                for tmp in self.run_list.read().await.iter() {
                    if tmp.msg_type() == send.msg_type {
                        let reply = send.reply;
                        let reply_id = send.id;
                        let res = tmp.run(send).await;
                        if let Err(ref err) = res {
                            info!("run notify fail: {}", err);
                        }
                        if reply {
                            let msg = MsgResultBody {
                                data: res,
                                reply_id,
                                from_host: self.hostname.clone(),
                            };
                            match serde_json::to_string(&MsgBody::Result(msg)){
                                Ok(send_msg) => {
                                    let mut redis = self
                                        .redis
                                        .get()
                                        .await
                                        .map_err(|e| e.to_string())?;
                                    let res: Result<(), _> = redis.publish(self.channel_name, send_msg).await;
                                    if let Err(err) = res {
                                        warn!("reply exec to redis fail :{}", err);
                                    };
                                },
                                Err(err) => {
                                    warn!("crate notify message fail :{}", err);
                                },
                            }
                           
                        }
                        return Ok(());
                    }
                }
                Err(format!("msg type not suport:{}",send.data))
            }
            MsgBody::Result(result) => {
                //其他机器返回的执行结果消息
                if let Some(tx) = self.callback.lock().await.get(&result.reply_id) {
                    if let Err(err) = tx.send(result).await {
                        info!("callback error :{}", err);
                    }
                }else{
                    //等待的回调通道已经被超时删除
                    info!("callback is timeout,ignore:{:?}", result.data);
                }
                Ok(())
            }
        }
    }
    pub async fn push_run(&self,task:Box<dyn RemoteTask>){
        self.run_list.write().await.push(task);
    }
    pub async fn listen(&self) {
        loop {
            match self.app_core.create_redis_client() {
                Ok(redis_client) => {
                    let con_res = redis_client.get_async_connection().await;
                    match con_res {
                        Ok(con) => {
                            let mut pubsub = con.into_pubsub();
                            let res = pubsub.subscribe(self.channel_name).await;
                            if let Err(err) = res {
                                error!("listen sub fail :{}", err);
                                continue;
                            } else {
                                info!("listen remote channel succ:{}", self.channel_name);
                            }
                            let mut pubsub_stream = pubsub.on_message();
                            loop {
                                match pubsub_stream.next().await {
                                    Some(msg) => match msg.get_payload::<String>() {
                                        Ok(pubsub_msg) => {
                                            debug!("recv msg:{}", pubsub_msg);
                                            match serde_json::from_str::<MsgBody>(&pubsub_msg) {
                                                Ok(msg_body) => {
                                                    if let Err(err) =
                                                        self.listen_run(msg_body).await
                                                    {
                                                        warn!("run remote msg fail :{}", err);
                                                    }
                                                }
                                                Err(err) => {
                                                    error!("parse payload fail :{}", err);
                                                }
                                            }
                                        }
                                        Err(err) => {
                                            error!("read payload fail :{}", err);
                                        }
                                    },
                                    None => {
                                        continue;
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
                    warn!("create remote notify listen client fail:{}", err);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }
}
