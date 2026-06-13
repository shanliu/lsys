//! 内存队列实现
//!
//! 基于 `flume` 实现的高并发内存队列。
//! 相比 `tokio::sync::mpsc`，`flume` 是真正的 MPMC（多生产者多消费者）
//! channel，多个消费者可以并发无锁地弹出消息，无需外部 Mutex/RwLock。
//!
//! # ACK 语义说明
//!
//! 内存队列的 `DeliveryToken` 为 `()`，`ack` 为空操作，`nack` 仅记录失败指标。
//! 内存队列**不支持 requeue**：消息一旦被弹出，若处理失败则丢弃。
//! 这是设计上的取舍——内存队列本身无持久化，进程重启消息就会丢失，
//! 提供 requeue 的意义有限。若需要可靠的 requeue，请使用 `RabbitMQQueue`。

use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;

use super::{QueueBackend, QueueCacheError, QueueMessage, QueueMetrics, QueueResult};

/// 内存队列配置
#[derive(Debug, Clone)]
pub struct MemoryQueueConfig {
    /// 队列最大容量（消息数量）
    ///
    /// 队列满时，`push` 将异步阻塞，直到有空间可用（背压机制）。
    pub capacity: usize,

    /// 是否启用监控统计
    pub enable_metrics: bool,
}

/// 内存队列实现
///
/// 使用 `flume::bounded` 作为底层通道，支持多生产者多消费者并发操作。
///
/// # 类型参数
///
/// - `M`: 消息类型，自动满足 `QueueMessage` 的任何 `Serialize + Deserialize + Send + Sync + 'static` 类型
pub struct MemoryQueue<M: QueueMessage> {
    sender: flume::Sender<M>,
    receiver: flume::Receiver<M>,
    enable_metrics: bool,
    metrics: Arc<QueueMetrics>,
}

#[async_trait]
impl<M: QueueMessage> QueueBackend for MemoryQueue<M> {
    type Message = M;
    type Config = MemoryQueueConfig;
    /// 内存队列的 ACK 凭证为空类型
    ///
    /// `ack(())` 无操作；`nack((), requeue)` 仅记录失败指标，不支持 requeue。
    type DeliveryToken = ();

    async fn new(config: Self::Config) -> QueueResult<Self> {
        let (sender, receiver) = flume::bounded(config.capacity);
        Ok(Self {
            sender,
            receiver,
            enable_metrics: config.enable_metrics,
            metrics: Arc::new(QueueMetrics::new()),
        })
    }

    async fn push(&self, message: Self::Message) -> QueueResult<()> {
        self.sender
            .send_async(message)
            .await
            .map_err(|_| QueueCacheError::QueueClosed)?;

        if self.enable_metrics {
            self.metrics.increment_pushed();
        }
        Ok(())
    }

    async fn pop_blocking(
        &self,
        timeout: Duration,
    ) -> QueueResult<Option<(Self::Message, Self::DeliveryToken)>> {
        match tokio::time::timeout(timeout, self.receiver.recv_async()).await {
            Ok(Ok(msg)) => {
                if self.enable_metrics {
                    self.metrics.increment_consumed();
                }
                Ok(Some((msg, ())))
            }
            Ok(Err(_)) => Err(QueueCacheError::QueueClosed),
            Err(_) => Ok(None), // timeout
        }
    }

    /// ack：内存队列无需操作，消息已从 channel 中移除
    async fn ack(&self, _token: Self::DeliveryToken) -> QueueResult<()> {
        Ok(())
    }

    /// nack：记录失败指标；不支持 requeue（内存队列无持久化）
    async fn nack(&self, _token: Self::DeliveryToken, _requeue: bool) -> QueueResult<()> {
        if self.enable_metrics {
            self.metrics.increment_failed();
        }
        Ok(())
    }

    async fn size(&self) -> usize {
        if self.enable_metrics {
            self.metrics.current_size()
        } else {
            self.receiver.len()
        }
    }

    async fn shutdown(&self, _timeout: Duration) -> QueueResult<()> {
        // sender/receiver 会在 MemoryQueue drop 时自动关闭
        Ok(())
    }
}

impl<M: QueueMessage> MemoryQueue<M> {
    /// 获取监控统计
    pub fn metrics(&self) -> Arc<QueueMetrics> {
        Arc::clone(&self.metrics)
    }
}
