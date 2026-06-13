//! Yaque 磁盘持久化队列实现
//!
//! 基于 `yaque` crate 实现的磁盘持久化队列后端。
//! 消息写入即持久化（每条 send 自动 fsync），进程崩溃不丢消息。
//!
//! # ACK 语义
//!
//! yaque 的 `RecvGuard` 提供事务性消费：
//! - `pop_blocking` 收到消息后立即 `commit()`，消息从磁盘队列中移除
//! - `nack` 时通过重新 `send` 将消息放回队列（模拟 requeue）
//!
//! # 性能特征
//!
//! - push 路径包含文件写入 + fsync，延迟约 0.1-1ms（取决于磁盘）
//! - 适合万级/秒吞吐，不适合几十万级/秒（此时应使用 RabbitMQ 或 MemoryQueue）
//! - 优势：零依赖外部服务，单机部署，崩溃安全

use async_trait::async_trait;
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use super::{QueueBackend, QueueMessage, QueueMetrics, QueueResult};

/// Yaque 磁盘队列配置
#[derive(Debug, Clone)]
pub struct YaqueQueueConfig {
    /// 队列数据存储目录路径
    ///
    /// yaque 会在该目录下创建段文件存储消息。
    /// 目录不存在时自动创建。
    pub queue_dir: String,
}

/// Yaque ACK/NACK 凭证
///
/// 持有消息的序列化字节，用于 `nack` 时重新入队。
pub struct YaqueDeliveryToken {
    payload: Vec<u8>,
}

/// Yaque 磁盘持久化队列
///
/// 基于 yaque 的 SPSC（单生产者单消费者）文件队列。
/// Sender/Receiver 通过 `Arc<Mutex<>>` 共享以支持 `&self` 方法。
///
/// # 类型参数
///
/// - `M`: 消息类型，自动满足 `QueueMessage`
pub struct YaqueQueue<M: QueueMessage> {
    sender: Arc<Mutex<yaque::Sender>>,
    receiver: Arc<Mutex<yaque::Receiver>>,
    metrics: Arc<QueueMetrics>,
    _phantom: PhantomData<M>,
}

#[async_trait]
impl<M: QueueMessage> QueueBackend for YaqueQueue<M> {
    type Message = M;
    type Config = YaqueQueueConfig;
    type DeliveryToken = YaqueDeliveryToken;

    async fn new(config: Self::Config) -> QueueResult<Self>
    where
        Self: Sized,
    {
        let (sender, receiver) = yaque::channel(&config.queue_dir)?;
        Ok(Self {
            sender: Arc::new(Mutex::new(sender)),
            receiver: Arc::new(Mutex::new(receiver)),
            metrics: Arc::new(QueueMetrics::new()),
            _phantom: PhantomData,
        })
    }

    /// 推送消息到磁盘队列
    ///
    /// 消息序列化为 JSON 后写入 yaque 段文件并 fsync。
    /// 返回 Ok 即表示消息已持久化。
    async fn push(&self, message: Self::Message) -> QueueResult<()> {
        let payload = serde_json::to_vec(&message)?;
        self.sender.lock().await.send(&payload).await?;
        self.metrics.increment_pushed();
        Ok(())
    }

    /// 阻塞弹出（带超时）
    ///
    /// 从 yaque 接收消息，反序列化后立即 commit（从磁盘移除）。
    /// DeliveryToken 持有原始字节，用于 nack 时重新入队。
    async fn pop_blocking(
        &self,
        timeout: Duration,
    ) -> QueueResult<Option<(Self::Message, Self::DeliveryToken)>> {
        let mut receiver = self.receiver.lock().await;
        let delay = Box::pin(tokio::time::sleep(timeout));

        match receiver.recv_timeout(delay).await {
            Ok(Some(guard)) => {
                let payload = guard.to_vec();
                if let Err(e) = guard.commit() {
                    log::warn!("Yaque commit failed: {:?}, message may be re-delivered on restart", e);
                }
                let message: M = serde_json::from_slice(&payload)?;
                Ok(Some((message, YaqueDeliveryToken { payload })))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// ack：消息已在 pop_blocking 中 commit，此处仅更新指标
    async fn ack(&self, _token: Self::DeliveryToken) -> QueueResult<()> {
        self.metrics.increment_consumed();
        Ok(())
    }

    /// nack：将消息重新推回队列（模拟 requeue）
    ///
    /// yaque 原生不支持 requeue（commit 后消息已从段文件移除），
    /// 因此通过重新 send 实现。
    async fn nack(&self, token: Self::DeliveryToken, _requeue: bool) -> QueueResult<()> {
        self.sender.lock().await.send(&token.payload).await?;
        self.metrics.increment_failed();
        Ok(())
    }

    async fn size(&self) -> usize {
        self.metrics.current_size()
    }

    async fn shutdown(&self, _timeout: Duration) -> QueueResult<()> {
        // Sender/Receiver 在 drop 时自动保存状态并释放文件锁
        Ok(())
    }
}
