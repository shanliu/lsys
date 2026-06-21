// 基于REDIS队列实现,任意一个主机派发出一个任务到队列,并监听结果,由监听队列的节点执行后,将结果返回到派发任务主机
// 当前用例:
// 执行短信发送,完成发送后,将结果返回到申请发送短信的主机
use std::fmt::Debug;
use std::sync::Arc;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;

use tokio::sync::Mutex;
use tokio::sync::oneshot::{self, Receiver, Sender};
use tokio::time::{Duration, sleep};

use redis::AsyncCommands;
use tracing::{Instrument, debug, error, info, warn};

use crate::app_core::AppCore;
use crate::fluent_message;
use crate::fluents::IntoFluentMessage;
mod result;
pub use result::*;
pub trait WaitItem {
    fn eq(&self, other: &Self) -> bool;
}

// 处理流程：

// 监听端：
// 建立`一次性channel`,监听超时跟该一次性channel接收端
// `一次性channel`发送端 标识T 加入待返回 数组sender_data
//
// 监听redis： 【主机名】队列，加超时，返回-》读取队列
// 	检查 数组sender_data 推送结果到 `一次性channel`发送端
// 	并清理已被超时关闭的 `一次性channel`发送端
//
// 推送端：
// 把 标识T + 消息推入【主机名】队列

pub struct WaitNotify<T: WaitItem + Serialize + DeserializeOwned + Debug> {
    channel_name: String,
    sender_data: Mutex<Vec<(T, Sender<WaitNotifyResult>)>>,
    app_core: Arc<AppCore>,
    redis: deadpool_redis::Pool,
    clear_timeout: u8,
}

//消息内容
#[derive(Serialize, Deserialize, Clone)]
pub struct ListenMsgBody<T> {
    data: T,
    res: WaitNotifyResult,
}

