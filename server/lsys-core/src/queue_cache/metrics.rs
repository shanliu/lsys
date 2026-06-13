//! 队列监控统计模块
//! 
//! 提供线程安全的队列运行指标收集和查询功能。

use std::sync::atomic::{AtomicUsize, Ordering};

/// 队列监控统计
/// 
/// 使用原子操作实现线程安全的计数器，用于跟踪队列的运行状态和性能指标。
/// 
/// # 示例
/// 
/// ```
/// use lsys_core::queue_cache::QueueMetrics;
/// 
/// let metrics = QueueMetrics::new();
/// 
/// // 记录消息推送
/// metrics.increment_pushed();
/// 
/// // 记录消息消费
/// metrics.increment_consumed();
/// 
/// // 查询统计信息
/// assert_eq!(metrics.pushed(), 1);
/// assert_eq!(metrics.consumed(), 1);
/// assert_eq!(metrics.current_size(), 0);
/// ```
#[derive(Debug)]
pub struct QueueMetrics {
    /// 推送消息总数
    pushed: AtomicUsize,
    
    /// 消费消息总数
    consumed: AtomicUsize,
    
    /// 处理失败总数
    failed: AtomicUsize,
}

impl QueueMetrics {
    /// 创建新的队列指标实例
    /// 
    /// 所有计数器初始化为 0。
    pub fn new() -> Self {
        Self {
            pushed: AtomicUsize::new(0),
            consumed: AtomicUsize::new(0),
            failed: AtomicUsize::new(0),
        }
    }
    
    /// 递增推送消息计数
    /// 
    /// 每当消息成功推送到队列时应调用此方法。
    /// 使用 Relaxed 排序以获得最佳性能，因为计数器之间没有依赖关系。
    pub fn increment_pushed(&self) {
        self.pushed.fetch_add(1, Ordering::Relaxed);
    }
    
    /// 递增消费消息计数
    /// 
    /// 每当消息从队列中消费时应调用此方法。
    /// 使用 Relaxed 排序以获得最佳性能。
    pub fn increment_consumed(&self) {
        self.consumed.fetch_add(1, Ordering::Relaxed);
    }
    
    /// 递增处理失败计数
    /// 
    /// 每当消息处理失败时应调用此方法。
    /// 使用 Relaxed 排序以获得最佳性能。
    pub fn increment_failed(&self) {
        self.failed.fetch_add(1, Ordering::Relaxed);
    }
    
    /// 获取推送消息总数
    /// 
    /// 返回自队列创建以来成功推送的消息总数。
    pub fn pushed(&self) -> usize {
        self.pushed.load(Ordering::Relaxed)
    }
    
    /// 获取消费消息总数
    /// 
    /// 返回自队列创建以来成功消费的消息总数。
    pub fn consumed(&self) -> usize {
        self.consumed.load(Ordering::Relaxed)
    }
    
    /// 获取处理失败总数
    /// 
    /// 返回自队列创建以来消息处理失败的总次数。
    pub fn failed(&self) -> usize {
        self.failed.load(Ordering::Relaxed)
    }
    
    /// 计算当前队列大小
    /// 
    /// 通过 pushed - consumed 计算当前队列中的消息数量。
    /// 使用 saturating_sub 防止下溢（在并发情况下可能发生）。
    /// 
    /// # 注意
    /// 
    /// 在高并发场景下，由于原子操作之间没有同步，
    /// 这个值可能不是完全精确的，但足够用于监控目的。
    pub fn current_size(&self) -> usize {
        self.pushed().saturating_sub(self.consumed())
    }
}

impl Default for QueueMetrics {
    fn default() -> Self {
        Self::new()
    }
}
