//! Message Wrapper
//!
//! 消息包装器，用于在队列中跟踪消息的重试次数和时间戳。
//! 适用于在 `MessageHandler::handle` 内部自行管理重试的场景。
//!
//! # 示例
//!
//! ```ignore
//! use serde::{Serialize, Deserialize};
//! use lsys_core::queue_cache::MessageWrapper;
//!
//! #[derive(Serialize, Deserialize)]
//! struct MyMessage { content: String }
//!
//! let message = MyMessage { content: "Hello".to_string() };
//! let mut wrapper = MessageWrapper::new(message);
//!
//! wrapper.increment_retry();
//! assert_eq!(wrapper.retry_count, 1);
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use super::QueueMessage;

/// 消息包装器
/// 
/// 包装原始消息并添加重试追踪信息。这对于实现可靠的消息处理
/// 和重试机制至关重要。
/// 
/// # 字段
/// 
/// - `message`: 原始消息内容
/// - `retry_count`: 当前重试次数（初始为 0）
/// - `first_enqueued_at`: 消息首次入队的 UTC 时间戳
/// - `last_retry_at`: 最后一次重试的 UTC 时间戳（如果有的话）
/// 
/// # 泛型约束
/// 
/// 泛型参数 `M` 必须实现 `QueueMessage` trait，确保消息可以被序列化和反序列化。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct MessageWrapper<M: QueueMessage> {
    /// 原始消息
    pub message: M,
    
    /// 重试次数
    /// 
    /// 初始值为 0，每次调用 `increment_retry()` 时递增。
    pub retry_count: u32,
    
    /// 首次入队时间
    /// 
    /// 消息首次被推送到队列时的 UTC 时间戳。
    /// 这个时间戳在消息的整个生命周期中保持不变。
    pub first_enqueued_at: DateTime<Utc>,
    
    /// 最后重试时间
    /// 
    /// 最近一次调用 `increment_retry()` 的 UTC 时间戳。
    /// 初始值为 None，表示还没有重试过。
    pub last_retry_at: Option<DateTime<Utc>>,
}

impl<M: QueueMessage> MessageWrapper<M> {
    /// 创建新的消息包装器
    /// 
    /// 使用当前 UTC 时间作为首次入队时间，重试次数初始化为 0。
    /// 
    /// # 参数
    /// 
    /// - `message`: 要包装的原始消息
    /// 
    /// # 返回
    /// 
    /// 返回一个新的 `MessageWrapper` 实例
    /// 
    /// # 示例
    /// 
    /// ```ignore
    /// let message = MyMessage { content: "Hello".to_string() };
    /// let wrapper = MessageWrapper::new(message);
    /// assert_eq!(wrapper.retry_count, 0);
    /// assert!(wrapper.last_retry_at.is_none());
    /// ```
    pub fn new(message: M) -> Self {
        Self {
            message,
            retry_count: 0,
            first_enqueued_at: Utc::now(),
            last_retry_at: None,
        }
    }
    
    /// 递增重试计数
    /// 
    /// 将重试计数加 1，并更新最后重试时间为当前 UTC 时间。
    /// 这个方法应该在消息处理失败后、准备重试之前调用。
    /// 
    /// # 示例
    /// 
    /// ```ignore
    /// let message = MyMessage { content: "Hello".to_string() };
    /// let mut wrapper = MessageWrapper::new(message);
    /// 
    /// wrapper.increment_retry();
    /// assert_eq!(wrapper.retry_count, 1);
    /// assert!(wrapper.last_retry_at.is_some());
    /// 
    /// wrapper.increment_retry();
    /// assert_eq!(wrapper.retry_count, 2);
    /// ```
    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
        self.last_retry_at = Some(Utc::now());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    // 测试用的简单消息类型
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestMessage {
        content: String,
    }

    #[test]
    fn test_new_message_wrapper() {
        let message = TestMessage {
            content: "Hello".to_string(),
        };
        let wrapper = MessageWrapper::new(message.clone());

        assert_eq!(wrapper.message, message);
        assert_eq!(wrapper.retry_count, 0);
        assert!(wrapper.last_retry_at.is_none());
        // 验证时间戳是最近的（在过去1秒内）
        let now = Utc::now();
        let diff = now.signed_duration_since(wrapper.first_enqueued_at);
        assert!(diff.num_seconds() < 1);
    }

    #[test]
    fn test_increment_retry() {
        let message = TestMessage {
            content: "Hello".to_string(),
        };
        let mut wrapper = MessageWrapper::new(message);

        // 第一次递增
        wrapper.increment_retry();
        assert_eq!(wrapper.retry_count, 1);
        assert!(wrapper.last_retry_at.is_some());
        let first_retry_time = wrapper.last_retry_at.unwrap();

        // 等待一小段时间确保时间戳不同
        std::thread::sleep(std::time::Duration::from_millis(10));

        // 第二次递增
        wrapper.increment_retry();
        assert_eq!(wrapper.retry_count, 2);
        assert!(wrapper.last_retry_at.is_some());
        let second_retry_time = wrapper.last_retry_at.unwrap();

        // 验证时间戳已更新
        assert!(second_retry_time > first_retry_time);
    }

    #[test]
    fn test_multiple_increments() {
        let message = TestMessage {
            content: "Test".to_string(),
        };
        let mut wrapper = MessageWrapper::new(message);

        // 递增多次
        for i in 1..=5 {
            wrapper.increment_retry();
            assert_eq!(wrapper.retry_count, i);
            assert!(wrapper.last_retry_at.is_some());
        }
    }

    #[test]
    fn test_message_wrapper_is_queue_message() {
        let message = TestMessage {
            content: "Hello".to_string(),
        };
        let wrapper = MessageWrapper::new(message);
        // MessageWrapper<M> 通过 blanket impl 自动满足 QueueMessage
        // 只需能序列化/反序列化即可验证
        let _json = serde_json::to_string(&wrapper).expect("should serialize");
    }

    #[test]
    fn test_serialization_roundtrip() {
        let message = TestMessage {
            content: "Serialize me".to_string(),
        };
        let wrapper = MessageWrapper::new(message);

        // 序列化
        let serialized = serde_json::to_string(&wrapper).expect("Failed to serialize");

        // 反序列化
        let deserialized: MessageWrapper<TestMessage> =
            serde_json::from_str(&serialized).expect("Failed to deserialize");

        // 验证数据完整性
        assert_eq!(deserialized.message, wrapper.message);
        assert_eq!(deserialized.retry_count, wrapper.retry_count);
        assert_eq!(
            deserialized.first_enqueued_at.timestamp(),
            wrapper.first_enqueued_at.timestamp()
        );
        assert_eq!(deserialized.last_retry_at, wrapper.last_retry_at);
    }

    #[test]
    fn test_first_enqueued_at_preserved() {
        let message = TestMessage {
            content: "Time test".to_string(),
        };
        let mut wrapper = MessageWrapper::new(message);
        let original_time = wrapper.first_enqueued_at;

        // 等待并递增重试
        std::thread::sleep(std::time::Duration::from_millis(10));
        wrapper.increment_retry();

        // 验证首次入队时间没有改变
        assert_eq!(wrapper.first_enqueued_at, original_time);
        // 验证最后重试时间已更新
        assert!(wrapper.last_retry_at.is_some());
        assert!(wrapper.last_retry_at.unwrap() > original_time);
    }
}