impl<T: WaitItem + Serialize + DeserializeOwned + Debug> WaitNotify<T> {
    pub fn new(
        channel_name: &str,
        redis: deadpool_redis::Pool,
        app_core: Arc<AppCore>,
        clear_timeout: u8,
    ) -> Self {
        WaitNotify::<T> {
            channel_name: channel_name.to_owned(),
            sender_data: Mutex::new(vec![]),
            app_core,
            redis,
            clear_timeout,
        }
    }
    fn redis_channel_name(&self, host: &str) -> String {
        format!("{}-{}", self.channel_name, host)
    }
    pub async fn wait(
        &self,
        data: T, //消息数据
    ) -> Receiver<WaitNotifyResult> {
        let (tx, rx) = oneshot::channel::<WaitNotifyResult>();
        self.sender_data.lock().await.push((data, tx));
        debug!("notify wait {} add listen wait", self.channel_name);
        rx
    }
    pub async fn wait_timeout(
        &self,
        receiver: Receiver<WaitNotifyResult>,
    ) -> Result<WaitNotifyResult, WaitNotifyError> {
        match tokio::time::timeout(Duration::from_secs(self.clear_timeout as u64), receiver).await {
            Ok(Ok(data)) => Ok(data),
            Ok(Err(err)) => Err(WaitNotifyError::System(fluent_message!(
                "wait-recv-fail",
                err
            ))),
            Err(_) => Err(WaitNotifyError::TimeOut),
        }
    }
    pub async fn notify(
        &self,
        host: &str,
        data: T,
        res: WaitNotifyResult,
    ) -> Result<(), WaitNotifyError> {
        let channel_name = self.redis_channel_name(host);
        debug!(
            "sender wait {} notify :{:?} [host:{}]",
            channel_name, data, host
        );
        let mut redis = self.redis.get().await.map_err(WaitNotifyError::RedisPool)?;
        let res: Result<(), _> = redis
            .lpush(
                &channel_name,
                json!(ListenMsgBody { data, res }).to_string(),
            )
            .await;

        if let Err(err) = res {
            warn!("notify wait {} redis fail:{}", channel_name, err);
            return Err(WaitNotifyError::Redis(err));
        };
        let res: Result<(), _> = redis
            .expire(&channel_name, (self.clear_timeout * 2) as i64)
            .await;
        if let Err(err) = res {
            info!(
                "notify wait {} redis set time out fail:{}",
                channel_name, err
            );
        };
        Ok(())
    }
    pub async fn listen(&self, cancel_token: tokio_util::sync::CancellationToken) {
        loop {
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    info!("notify wait {} listen cancelled", self.channel_name);
                    return;
                }
                result = self.listen_once() => {
                    if result.is_none() {
                        // listen_once 返回 None 表示需要退出
                        return;
                    }
                }
            }
        }
    }

    /// 单次监听循环（提取自原 listen 方法）
    /// 返回 Some(()) 表示继续循环，None 表示应退出
    async fn listen_once(&self) -> Option<()> {
        match crate::app_core::create_redis_client(self.app_core.as_ref()).await {
            Ok(redis_client) => {
                // redis 1.0.x 默认 response_timeout 为 500ms，会导致 blpop 等阻塞命令立即超时返回
                // 设置为 blpop 超时 + 5s 缓冲，既允许 blpop 正常等待，又能在连接异常时兜底超时
                let blpop_conn_config = redis::AsyncConnectionConfig::new()
                    .set_response_timeout(Some(std::time::Duration::from_secs(
                        self.clear_timeout as u64 + 5,
                    )));
                let con_res = redis_client
                    .get_multiplexed_async_connection_with_config(&blpop_conn_config)
                    .await;
                match con_res {
                    Ok(mut redis) => {
                        let channel_name = self.redis_channel_name(
                            hostname::get()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .as_ref(),
                        );
                        //用list 不用subscribe 这里监听重启后也可以接着处理
                        // 使用 Option 包裹，BLPOP 超时返回 nil 时解析为 None 而非报错
                        let msg: Result<Option<(String, String)>, _> =
                            redis.blpop(&channel_name, self.clear_timeout as f64).await;

                        match msg {
                            Ok(Some(pubsub_msg)) => {
                                debug!(
                                    "notify  wait {} sender wait msg:{:?}",
                                    channel_name, pubsub_msg
                                );
                                match serde_json::from_str::<ListenMsgBody<T>>(&pubsub_msg.1) {
                                    Ok(msg_body) => {
                                        let task_id = crate::utils::rand_str(crate::utils::RandType::LowerHex, 8);
                                        async {
                                            if let Err(err) = self.listen_run(msg_body).await {
                                                warn!(
                                                    "notify  wait {} run remote msg fail :{}",
                                                    channel_name, err
                                                );
                                            }
                                        }
                                        .instrument(tracing::info_span!(
                                            "background_task",
                                            task = "listen-notify-run",
                                            task_id = task_id,
                                            channel = %channel_name
                                        ))
                                        .await;
                                    }
                                    Err(err) => {
                                        error!(
                                            "notify  wait {} parse payload fail :{}",
                                            channel_name, err
                                        );
                                    }
                                }
                            }
                            Ok(None) => {
                                // BLPOP 超时，执行清理逻辑
                                self.listen_clear().await;
                                return Some(());
                            }
                            Err(err) => {
                                if err.is_timeout() {
                                    self.listen_clear().await;
                                } else {
                                    warn!(
                                        "notify  wait {} read notify list error:{}",
                                        channel_name, err
                                    );
                                    sleep(Duration::from_secs(1)).await;
                                }
                                return Some(());
                            }
                        };
                    }
                    Err(err) => {
                        error!("notify clear conn redis:{}", err);
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }
            Err(err) => {
                warn!(
                    "notify create remote notify listen client fail:{}",
                    err.to_fluent_message().default_format()
                );
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
        Some(())
    }
    async fn listen_run(&self, msg: ListenMsgBody<T>) -> Result<(), String> {
        let mut lock_data = self.sender_data.lock().await;
        let tmp = std::mem::take(&mut *lock_data);
        debug!(
            "notify  wait {}: all list count {}",
            self.channel_name,
            tmp.len()
        );
        // println!("{:?}", tmp.iter().map(|e| &e.0).collect::<Vec<&T>>());
        let (tmp1, tmp2) = tmp.into_iter().partition(|(a, _)| {
            debug!(
                "notify  wait {}: list data:{:?},notify data:{:?}",
                self.channel_name, &a, &msg.data
            );
            a.eq(&msg.data)
        });
        *lock_data = tmp2;
        drop(lock_data);
        debug!(
            "notify  wait {}: find list count {}",
            self.channel_name,
            tmp1.len()
        );
        let succ_find = !tmp1.is_empty();
        for (tmp_data, tmp_res) in tmp1 {
            debug!(
                "notify wait {}: item [{:?} ={:?}]",
                self.channel_name, tmp_data, msg.data
            );
            if tmp_res.is_closed() {
                info!(
                    "notify wait {}: channel is close[run] {:?}",
                    self.channel_name, tmp_data
                );
            } else if let Err(err) = tmp_res.send(msg.res.to_owned()) {
                warn!(
                    "notify wait {}: channel send fail,data: {:?} error:{:?}",
                    self.channel_name, tmp_data, err
                );
            }
        }
        if !succ_find {
            info!(
                "notify wait {}: data [{:?}] not match any wait",
                self.channel_name, msg.data
            );
        }
        Ok(())
    }
    async fn listen_clear(&self) {
        let mut lock_data = self.sender_data.lock().await;
        let tmp = std::mem::take(&mut *lock_data);
        let (tmp1, tmp2) = tmp.into_iter().partition(|(_, b)| b.is_closed());
        *lock_data = tmp2;
        drop(lock_data);
        for tmp in tmp1 {
            info!(
                "notify wait {} channel is close[chear] {:?}",
                self.channel_name, tmp.0
            );
        }
    }
}

#[tokio::test]
async fn test_listen_notify() {
    let app_core = AppCore::new(
        &format!("{}/../examples/lsys-actix-web", env!("CARGO_MANIFEST_DIR")),
        &format!(
            "{}/../examples/lsys-actix-web/config",
            env!("CARGO_MANIFEST_DIR")
        ),
        "app",
        None,
        None,
    )
    .await
    .unwrap();
    #[derive(Serialize, Debug, Deserialize)]
    struct TmpData(u64);
    impl crate::listen_notify::WaitItem for TmpData {
        fn eq(&self, other: &Self) -> bool {
            self.0 == other.0
        }
    }
    let notify = std::sync::Arc::new(WaitNotify::<TmpData>::new(
        "sms",
        crate::app_core::create_redis_pool(&app_core).await.unwrap(),
        Arc::new(app_core),
        10,
    ));

    let tmp = notify.clone();
    let cancel_token = tokio_util::sync::CancellationToken::new();
    let listen_token = cancel_token.clone();
    tokio::spawn(async move {
        tmp.listen(listen_token).await;
    });
    let wait = notify.wait(TmpData(11)).await;
    notify
        .notify(
            hostname::get()
                .unwrap_or_default()
                .to_string_lossy()
                .as_ref(),
            TmpData(11),
            Err("bad".to_string()),
        )
        .await
        .unwrap();
    let data = notify.wait_timeout(wait).await.unwrap();
    cancel_token.cancel();
    assert_eq!(data, Err("bad".to_string()))
}
