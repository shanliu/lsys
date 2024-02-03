use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;

use deadpool_redis::PoolError;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::oneshot::{self, Receiver, Sender};
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

use redis::AsyncCommands;
use tracing::{debug, error, info, warn};

use crate::{fluent_message, AppCore, FluentMessage};

#[derive(Debug)]
pub enum WaitNotifyError {
    System(FluentMessage),
    Redis(redis::RedisError),
    RedisPool(PoolError),
    TimeOut,
}

impl Display for WaitNotifyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub type WaitNotifyResult = Result<bool, String>;

pub trait WaitItem {
    fn eq(&self, other: &Self) -> bool;
}

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
        let mut redis = self.redis.get().await.map_err(WaitNotifyError::RedisPool)?;
        let res: Result<(), _> = redis
            .lpush(
                &channel_name,
                json!(ListenMsgBody { data, res }).to_string(),
            )
            .await;

        if let Err(err) = res {
            warn!("notify redis fail:{}", err);
            return Err(WaitNotifyError::Redis(err));
        };
        let res: Result<(), _> = redis
            .expire(&channel_name, (self.clear_timeout * 2) as usize)
            .await;
        if let Err(err) = res {
            info!("notify redis set time out fail:{}", err);
        };
        Ok(())
    }
    pub async fn listen(&self) {
        loop {
            match self.app_core.create_redis_client() {
                Ok(redis_client) => {
                    let con_res = redis_client.get_async_connection().await;
                    match con_res {
                        Ok(mut redis) => {
                            debug!("notify is connect");
                            let channel_name = self.redis_channel_name(
                                hostname::get()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .as_ref(),
                            );
                            //用list 不用subscribe 这里监听重启后也可以接着处理
                            let msg: Result<(String, String), _> = redis
                                .blpop(&channel_name, self.clear_timeout as usize)
                                .await;

                            match msg {
                                Ok(pubsub_msg) => {
                                    debug!("recv msg:{:?}", pubsub_msg);
                                    match serde_json::from_str::<ListenMsgBody<T>>(&pubsub_msg.1) {
                                        Ok(msg_body) => {
                                            if let Err(err) = self.listen_run(msg_body).await {
                                                warn!("run remote msg fail :{}", err);
                                            }
                                        }
                                        Err(err) => {
                                            error!("parse payload fail :{}", err);
                                        }
                                    }
                                }
                                Err(err) => {
                                    if err.kind() == redis::ErrorKind::TypeError || err.is_timeout()
                                    {
                                        self.listen_clear().await;
                                    } else {
                                        warn!("read notify list error:{}", err);
                                        sleep(Duration::from_secs(1)).await;
                                    }
                                    continue;
                                }
                            };
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
    async fn listen_run(&self, msg: ListenMsgBody<T>) -> Result<(), String> {
        let tmp = std::mem::take(&mut *self.sender_data.lock().await);
        // println!("{:?}", tmp.iter().map(|e| &e.0).collect::<Vec<&T>>());
        let (tmp1, tmp2) = tmp
            .into_iter()
            .partition(|(a, b)| a.eq(&msg.data) || b.is_closed());
        (*self.sender_data.lock().await) = tmp2;

        for tmp in tmp1 {
            if tmp.0.eq(&msg.data) {
                if let Err(err) = tmp.1.send(msg.res.to_owned()) {
                    return Err(format!("notify channel send fail,data:{:?}", err));
                }
                return Ok(());
            } else if tmp.1.is_closed() {
                info!("notify channel is close[run] {:?}", tmp.0);
            }
        }
        Err(format!("unkown notify {:?}", msg.data))
    }
    async fn listen_clear(&self) {
        let tmp = std::mem::take(&mut *self.sender_data.lock().await);
        let (tmp1, tmp2) = tmp.into_iter().partition(|(_, b)| b.is_closed());
        (*self.sender_data.lock().await) = tmp2;
        for tmp in tmp1 {
            info!("notify channel is close[chear] {:?}", tmp.0);
        }
    }
}

#[tokio::test]
async fn test_listen_notify() {
    let app_core = AppCore::init(
        &format!("{}/../examples/lsys-actix-web", env!("CARGO_MANIFEST_DIR")),
        &format!(
            "{}/../examples/lsys-actix-web/config",
            env!("CARGO_MANIFEST_DIR")
        ),
        None,
    )
    .await
    .unwrap();
    impl crate::WaitItem for u64 {
        fn eq(&self, other: &Self) -> bool {
            *self == *other
        }
    }
    let notify = std::sync::Arc::new(WaitNotify::<u64>::new(
        "sms",
        app_core.create_redis().await.unwrap(),
        Arc::new(app_core),
        10,
    ));

    let tmp = notify.clone();
    tokio::spawn(async move {
        tmp.listen().await;
    });
    let wait = notify.wait(11).await;
    notify
        .notify(
            hostname::get()
                .unwrap_or_default()
                .to_string_lossy()
                .as_ref(),
            11,
            Err("bad".to_string()),
        )
        .await
        .unwrap();
    let data = notify.wait_timeout(wait).await.unwrap();
    assert_eq!(data, Err("bad".to_string()))
}
