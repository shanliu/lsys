//! 核心 Trait 及队列实现测试

#[cfg(test)]
mod tests {
    use super::super::*;
    use async_trait::async_trait;
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

    // 测试消息类型：只需 derive，无需手动 impl QueueMessage
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestMessage {
        pub content: String,
        pub id: u64,
    }

    // 测试消息处理器
    struct TestHandler {
        pub processed_count: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl MessageHandler<TestMessage> for TestHandler {
        async fn handle(&self, message: TestMessage) -> QueueResult<()> {
            self.processed_count.fetch_add(1, AtomicOrdering::SeqCst);
            if message.content.contains("error") {
                Err(QueueCacheError::System("test-processing-failed".to_string()))
            } else {
                Ok(())
            }
        }
    }

    // ---- QueueMessage 通过 blanket impl 自动满足 ----

    #[tokio::test]
    async fn test_message_serde_roundtrip() {
        let message = TestMessage {
            content: "Hello, World!".to_string(),
            id: 1,
        };
        let json = serde_json::to_string(&message).unwrap();
        let deserialized: TestMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(message, deserialized);
    }

    #[tokio::test]
    async fn test_message_handler_success() {
        let handler = TestHandler {
            processed_count: Arc::new(AtomicUsize::new(0)),
        };
        let message = TestMessage {
            content: "success".to_string(),
            id: 1,
        };
        let result = handler.handle(message).await;
        assert!(result.is_ok());
        assert_eq!(handler.processed_count.load(AtomicOrdering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_message_handler_failure() {
        let handler = TestHandler {
            processed_count: Arc::new(AtomicUsize::new(0)),
        };
        let message = TestMessage {
            content: "error message".to_string(),
            id: 2,
        };
        let result = handler.handle(message).await;
        assert!(result.is_err());
    }

    // ---- MemoryQueue 测试 ----

    fn make_config(capacity: usize) -> MemoryQueueConfig {
        MemoryQueueConfig {
            capacity,
            enable_metrics: true,
        }
    }

    #[tokio::test]
    async fn test_memory_queue_push_pop() {
        let queue = MemoryQueue::<TestMessage>::new(make_config(10)).await.unwrap();

        let msg = TestMessage { content: "hello".to_string(), id: 1 };
        queue.push(msg.clone()).await.unwrap();

        let (popped, token) = queue
            .pop_blocking(std::time::Duration::from_millis(100))
            .await.unwrap().unwrap();
        assert_eq!(popped, msg);

        // ack：无操作，仅验证不报错
        queue.ack(token).await.unwrap();
    }

    #[tokio::test]
    async fn test_memory_queue_pop_empty() {
        let queue = MemoryQueue::<TestMessage>::new(make_config(10)).await.unwrap();
        let result = queue
            .pop_blocking(std::time::Duration::from_millis(10))
            .await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_memory_queue_nack_increments_failed() {
        let queue = MemoryQueue::<TestMessage>::new(make_config(10)).await.unwrap();

        let msg = TestMessage { content: "hello".to_string(), id: 1 };
        queue.push(msg).await.unwrap();

        let (_, token) = queue
            .pop_blocking(std::time::Duration::from_millis(100))
            .await.unwrap().unwrap();
        queue.nack(token, false).await.unwrap();

        assert_eq!(queue.metrics().failed(), 1);
    }

    #[tokio::test]
    async fn test_memory_queue_pop_blocking_timeout() {
        let queue = MemoryQueue::<TestMessage>::new(make_config(10)).await.unwrap();
        let result = queue
            .pop_blocking(std::time::Duration::from_millis(100))
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_memory_queue_metrics() {
        let queue = MemoryQueue::<TestMessage>::new(make_config(10)).await.unwrap();

        for i in 0..3u64 {
            queue.push(TestMessage { content: "msg".to_string(), id: i }).await.unwrap();
        }
        assert_eq!(queue.metrics().pushed(), 3);

        let (_, t1) = queue.pop_blocking(std::time::Duration::from_millis(100)).await.unwrap().unwrap();
        queue.ack(t1).await.unwrap();
        let (_, t2) = queue.pop_blocking(std::time::Duration::from_millis(100)).await.unwrap().unwrap();
        queue.nack(t2, false).await.unwrap();

        assert_eq!(queue.metrics().consumed(), 2);
        assert_eq!(queue.metrics().failed(), 1);
        assert_eq!(queue.metrics().current_size(), 1);
    }

    // ---- Error 转换测试 ----

    #[test]
    fn test_error_serde_conversion() {
        let json_err = serde_json::from_str::<TestMessage>("invalid json");
        assert!(json_err.is_err());
        let queue_err: QueueCacheError = json_err.unwrap_err().into();
        assert!(matches!(queue_err, QueueCacheError::Serialization(_)));
    }

    #[test]
    fn test_error_to_fluent_message() {
        use crate::fluents::IntoFluentMessage;

        let errors = vec![
            QueueCacheError::QueueFull { capacity: 100 },
            QueueCacheError::QueueClosed,
            QueueCacheError::Timeout { timeout: std::time::Duration::from_secs(30) },
        ];
        for error in errors {
            let message = error.to_fluent_message();
            assert!(!message.id.is_empty());
        }
    }

    // ---- QueueMetrics 测试 ----

    #[test]
    fn test_metrics_new_all_zero() {
        let metrics = QueueMetrics::new();
        assert_eq!(metrics.pushed(), 0);
        assert_eq!(metrics.consumed(), 0);
        assert_eq!(metrics.failed(), 0);
        assert_eq!(metrics.current_size(), 0);
    }

    #[test]
    fn test_metrics_increment_pushed() {
        let metrics = QueueMetrics::new();
        metrics.increment_pushed();
        metrics.increment_pushed();
        assert_eq!(metrics.pushed(), 2);
    }

    #[test]
    fn test_metrics_increment_consumed() {
        let metrics = QueueMetrics::new();
        metrics.increment_consumed();
        metrics.increment_consumed();
        metrics.increment_consumed();
        assert_eq!(metrics.consumed(), 3);
    }

    #[test]
    fn test_metrics_increment_failed() {
        let metrics = QueueMetrics::new();
        metrics.increment_failed();
        assert_eq!(metrics.failed(), 1);
    }

    #[test]
    fn test_metrics_current_size_calculation() {
        let metrics = QueueMetrics::new();
        for _ in 0..5 { metrics.increment_pushed(); }
        assert_eq!(metrics.current_size(), 5);
        for _ in 0..2 { metrics.increment_consumed(); }
        assert_eq!(metrics.current_size(), 3);
        for _ in 0..3 { metrics.increment_consumed(); }
        assert_eq!(metrics.current_size(), 0);
    }

    #[test]
    fn test_metrics_current_size_saturating_sub() {
        let metrics = QueueMetrics::new();
        // consumed > pushed 不应下溢
        metrics.increment_consumed();
        assert_eq!(metrics.current_size(), 0);
    }
}
