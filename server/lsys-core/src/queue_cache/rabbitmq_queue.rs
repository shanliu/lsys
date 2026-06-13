//! RabbitMQ 队列实现
//!
//! 基于 lapin crate 实现的 RabbitMQ 队列后端。
//! 提供消息持久化、ACK/NACK 确认机制和自动重连功能。

use async_trait::async_trait;
use lapin::{
    options::*,
    types::FieldTable,
    BasicProperties, Channel, Connection, ConnectionProperties,
};
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use super::{QueueBackend, QueueCacheError, QueueMessage, QueueMetrics, QueueResult};

/// RabbitMQ 队列配置
#[derive(Debug, Clone)]
pub struct RabbitMQConfig {
    /// RabbitMQ 连接 URL，格式：`amqp://username:password@host:port/vhost`
    pub connection_url: String,

    /// 队列名称（不存在时自动创建）
    pub queue_name: String,

    /// 是否持久化消息和队列（服务重启后不丢失）
    pub durable: bool,

    /// 连接失败时是否自动重试
    pub enable_retry: bool,

    /// 最大连接重试次数（`enable_retry = true` 时生效）
    pub max_retries: u32,

    /// 基础重试延迟（毫秒），使用指数退避
    pub retry_delay_ms: u64,
}

/// RabbitMQ ACK/NACK 凭证
///
/// 持有确认消息所需的通道引用和投递标签。
/// 从 `pop` / `pop_blocking` 返回后，**必须**调用 `ack` 或 `nack`，
/// 否则 RabbitMQ 会在连接断开后将消息重新入队。
pub struct RabbitMQDeliveryToken {
    channel: Channel,
    delivery_tag: u64,
}

/// RabbitMQ 队列实现
pub struct RabbitMQQueue<M: QueueMessage> {
    #[allow(dead_code)]
    connection: Arc<Connection>,
    channel: Arc<RwLock<Channel>>,
    config: RabbitMQConfig,
    metrics: Arc<QueueMetrics>,
    _phantom: PhantomData<M>,
}

impl<M: QueueMessage> RabbitMQQueue<M> {
    async fn connect_with_retry(config: &RabbitMQConfig) -> QueueResult<Connection> {
        let mut attempts = 0;
        let max_attempts = if config.enable_retry {
            config.max_retries + 1
        } else {
            1
        };

        loop {
            attempts += 1;
            match Connection::connect(&config.connection_url, ConnectionProperties::default()).await
            {
                Ok(connection) => {
                    if attempts > 1 {
                        tracing::info!(
                            "RabbitMQ connection established after {} attempts",
                            attempts
                        );
                    }
                    return Ok(connection);
                }
                Err(e) if attempts >= max_attempts => {
                    return Err(QueueCacheError::from(e));
                }
                Err(e) => {
                    let base_delay = Duration::from_millis(config.retry_delay_ms);
                    let delay = base_delay
                        .checked_mul(2u32.saturating_pow(attempts - 1))
                        .unwrap_or(Duration::from_secs(30))
                        .min(Duration::from_secs(30));
                    tracing::warn!(
                        "RabbitMQ connect failed ({}/{}): {}. Retrying in {:?}",
                        attempts,
                        max_attempts,
                        e,
                        delay
                    );
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }
}

#[async_trait]
impl<M: QueueMessage> QueueBackend for RabbitMQQueue<M> {
    type Message = M;
    type Config = RabbitMQConfig;
    /// RabbitMQ 的 ACK 凭证，持有通道引用和 delivery_tag
    type DeliveryToken = RabbitMQDeliveryToken;

    async fn new(config: Self::Config) -> QueueResult<Self>
    where
        Self: Sized,
    {
        let connection = Self::connect_with_retry(&config).await?;
        let channel = connection.create_channel().await?;

        channel
            .queue_declare(
                &config.queue_name,
                QueueDeclareOptions {
                    durable: config.durable,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await?;

        tracing::info!("RabbitMQ queue '{}' ready", config.queue_name);

        Ok(Self {
            connection: Arc::new(connection),
            channel: Arc::new(RwLock::new(channel)),
            config,
            metrics: Arc::new(QueueMetrics::new()),
            _phantom: PhantomData,
        })
    }

    async fn push(&self, message: Self::Message) -> QueueResult<()> {
        let payload = serde_json::to_vec(&message)?;
        let channel = self.channel.read().await;

        let properties = if self.config.durable {
            BasicProperties::default().with_delivery_mode(2)
        } else {
            BasicProperties::default()
        };

        channel
            .basic_publish(
                "",
                &self.config.queue_name,
                BasicPublishOptions::default(),
                &payload,
                properties,
            )
            .await?;

        self.metrics.increment_pushed();
        Ok(())
    }

    /// 阻塞弹出（带超时），使用 100ms 轮询
    ///
    /// 消息**不会**在此处被 ack，需调用方在处理后显式调用 `ack` 或 `nack`。
    async fn pop_blocking(
        &self,
        timeout: Duration,
    ) -> QueueResult<Option<(Self::Message, Self::DeliveryToken)>> {
        let start = std::time::Instant::now();
        let poll_interval = Duration::from_millis(100);

        loop {
            let channel = self.channel.read().await;
            match channel
                .basic_get(&self.config.queue_name, BasicGetOptions::default())
                .await
            {
                Ok(Some(delivery)) => {
                    let message: M = serde_json::from_slice(&delivery.data)?;
                    let token = RabbitMQDeliveryToken {
                        channel: channel.clone(),
                        delivery_tag: delivery.delivery_tag,
                    };
                    return Ok(Some((message, token)));
                }
                Ok(None) => {}
                Err(e) => return Err(QueueCacheError::from(e)),
            }
            if start.elapsed() >= timeout {
                return Ok(None);
            }
            tokio::time::sleep(poll_interval).await;
        }
    }

    /// 确认消息处理成功，RabbitMQ 将永久删除该消息
    async fn ack(&self, token: Self::DeliveryToken) -> QueueResult<()> {
        token
            .channel
            .basic_ack(token.delivery_tag, BasicAckOptions::default())
            .await?;
        self.metrics.increment_consumed();
        Ok(())
    }

    /// 标记消息处理失败
    ///
    /// - `requeue = true`：消息重新入队，等待再次消费
    /// - `requeue = false`：消息丢弃（若配置了 DLX，会投递到死信队列）
    async fn nack(&self, token: Self::DeliveryToken, requeue: bool) -> QueueResult<()> {
        token
            .channel
            .basic_nack(
                token.delivery_tag,
                BasicNackOptions {
                    requeue,
                    ..Default::default()
                },
            )
            .await?;
        self.metrics.increment_failed();
        Ok(())
    }

    async fn size(&self) -> usize {
        self.metrics.current_size()
    }

    async fn shutdown(&self, _timeout: Duration) -> QueueResult<()> {
        tracing::info!("Shutting down RabbitMQ queue '{}'", self.config.queue_name);
        let channel = self.channel.read().await;
        channel.close(200, "Normal shutdown").await?;
        Ok(())
    }
}
