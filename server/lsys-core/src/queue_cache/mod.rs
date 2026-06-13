//! Queue Cache Layer
//! 
//! 通用队列缓存层提供统一的消息队列抽象接口,支持多种后端实现。
//! 
//! # 特性
//! 
//! - 类型安全的泛型消息处理
//! - 可扩展的队列后端实现
//! - 异步操作支持
//! - 统一的错误处理
//! 
//! # 示例
//! 
//! ```ignore
//! use lsys_core::queue_cache::{QueueMessage, MessageHandler, QueueBackend};
//!
//! // 定义消息类型（只需 derive，无需 impl QueueMessage）
//! #[derive(Serialize, Deserialize)]
//! struct MyMessage {
//!     content: String,
//! }
//!
//! // 实现消息处理器
//! struct MyHandler;
//!
//! #[async_trait]
//! impl MessageHandler<MyMessage> for MyHandler {
//!     async fn handle(&self, message: MyMessage) -> QueueResult<()> {
//!         println!("Processing: {}", message.content);
//!         Ok(())
//!     }
//! }
//! ```

// Re-exports
pub use traits::{QueueMessage, MessageHandler, QueueBackend};
pub use metrics::QueueMetrics;
pub use message_wrapper::MessageWrapper;
pub use result::{QueueCacheError, QueueResult};
pub use memory_queue::{MemoryQueue, MemoryQueueConfig};
pub use consumer::{QueueConsumer, ListenCallback};

#[cfg(feature = "queue-cache-rabbitmq")]
pub use rabbitmq_queue::{RabbitMQQueue, RabbitMQConfig};

#[cfg(feature = "queue-cache-yaque")]
pub use yaque_queue::{YaqueQueue, YaqueQueueConfig};

mod traits;
mod metrics;
mod message_wrapper;
mod result;
mod memory_queue;
mod consumer;

#[cfg(feature = "queue-cache-rabbitmq")]
mod rabbitmq_queue;

#[cfg(feature = "queue-cache-yaque")]
mod yaque_queue;

#[cfg(test)]
mod tests;
