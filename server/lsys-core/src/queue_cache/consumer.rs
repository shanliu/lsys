//! Queue Consumer
//!
//! 提供消息消费功能，支持 ACK/NACK 和优雅关闭。
//!
//! # 消费模型
//!
//! 消费者从队列后端弹出消息，交由 `MessageHandler` 处理：
//! - 处理成功（`Ok(())`）→ 调用后端 `ack`，消息被永久确认
//! - 处理失败（`Err(_)`）→ 调用后端 `nack(false)`，消息被丢弃（或由后端 DLQ 处理）
//!
//! 重试逻辑应在 `MessageHandler::handle` 内部自行实现。
//!
//! # 示例
//!
//! ```ignore
//! use std::sync::Arc;
//! use lsys_core::queue_cache::{QueueConsumer, MemoryQueue};
//!
//! let queue = MemoryQueue::new(config).await?;
//! let consumer = QueueConsumer::new(queue);
//! let handler = Arc::new(MyHandler);
//!
//! consumer.listen(handler, Box::new(|result| {
//!     if let Err(e) = result {
//!         eprintln!("处理失败: {:?}", e);
//!     }
//! })).await?;
//! ```

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use super::{MessageHandler, QueueBackend, QueueResult};

/// 消息处理结果回调类型
///
/// 每次消息处理完成（无论成功还是失败）后触发。
pub type ListenCallback = Box<dyn Fn(QueueResult<()>) + Send + Sync>;

/// 队列消费者
///
/// 持续从队列后端弹出消息并使用 `MessageHandler` 处理。
/// 每条消息处理后自动执行 `ack`（成功）或 `nack`（失败）。
///
/// # 类型参数
///
/// - `B`: 实现 `QueueBackend` 的队列后端类型
pub struct QueueConsumer<B: QueueBackend> {
    backend: Arc<B>,
    running: Arc<AtomicBool>,
}

impl<B: QueueBackend> QueueConsumer<B> {
    /// 创建消费者
    pub fn new(backend: B) -> Self {
        Self {
            backend: Arc::new(backend),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// 启动消费循环（阻塞直到 `shutdown` 被调用）
    ///
    /// 每次轮询等待最多 1 秒；若队列为空则继续等待，不消耗 CPU。
    ///
    /// # 参数
    ///
    /// - `handler`: 消息处理器（`Arc` 包装，支持多消费者共享）
    /// - `callback`: 每条消息处理后的结果回调
    pub async fn listen<H>(
        &self,
        handler: Arc<H>,
        callback: ListenCallback,
    ) -> QueueResult<()>
    where
        H: MessageHandler<B::Message> + 'static,
    {
        self.running.store(true, Ordering::SeqCst);

        while self.running.load(Ordering::SeqCst) {
            match self.backend.pop_blocking(Duration::from_secs(1)).await {
                Ok(Some((message, token))) => {
                    match handler.handle(message).await {
                        Ok(()) => {
                            if let Err(e) = self.backend.ack(token).await {
                                log::warn!("ack failed: {:?}", e);
                            }
                            callback(Ok(()));
                        }
                        Err(e) => {
                            if let Err(ne) = self.backend.nack(token, false).await {
                                log::warn!("nack failed: {:?}", ne);
                            }
                            callback(Err(e));
                        }
                    }
                }
                Ok(None) => {
                    // 超时，继续轮询
                }
                Err(e) => {
                    // 后端错误（如连接断开），通知外部但继续尝试
                    callback(Err(e));
                }
            }
        }

        Ok(())
    }

    /// 优雅关闭消费者
    ///
    /// 停止轮询并关闭后端队列。
    pub async fn shutdown(&self, timeout: Duration) -> QueueResult<()> {
        self.running.store(false, Ordering::SeqCst);
        self.backend.shutdown(timeout).await
    }

    /// 获取后端队列的引用
    pub fn backend(&self) -> &B {
        &self.backend
    }
}
