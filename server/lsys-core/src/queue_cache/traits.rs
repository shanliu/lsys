//! 核心 Trait 定义
//!
//! 定义了队列缓存层的核心抽象接口。

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::QueueResult;

/// 可序列化消息 trait
///
/// 凡是实现了 `Serialize + Deserialize + Send + Sync + 'static` 的类型
/// 均自动满足此 trait，无需手动 `impl`。
///
/// # 示例
///
/// ```ignore
/// use serde::{Serialize, Deserialize};
///
/// // 只需 derive，无需手动 impl QueueMessage
/// #[derive(Serialize, Deserialize)]
/// struct UserEvent {
///     user_id: i64,
///     action: String,
/// }
/// ```
pub trait QueueMessage: Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static {}

/// 为满足约束的所有类型自动实现 QueueMessage
impl<T: Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static> QueueMessage for T {}

/// 消息处理器 trait
///
/// 实现此 trait 以定义如何处理特定类型的消息。
///
/// # 示例
///
/// ```ignore
/// use async_trait::async_trait;
/// use lsys_core::queue_cache::{MessageHandler, QueueResult};
///
/// struct EmailHandler;
///
/// #[async_trait]
/// impl MessageHandler<EmailMessage> for EmailHandler {
///     async fn handle(&self, message: EmailMessage) -> QueueResult<()> {
///         // 处理失败时直接返回 Err，由消费者执行 nack
///         send_email(&message).await?;
///         Ok(())
///     }
/// }
/// ```
#[async_trait]
pub trait MessageHandler<M: QueueMessage>: Send + Sync {
    /// 处理消息
    ///
    /// - 返回 `Ok(())` 时消费者调用 `ack`，消息被确认消费
    /// - 返回 `Err(_)` 时消费者调用 `nack`，由后端决定丢弃或重入队列
    ///
    /// 重试逻辑应在此方法内自行实现。
    async fn handle(&self, message: M) -> QueueResult<()>;
}

/// 队列后端 trait
///
/// 不同的队列实现（内存队列、RabbitMQ 等）通过实现此 trait 提供统一接口。
///
/// # ACK 语义
///
/// `pop` / `pop_blocking` 返回消息时同时返回一个 `DeliveryToken`。
/// 调用方在处理完消息后**必须**调用 `ack` 或 `nack`：
/// - `ack(token)` — 消息处理成功，从队列中永久移除
/// - `nack(token, requeue)` — 消息处理失败，`requeue=true` 表示重入队列
///
/// # 注意
///
/// `MemoryQueue` 的 `DeliveryToken` 为 `()`，`nack` 仅记录指标，不支持 requeue
/// （内存队列无持久化，消息在进程退出后丢失）。
/// 若需要可靠的 requeue 语义，请使用 `RabbitMQQueue`。
#[async_trait]
pub trait QueueBackend: Send + Sync {
    /// 队列中传输的消息类型
    type Message: QueueMessage;

    /// 创建队列所需的配置类型
    type Config;

    /// ACK / NACK 凭证
    ///
    /// 从 `pop_blocking` 返回的凭证，用于后续的 `ack` / `nack` 调用。
    type DeliveryToken: Send + 'static;

    /// 初始化队列实例
    async fn new(config: Self::Config) -> QueueResult<Self>
    where
        Self: Sized;

    /// 推送消息到队列
    ///
    /// - `Err(QueueCacheError::QueueFull)` — 队列已满
    /// - `Err(QueueCacheError::QueueClosed)` — 队列已关闭
    async fn push(&self, message: Self::Message) -> QueueResult<()>;

    /// 阻塞弹出（带超时）
    ///
    /// 队列为空时阻塞等待，超时后返回 `Ok(None)`。
    /// 返回的 `DeliveryToken` 须在处理后调用 `ack` 或 `nack`。
    async fn pop_blocking(
        &self,
        timeout: Duration,
    ) -> QueueResult<Option<(Self::Message, Self::DeliveryToken)>>;

    /// 确认消息已成功处理，从队列中永久移除
    async fn ack(&self, token: Self::DeliveryToken) -> QueueResult<()>;

    /// 标记消息处理失败
    ///
    /// - `requeue = true` — 将消息重新放回队列（仅部分后端支持）
    /// - `requeue = false` — 丢弃消息（或投递到 DLQ，取决于后端配置）
    async fn nack(&self, token: Self::DeliveryToken, requeue: bool) -> QueueResult<()>;

    /// 当前队列中待处理消息数量（估算）
    async fn size(&self) -> usize;

    /// 优雅关闭队列
    ///
    /// 超时未完成时返回 `Err(QueueCacheError::Timeout)`。
    async fn shutdown(&self, timeout: Duration) -> QueueResult<()>;
}
