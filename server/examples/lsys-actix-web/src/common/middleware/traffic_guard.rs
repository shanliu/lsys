//! 流量守护中间件 - 熔断 + 限流，防止恶意攻击
//!
//! 设计原则:
//! - **业务驱动**: 业务端通过 X-Fuse 响应头标记失败，中间件按规则匹配
//! - **标签熔断**: 同一 tag 支持多阈值规则，任一触发即熔断
//! - **无锁并发**: 使用DashMap和原子操作，支持超高并发
//! - **滑动窗口**: 精确的时间窗口统计
//! - **绝对可靠**: 所有状态转换使用原子操作，无竞态条件
//!
//! # 使用示例
//! ```rust
//! use crate::common::middleware::{TrafficGuard, FuseTagRule, FuseThreshold, FuseTag, IpThrottle};
//!
//! let traffic_guard = TrafficGuard::builder()
//!     .fuse_rule(FuseTagRule {
//!         tag: FuseTag::Prefix("LOGIN_".into()),
//!         use_ip: true,
//!         rules: vec![
//!             FuseThreshold { window_secs: 60, max_failures: 20, circuit_duration_secs: 300, half_open_requests: 3 },
//!         ],
//!     })
//!     .ip_throttle(IpThrottle {
//!         path_rules: vec![
//!             IpThrottleRule {
//!                 path: IpPath::Prefix("/api"),
//!                 window_secs: 60,
//!                 max_requests: 500,
//!                 circuit_duration_secs: 1800,
//!             },
//!             // IpPath::None 作为全局兜底（仅当 Exact/Prefix 都未命中时生效）
//!             IpThrottleRule {
//!                 path: IpPath::None,
//!                 window_secs: 60,
//!                 max_requests: 1000,
//!                 circuit_duration_secs: 1800,
//!             },
//!         ],
//!     })
//!     .build();
//! ```
//!
//! # 业务端标记失败
//! ```rust
//! use crate::common::middleware::traffic_guard::fuse_header;
//! use actix_web::HttpResponse;
//!
//! // 在 handler 中标记失败
//! let mut response = HttpResponse::Ok().json(result);
//! fuse_header(&mut response, "LOGIN_U1");
//! ```
///
///

use actix_utils::future::{Ready, ready};
use actix_web::{
    Error, HttpResponse,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::{StatusCode, header::{HeaderName, HeaderValue}},
};
use futures_util::future::LocalBoxFuture;
use std::{
    borrow::Cow,
    collections::HashSet,
    sync::Arc,
    time::{Duration, Instant},
};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use lsys_web::common::{JsonData, JsonResponse};
use tracing::warn;

use dashmap::DashMap;

// ============================================================
// 性能优化常量
// ============================================================

/// 时间戳缓存刷新间隔（毫秒）
const TIMESTAMP_CACHE_MS: u64 = 10;

// ============================================================
// 常量
// ============================================================

/// 业务端在响应中设置此头，标记失败标签
pub static X_FUSE: HeaderName = HeaderName::from_static("x-fuse");

/// 中间件拒绝时回写此头，格式: `{tag};{window_secs}-{max_failures}`
pub static X_FUSE_TRIGGERED: HeaderName = HeaderName::from_static("x-fuse-triggered");

/// IP 限流触发标识
const IP_THROTTLE_TAG: &str = "_IP_THROTTLE";

// ============================================================
// 公开类型
// ============================================================

/// 熔断状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitState {
    /// 关闭状态 - 正常运行，统计失败数
    Closed,
    /// 打开状态 - 熔断生效，拒绝请求
    Open,
    /// 半开状态 - 允许部分请求探测服务是否恢复
    HalfOpen,
}

/// 标签匹配模式
///
/// - `Exact("SMS_SEND")` — 精确匹配，X-Fuse 值必须等于 "SMS_SEND"
/// - `Prefix("LOGIN_")` — 前缀匹配，X-Fuse 值以 "LOGIN_" 开头即可
///
/// 使用 `&'static str` 避免堆分配，标签均为编译期常量。
///
/// # 生成 X-Fuse 头值
///
/// ```rust
/// use crate::common::middleware::FuseTag;
///
/// let tag = FuseTag::Exact("SMS_SEND");
/// assert_eq!(tag.to_tag(None), "SMS_SEND");
///
/// let tag = FuseTag::Prefix("LOGIN_");
/// assert_eq!(tag.to_tag(Some("U1")), "LOGIN_U1");
/// assert_eq!(tag.to_tag(None), "LOGIN_");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[allow(dead_code)] // Prefix 变体为中间件核心功能，当前业务未使用但保留
pub enum FuseTag {
    /// 精确匹配
    Exact(&'static str),
    /// 前缀匹配
    Prefix(&'static str),
}

impl FuseTag {
    /// 判断给定的 tag 值是否匹配此模式
    fn matches(&self, value: &str) -> bool {
        match self {
            FuseTag::Exact(s) => value == *s,
            FuseTag::Prefix(s) => value.starts_with(s),
        }
    }

    /// 获取内部字符串引用
    #[allow(dead_code)]
    fn as_str(&self) -> &'static str {
        match self {
            FuseTag::Exact(s) => s,
            FuseTag::Prefix(s) => s,
        }
    }

    /// 生成 X-Fuse 头值
    ///
    /// - `Exact` → 忽略 suffix，直接返回标签本身
    /// - `Prefix` → 拼接 suffix（如 `"LOGIN_"` + `"U1"` = `"LOGIN_U1"`）
    ///
    /// 业务端可直接将返回值传给 `fuse_header()`：
    /// ```rust
    /// fuse_header(&mut response, tag.to_tag(Some("U1")));
    /// ```
    #[allow(dead_code)]
    pub fn to_tag(&self, suffix: Option<&str>) -> Cow<'static, str> {
        match self {
            FuseTag::Exact(s) => Cow::Borrowed(s),
            FuseTag::Prefix(s) => match suffix {
                Some(sf) => Cow::Owned(format!("{}{}", s, sf)),
                None => Cow::Borrowed(s),
            },
        }
    }
}

/// 熔断阈值规则 — 一个 tag 可配多个阈值，任一触发即熔断
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuseThreshold {
    /// 滑动窗口时长(秒)
    pub window_secs: u64,
    /// 窗口内最大失败次数
    pub max_failures: u64,
    /// 熔断持续时间(秒)
    pub circuit_duration_secs: u64,
    /// 半开状态允许探测请求数 (0=跳过半开直接恢复)
    pub half_open_requests: u64,
}

/// 标签熔断规则
#[derive(Debug, Clone, Serialize)]
pub struct FuseTagRule {
    /// 标签匹配模式
    pub tag: FuseTag,
    /// true → 维度键含 IP / false → 仅 tag
    pub use_ip: bool,
    /// 多阈值规则，任一触发即熔断
    pub rules: Vec<FuseThreshold>,
}

/// IP 限流配置（路径规则驱动，与标签熔断独立）
///
/// 匹配优先级：`Exact` > `Prefix` > `None`（兜底）
///
/// `IpPath::None` 规则充当全局兜底：仅当 `Exact`/`Prefix` 都未命中时生效。
#[derive(Debug, Clone, Serialize)]
pub struct IpThrottle {
    /// 路径规则（优先级：Exact > Prefix > None）
    pub path_rules: Vec<IpThrottleRule>,
}

/// 路径匹配模式（用于 IP 限流）
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum IpPath {
    /// 精确匹配
    Exact(&'static str),
    /// 前缀匹配
    Prefix(&'static str),
    /// 任意路径（兜底）
    None,
}

impl IpPath {
    fn normalize_path(value: &str) -> &str {
        value.trim_start_matches('/')
    }

    fn matches(&self, request_path: &str) -> bool {
        match self {
            IpPath::Exact(s) => Self::normalize_path(request_path) == Self::normalize_path(s),
            IpPath::Prefix(s) => Self::normalize_path(request_path).starts_with(Self::normalize_path(s)),
            IpPath::None => true,
        }
    }

    fn as_pattern(&self) -> Option<&'static str> {
        match self {
            IpPath::Exact(s) => Some(s),
            IpPath::Prefix(s) => Some(s),
            IpPath::None => None,
        }
    }

    fn kind_name(&self) -> &'static str {
        match self {
            IpPath::Exact(_) => "exact",
            IpPath::Prefix(_) => "prefix",
            IpPath::None => "none",
        }
    }
}

/// 路径限流规则
#[derive(Debug, Clone, Serialize)]
pub struct IpThrottleRule {
    /// 路径匹配规则
    pub path: IpPath,
    /// 阈值规则（支持多级，如 60s/100次→5分钟封禁 + 300s/500次→4小时封禁）
    pub rules: Vec<IpThreshold>,
}

/// IP 限流阈值
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct IpThreshold {
    /// 滑动窗口时长(秒)
    pub window_secs: u64,
    /// 窗口内最大请求数
    pub max_requests: u64,
    /// 超限后封禁时长(秒)
    pub circuit_duration_secs: u64,
}

// ============================================================
// 业务端辅助函数
// ============================================================

/// 在响应中设置 X-Fuse 头，标记失败标签
///
/// # 使用示例
/// ```rust
/// use crate::common::middleware::traffic_guard::fuse_header;
/// use crate::common::middleware::FuseTag;
/// use actix_web::HttpResponse;
///
/// // 方式1: 直接传字符串
/// fuse_header(&mut response, "LOGIN_U1");
///
/// // 方式2: 通过 FuseTag 生成
/// let tag = FuseTag::Prefix("LOGIN_");
/// fuse_header(&mut response, &tag.to_tag(Some("U1")));
/// ```
#[allow(dead_code)]
pub fn fuse_header(response: &mut HttpResponse, tag: &str) {
    if let Ok(val) = HeaderValue::from_str(tag) {
        response.headers_mut().insert(X_FUSE.clone(), val);
    }
}

/// 在任意 ServiceResponse 中设置 X-Fuse 头
#[allow(dead_code)]
pub fn fuse_header_on_response<B>(response: &mut ServiceResponse<B>, tag: &str) {
    if let Ok(val) = HeaderValue::from_str(tag) {
        response.response_mut().headers_mut().insert(X_FUSE.clone(), val);
    }
}

// ============================================================
// 内部类型: SlidingWindow (复用原有实现)
// ============================================================

/// 滑动窗口计数器 - 使用原子操作实现无锁并发
///
/// 优化要点：
/// - 使用Instant::elapsed()，永远不会panic，不受系统时间调整影响
/// - 所有操作都是原子操作，无竞态条件
/// - 桶的时间戳使用相对时间（距离创建时的毫秒数），确保时间计算的准确性
/// - 使用 Relaxed 内存顺序优化性能（计数场景无需强同步）
/// - Cache line 对齐避免 false sharing
/// - 时间戳缓存减少系统调用
#[derive(Debug)]
struct SlidingWindow {
    window_ms: u64,
    bucket_ms: u64,
    bucket_count: usize,
    request_buckets: Box<[CacheLinePadded<AtomicU64>; 10]>,
    failure_buckets: Box<[CacheLinePadded<AtomicU64>; 10]>,
    bucket_timestamps: Box<[CacheLinePadded<AtomicU64>; 10]>,
    created_at: Instant,
    /// 缓存的时间戳（毫秒），减少 elapsed() 调用
    cached_time_ms: CacheLinePadded<AtomicU64>,
    /// 缓存更新时间戳（纳秒）
    cache_updated_at_ns: CacheLinePadded<AtomicU64>,
}

/// Cache line 对齐的原子类型，避免 false sharing
#[repr(align(64))]
#[derive(Debug)]
struct CacheLinePadded<T> {
    value: T,
}

impl<T> CacheLinePadded<T> {
    fn new(value: T) -> Self {
        Self { value }
    }
}

impl<T> std::ops::Deref for CacheLinePadded<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl SlidingWindow {
    fn new(window_secs: u64, created_at: Instant) -> Self {
        let bucket_count = 10;
        let window_ms = window_secs * 1000;
        let bucket_ms = window_ms / bucket_count as u64;
        let elapsed_ms = created_at.elapsed().as_millis() as u64;

        let request_buckets = Box::new([
            CacheLinePadded::new(AtomicU64::new(0)), CacheLinePadded::new(AtomicU64::new(0)),
            CacheLinePadded::new(AtomicU64::new(0)), CacheLinePadded::new(AtomicU64::new(0)),
            CacheLinePadded::new(AtomicU64::new(0)), CacheLinePadded::new(AtomicU64::new(0)),
            CacheLinePadded::new(AtomicU64::new(0)), CacheLinePadded::new(AtomicU64::new(0)),
            CacheLinePadded::new(AtomicU64::new(0)), CacheLinePadded::new(AtomicU64::new(0)),
        ]);
        let failure_buckets = Box::new([
            CacheLinePadded::new(AtomicU64::new(0)), CacheLinePadded::new(AtomicU64::new(0)),
            CacheLinePadded::new(AtomicU64::new(0)), CacheLinePadded::new(AtomicU64::new(0)),
            CacheLinePadded::new(AtomicU64::new(0)), CacheLinePadded::new(AtomicU64::new(0)),
            CacheLinePadded::new(AtomicU64::new(0)), CacheLinePadded::new(AtomicU64::new(0)),
            CacheLinePadded::new(AtomicU64::new(0)), CacheLinePadded::new(AtomicU64::new(0)),
        ]);

        let aligned_elapsed = (elapsed_ms / bucket_ms) * bucket_ms;
        let bucket_timestamps = Box::new([
            CacheLinePadded::new(AtomicU64::new(aligned_elapsed.saturating_sub(bucket_ms * 9))),
            CacheLinePadded::new(AtomicU64::new(aligned_elapsed.saturating_sub(bucket_ms * 8))),
            CacheLinePadded::new(AtomicU64::new(aligned_elapsed.saturating_sub(bucket_ms * 7))),
            CacheLinePadded::new(AtomicU64::new(aligned_elapsed.saturating_sub(bucket_ms * 6))),
            CacheLinePadded::new(AtomicU64::new(aligned_elapsed.saturating_sub(bucket_ms * 5))),
            CacheLinePadded::new(AtomicU64::new(aligned_elapsed.saturating_sub(bucket_ms * 4))),
            CacheLinePadded::new(AtomicU64::new(aligned_elapsed.saturating_sub(bucket_ms * 3))),
            CacheLinePadded::new(AtomicU64::new(aligned_elapsed.saturating_sub(bucket_ms * 2))),
            CacheLinePadded::new(AtomicU64::new(aligned_elapsed.saturating_sub(bucket_ms))),
            CacheLinePadded::new(AtomicU64::new(aligned_elapsed)),
        ]);

        Self {
            window_ms,
            bucket_ms,
            bucket_count,
            request_buckets,
            failure_buckets,
            bucket_timestamps,
            created_at,
            cached_time_ms: CacheLinePadded::new(AtomicU64::new(elapsed_ms)),
            cache_updated_at_ns: CacheLinePadded::new(AtomicU64::new(created_at.elapsed().as_nanos() as u64)),
        }
    }

    /// 获取当前时间戳（毫秒），使用缓存减少系统调用
    #[inline]
    fn get_elapsed_ms(&self) -> u64 {
        let now_ns = self.created_at.elapsed().as_nanos() as u64;
        let cache_ns = self.cache_updated_at_ns.load(Ordering::Relaxed);
        
        // 缓存未过期，直接返回
        if now_ns.saturating_sub(cache_ns) < TIMESTAMP_CACHE_MS * 1_000_000 {
            return self.cached_time_ms.load(Ordering::Relaxed);
        }

        // 缓存过期，更新（允许多线程竞争更新，无需 CAS）
        let now_ms = (now_ns / 1_000_000) as u64;
        self.cached_time_ms.store(now_ms, Ordering::Relaxed);
        self.cache_updated_at_ns.store(now_ns, Ordering::Relaxed);
        now_ms
    }

    /// 记录一次请求 (is_failure=true 记录失败, false 仅记录请求)
    #[allow(dead_code)]
    fn record(&self, is_failure: bool) {
        let now_ms = self.get_elapsed_ms();
        let bucket_idx = self.get_bucket_index(now_ms);
        self.update_bucket_timestamp(bucket_idx, now_ms);
        self.request_buckets[bucket_idx].fetch_add(1, Ordering::Relaxed);
        if is_failure {
            self.failure_buckets[bucket_idx].fetch_add(1, Ordering::Relaxed);
        }
    }

    /// 仅记录请求计数（无预计算时间戳版本，测试用）
    #[allow(dead_code)]
    #[inline]
    fn record_request(&self) {
        let now_ms = self.get_elapsed_ms();
        self.record_request_at(now_ms);
    }

    /// 仅记录请求计数，使用预计算的时间戳（避免重复 get_elapsed_ms）
    #[inline]
    fn record_request_at(&self, now_ms: u64) {
        let bucket_idx = self.get_bucket_index(now_ms);
        self.update_bucket_timestamp(bucket_idx, now_ms);
        self.request_buckets[bucket_idx].fetch_add(1, Ordering::Relaxed);
    }

    /// 仅记录失败计数（无预计算时间戳版本，测试用）
    #[allow(dead_code)]
    #[inline]
    fn record_failure(&self) {
        let now_ms = self.get_elapsed_ms();
        self.record_failure_at(now_ms);
    }

    /// 仅记录失败计数，使用预计算的时间戳（避免重复 get_elapsed_ms）
    #[inline]
    fn record_failure_at(&self, now_ms: u64) {
        let bucket_idx = self.get_bucket_index(now_ms);
        self.update_bucket_timestamp(bucket_idx, now_ms);
        self.failure_buckets[bucket_idx].fetch_add(1, Ordering::Relaxed);
    }

    /// 获取窗口内统计 (总请求数, 总失败数)
    #[allow(dead_code)]
    fn get_stats(&self) -> (u64, u64) {
        let now_ms = self.get_elapsed_ms();
        let window_start_ms = now_ms.saturating_sub(self.window_ms);
        let mut total_requests = 0u64;
        let mut total_failures = 0u64;
        for i in 0..self.bucket_count {
            let bucket_start_ms = self.bucket_timestamps[i].load(Ordering::Acquire);
            if bucket_start_ms >= window_start_ms {
                total_requests = total_requests.saturating_add(self.request_buckets[i].load(Ordering::Relaxed));
                total_failures = total_failures.saturating_add(self.failure_buckets[i].load(Ordering::Relaxed));
            }
        }
        (total_requests, total_failures)
    }

    /// 窗口内失败数（使用预计算时间戳，避免重复 get_elapsed_ms）
    #[inline]
    fn get_failure_count_at(&self, now_ms: u64) -> u64 {
        let window_start_ms = now_ms.saturating_sub(self.window_ms);
        let mut total_failures = 0u64;
        for i in 0..self.bucket_count {
            let bucket_start_ms = self.bucket_timestamps[i].load(Ordering::Acquire);
            if bucket_start_ms >= window_start_ms {
                total_failures = total_failures.saturating_add(self.failure_buckets[i].load(Ordering::Relaxed));
            }
        }
        total_failures
    }

    /// 窗口内失败数（无预计算时间戳版本，测试用）
    #[allow(dead_code)]
    #[inline]
    fn get_failure_count(&self) -> u64 {
        self.get_failure_count_at(self.get_elapsed_ms())
    }

    /// 窗口内请求数（使用预计算时间戳，避免重复 get_elapsed_ms）
    #[inline]
    fn get_request_count_at(&self, now_ms: u64) -> u64 {
        let window_start_ms = now_ms.saturating_sub(self.window_ms);
        let mut total_requests = 0u64;
        for i in 0..self.bucket_count {
            let bucket_start_ms = self.bucket_timestamps[i].load(Ordering::Acquire);
            if bucket_start_ms >= window_start_ms {
                total_requests = total_requests.saturating_add(self.request_buckets[i].load(Ordering::Relaxed));
            }
        }
        total_requests
    }

    /// 窗口内请求数（无预计算时间戳版本，测试用）
    #[allow(dead_code)]
    #[inline]
    fn get_request_count(&self) -> u64 {
        self.get_request_count_at(self.get_elapsed_ms())
    }

    #[inline]
    fn get_bucket_index(&self, now_ms: u64) -> usize {
        (now_ms / self.bucket_ms % self.bucket_count as u64) as usize
    }

    fn update_bucket_timestamp(&self, bucket_idx: usize, now_ms: u64) {
        let bucket_start_ms = self.bucket_timestamps[bucket_idx].load(Ordering::Acquire);
        let bucket_age = now_ms.saturating_sub(bucket_start_ms);
        if bucket_age > self.window_ms {
            let aligned_now = (now_ms / self.bucket_ms) * self.bucket_ms;
            if self.bucket_timestamps[bucket_idx].compare_exchange(
                bucket_start_ms, aligned_now, Ordering::Release, Ordering::Acquire,
            ).is_ok() {
                self.request_buckets[bucket_idx].store(0, Ordering::Relaxed);
                self.failure_buckets[bucket_idx].store(0, Ordering::Relaxed);
            }
        }
    }

    fn reset(&self) {
        let now_ms = self.get_elapsed_ms();
        let aligned_now = (now_ms / self.bucket_ms) * self.bucket_ms;
        for i in 0..self.bucket_count {
            let new_start_ms = aligned_now.saturating_sub(
                self.bucket_ms.saturating_mul((self.bucket_count - i - 1) as u64)
            );
            self.bucket_timestamps[i].store(new_start_ms, Ordering::Release);
            self.request_buckets[i].store(0, Ordering::Relaxed);
            self.failure_buckets[i].store(0, Ordering::Relaxed);
        }
    }
}

impl Clone for SlidingWindow {
    fn clone(&self) -> Self {
        Self {
            window_ms: self.window_ms,
            bucket_ms: self.bucket_ms,
            bucket_count: self.bucket_count,
            request_buckets: Box::new([
                CacheLinePadded::new(AtomicU64::new(self.request_buckets[0].load(Ordering::Relaxed))),
                CacheLinePadded::new(AtomicU64::new(self.request_buckets[1].load(Ordering::Relaxed))),
                CacheLinePadded::new(AtomicU64::new(self.request_buckets[2].load(Ordering::Relaxed))),
                CacheLinePadded::new(AtomicU64::new(self.request_buckets[3].load(Ordering::Relaxed))),
                CacheLinePadded::new(AtomicU64::new(self.request_buckets[4].load(Ordering::Relaxed))),
                CacheLinePadded::new(AtomicU64::new(self.request_buckets[5].load(Ordering::Relaxed))),
                CacheLinePadded::new(AtomicU64::new(self.request_buckets[6].load(Ordering::Relaxed))),
                CacheLinePadded::new(AtomicU64::new(self.request_buckets[7].load(Ordering::Relaxed))),
                CacheLinePadded::new(AtomicU64::new(self.request_buckets[8].load(Ordering::Relaxed))),
                CacheLinePadded::new(AtomicU64::new(self.request_buckets[9].load(Ordering::Relaxed))),
            ]),
            failure_buckets: Box::new([
                CacheLinePadded::new(AtomicU64::new(self.failure_buckets[0].load(Ordering::Relaxed))),
                CacheLinePadded::new(AtomicU64::new(self.failure_buckets[1].load(Ordering::Relaxed))),
                CacheLinePadded::new(AtomicU64::new(self.failure_buckets[2].load(Ordering::Relaxed))),
                CacheLinePadded::new(AtomicU64::new(self.failure_buckets[3].load(Ordering::Relaxed))),
                CacheLinePadded::new(AtomicU64::new(self.failure_buckets[4].load(Ordering::Relaxed))),
                CacheLinePadded::new(AtomicU64::new(self.failure_buckets[5].load(Ordering::Relaxed))),
                CacheLinePadded::new(AtomicU64::new(self.failure_buckets[6].load(Ordering::Relaxed))),
                CacheLinePadded::new(AtomicU64::new(self.failure_buckets[7].load(Ordering::Relaxed))),
                CacheLinePadded::new(AtomicU64::new(self.failure_buckets[8].load(Ordering::Relaxed))),
                CacheLinePadded::new(AtomicU64::new(self.failure_buckets[9].load(Ordering::Relaxed))),
            ]),
            bucket_timestamps: Box::new([
                CacheLinePadded::new(AtomicU64::new(self.bucket_timestamps[0].load(Ordering::Relaxed))),
                CacheLinePadded::new(AtomicU64::new(self.bucket_timestamps[1].load(Ordering::Relaxed))),
                CacheLinePadded::new(AtomicU64::new(self.bucket_timestamps[2].load(Ordering::Relaxed))),
                CacheLinePadded::new(AtomicU64::new(self.bucket_timestamps[3].load(Ordering::Relaxed))),
                CacheLinePadded::new(AtomicU64::new(self.bucket_timestamps[4].load(Ordering::Relaxed))),
                CacheLinePadded::new(AtomicU64::new(self.bucket_timestamps[5].load(Ordering::Relaxed))),
                CacheLinePadded::new(AtomicU64::new(self.bucket_timestamps[6].load(Ordering::Relaxed))),
                CacheLinePadded::new(AtomicU64::new(self.bucket_timestamps[7].load(Ordering::Relaxed))),
                CacheLinePadded::new(AtomicU64::new(self.bucket_timestamps[8].load(Ordering::Relaxed))),
                CacheLinePadded::new(AtomicU64::new(self.bucket_timestamps[9].load(Ordering::Relaxed))),
            ]),
            created_at: self.created_at,
            cached_time_ms: CacheLinePadded::new(AtomicU64::new(self.cached_time_ms.load(Ordering::Relaxed))),
            cache_updated_at_ns: CacheLinePadded::new(AtomicU64::new(self.cache_updated_at_ns.load(Ordering::Relaxed))),
        }
    }
}

// ============================================================
// 内部类型: FuseDimension (标签维度状态)
// ============================================================

/// 标签维度状态 — 每个维度键对应一个实例
///
/// 维度键: use_ip=true → "{tag}:{ip}", use_ip=false → "{tag}"
struct FuseDimension {
    /// 每个阈值一个滑动窗口，与 rules 等长
    windows: Vec<SlidingWindow>,
    /// 熔断状态 (0=Closed, 1=Open, 2=HalfOpen)
    state: AtomicU64,
    /// 熔断开始时间戳(距离创建时的毫秒数)
    circuit_opened_at_ms: AtomicU64,
    /// 哪个阈值规则触发的熔断 (索引)
    triggered_rule_index: AtomicU64,
    /// 半开探测已使用配额
    half_open_used: AtomicU64,
    /// 最后访问时间戳(距离创建时的毫秒数)
    last_access_ms: AtomicU64,
    /// 创建时间点
    created_at: Instant,
}

#[allow(dead_code)]
impl FuseDimension {
    fn new(rules: &[FuseThreshold]) -> Self {
        let created_at = Instant::now();
        let elapsed_ms = created_at.elapsed().as_millis().min(u64::MAX as u128) as u64;
        let windows = rules.iter().map(|r| SlidingWindow::new(r.window_secs, created_at)).collect();
        Self {
            windows,
            state: AtomicU64::new(0),
            circuit_opened_at_ms: AtomicU64::new(0),
            triggered_rule_index: AtomicU64::new(0),
            half_open_used: AtomicU64::new(0),
            last_access_ms: AtomicU64::new(elapsed_ms),
            created_at,
        }
    }

    fn get_elapsed_ms(&self) -> u64 {
        self.created_at.elapsed().as_millis().min(u64::MAX as u128) as u64
    }

    fn get_state(&self) -> CircuitState {
        match self.state.load(Ordering::Acquire) {
            0 => CircuitState::Closed,
            1 => CircuitState::Open,
            _ => CircuitState::HalfOpen,
        }
    }

    fn try_transition(&self, from: CircuitState, to: CircuitState) -> bool {
        let from_val = match from { CircuitState::Closed => 0, CircuitState::Open => 1, CircuitState::HalfOpen => 2 };
        let to_val = match to { CircuitState::Closed => 0, CircuitState::Open => 1, CircuitState::HalfOpen => 2 };
        self.state.compare_exchange(from_val, to_val, Ordering::AcqRel, Ordering::Acquire).is_ok()
    }

    fn set_circuit_opened_at(&self) {
        // 0 被用作"未打开"哨兵值，因此实际时间戳至少存 1
        let opened = self.get_elapsed_ms().max(1);
        self.circuit_opened_at_ms.store(opened, Ordering::Release);
    }

    fn clear_circuit_opened_at(&self) {
        self.circuit_opened_at_ms.store(0, Ordering::Release);
    }

    fn get_circuit_opened_elapsed(&self) -> Option<Duration> {
        let opened_ms = self.circuit_opened_at_ms.load(Ordering::Acquire);
        if opened_ms == 0 { None } else {
            Some(Duration::from_millis(self.get_elapsed_ms().saturating_sub(opened_ms)))
        }
    }

    fn update_last_access(&self) {
        self.last_access_ms.store(self.get_elapsed_ms(), Ordering::Relaxed);
    }

    fn get_last_access_elapsed(&self) -> Duration {
        let last_ms = self.last_access_ms.load(Ordering::Relaxed);
        Duration::from_millis(self.get_elapsed_ms().saturating_sub(last_ms))
    }

    /// 检查是否允许请求通过
    /// 返回 (allowed, state, remaining_secs, triggered_rule_index)
    fn should_allow(&self, rules: &[FuseThreshold]) -> (bool, CircuitState, u64, usize) {
        let state = self.get_state();
        match state {
            CircuitState::Closed => (true, CircuitState::Closed, 0, 0),
            CircuitState::Open => {
                if let Some(opened_elapsed) = self.get_circuit_opened_elapsed() {
                    let triggered_idx = self.triggered_rule_index.load(Ordering::Acquire) as usize;
                    let triggered_idx = triggered_idx.min(rules.len().saturating_sub(1));
                    let circuit_duration = rules[triggered_idx].circuit_duration_secs;
                    let half_open_max = rules[triggered_idx].half_open_requests;

                    if opened_elapsed >= Duration::from_secs(circuit_duration) {
                        if half_open_max == 0 {
                            // 跳过半开，直接恢复
                            if self.try_transition(CircuitState::Open, CircuitState::Closed) {
                                for w in &self.windows { w.reset(); }
                                self.clear_circuit_opened_at();
                            }
                            return (true, CircuitState::Closed, 0, 0);
                        }
                        // 转 HalfOpen
                        if self.try_transition(CircuitState::Open, CircuitState::HalfOpen) {
                            self.half_open_used.store(0, Ordering::Release);
                            return (true, CircuitState::HalfOpen, 0, triggered_idx);
                        }
                        // CAS 失败，其他线程已转换
                        return (false, CircuitState::HalfOpen, 0, triggered_idx);
                    }
                    let remaining = circuit_duration.saturating_sub(opened_elapsed.as_secs());
                    (false, CircuitState::Open, remaining, triggered_idx)
                } else {
                    // opened_at 丢失（异常情况）→ 强制恢复到 Closed 避免永久封禁
                    warn!("TrafficGuard: Open state but circuit_opened_at_ms is 0, force recovering to Closed");
                    self.try_transition(CircuitState::Open, CircuitState::Closed);
                    for w in &self.windows { w.reset(); }
                    self.clear_circuit_opened_at();
                    (true, CircuitState::Closed, 0, 0)
                }
            }
            CircuitState::HalfOpen => {
                let triggered_idx = self.triggered_rule_index.load(Ordering::Acquire) as usize;
                let triggered_idx = triggered_idx.min(rules.len().saturating_sub(1));
                let half_open_max = rules[triggered_idx].half_open_requests;
                // CAS 递增探测配额 —— 使用 Relaxed 循环 + 单次 AcqRel CAS
                loop {
                    let current = self.half_open_used.load(Ordering::Relaxed);
                    if current >= half_open_max {
                        return (false, CircuitState::HalfOpen, 0, triggered_idx);
                    }
                    match self.half_open_used.compare_exchange_weak(current, current + 1, Ordering::AcqRel, Ordering::Relaxed) {
                        Ok(_) => return (true, CircuitState::HalfOpen, 0, triggered_idx),
                        Err(_) => continue,
                    }
                }
            }
        }
    }

    /// 记录失败，返回触发熔断的规则索引 (None=未触发)
    fn record_failure(&self, rules: &[FuseThreshold]) -> Option<usize> {
        // 预计算时间戳，所有窗口共用
        let now_ms = self.get_elapsed_ms();
        // 向所有窗口写入失败
        for w in &self.windows {
            w.record_failure_at(now_ms);
        }
        // 检查每个窗口是否超过阈值
        for (i, w) in self.windows.iter().enumerate() {
            let failures = w.get_failure_count_at(now_ms);
            if failures >= rules[i].max_failures {
                if self.try_transition(CircuitState::Closed, CircuitState::Open) {
                    self.triggered_rule_index.store(i as u64, Ordering::Release);
                    self.set_circuit_opened_at();
                    return Some(i);
                }
                // 其他线程已触发，不再重复
                return None;
            }
        }
        None
    }

    /// 半开状态下记录失败，重新熔断
    fn record_half_open_failure(&self, rules: &[FuseThreshold]) -> Option<usize> {
        let triggered_idx = self.triggered_rule_index.load(Ordering::Acquire) as usize;
        let triggered_idx = triggered_idx.min(rules.len().saturating_sub(1));
        if self.try_transition(CircuitState::HalfOpen, CircuitState::Open) {
            self.set_circuit_opened_at();
            return Some(triggered_idx);
        }
        None
    }

    /// 半开状态下记录成功，恢复到 Closed
    fn record_success(&self) -> bool {
        if self.get_state() == CircuitState::HalfOpen
            && self.try_transition(CircuitState::HalfOpen, CircuitState::Closed)
        {
            for w in &self.windows { w.reset(); }
            self.clear_circuit_opened_at();
            return true;
        }
        false
    }
}

impl Clone for FuseDimension {
    fn clone(&self) -> Self {
        Self {
            windows: self.windows.clone(),
            state: AtomicU64::new(self.state.load(Ordering::Acquire)),
            circuit_opened_at_ms: AtomicU64::new(self.circuit_opened_at_ms.load(Ordering::Acquire)),
            triggered_rule_index: AtomicU64::new(self.triggered_rule_index.load(Ordering::Acquire)),
            half_open_used: AtomicU64::new(self.half_open_used.load(Ordering::Acquire)),
            last_access_ms: AtomicU64::new(self.last_access_ms.load(Ordering::Relaxed)),
            created_at: self.created_at,
        }
    }
}

// ============================================================
// 内部类型: IpDimension (IP 限流维度)
// ============================================================

/// IP 限流维度 — 每个 IP 对应一个实例
struct IpDimension {
    /// 每个阈值一个滑动窗口，与 rules 等长
    windows: Vec<SlidingWindow>,
    state: AtomicU64,
    circuit_opened_at_ms: AtomicU64,
    /// 哪个阈值规则触发的封禁 (索引)
    triggered_rule_index: AtomicU64,
    last_access_ms: AtomicU64,
    created_at: Instant,
}

impl IpDimension {
    fn new(rules: &[IpThreshold]) -> Self {
        let created_at = Instant::now();
        let elapsed_ms = created_at.elapsed().as_millis().min(u64::MAX as u128) as u64;
        let windows = rules.iter().map(|r| SlidingWindow::new(r.window_secs, created_at)).collect();
        Self {
            windows,
            state: AtomicU64::new(0),
            circuit_opened_at_ms: AtomicU64::new(0),
            triggered_rule_index: AtomicU64::new(0),
            last_access_ms: AtomicU64::new(elapsed_ms),
            created_at,
        }
    }

    fn get_elapsed_ms(&self) -> u64 {
        self.created_at.elapsed().as_millis().min(u64::MAX as u128) as u64
    }

    fn get_state(&self) -> CircuitState {
        match self.state.load(Ordering::Acquire) {
            0 => CircuitState::Closed,
            _ => CircuitState::Open,
        }
    }

    fn try_transition(&self, from: CircuitState, to: CircuitState) -> bool {
        let from_val = match from { CircuitState::Closed => 0, _ => 1 };
        let to_val = match to { CircuitState::Closed => 0, _ => 1 };
        self.state.compare_exchange(from_val, to_val, Ordering::AcqRel, Ordering::Acquire).is_ok()
    }

    fn set_circuit_opened_at(&self) {
        // 0 被用作"未打开"哨兵值，因此实际时间戳至少存 1
        let opened = self.get_elapsed_ms().max(1);
        self.circuit_opened_at_ms.store(opened, Ordering::Release);
    }

    fn clear_circuit_opened_at(&self) {
        self.circuit_opened_at_ms.store(0, Ordering::Release);
    }

    fn get_circuit_opened_elapsed(&self) -> Option<Duration> {
        let opened_ms = self.circuit_opened_at_ms.load(Ordering::Acquire);
        if opened_ms == 0 { None } else {
            Some(Duration::from_millis(self.get_elapsed_ms().saturating_sub(opened_ms)))
        }
    }

    fn update_last_access(&self) {
        self.last_access_ms.store(self.get_elapsed_ms(), Ordering::Relaxed);
    }

    fn get_last_access_elapsed(&self) -> Duration {
        let last_ms = self.last_access_ms.load(Ordering::Relaxed);
        Duration::from_millis(self.get_elapsed_ms().saturating_sub(last_ms))
    }

    /// 检查是否允许请求
    /// 返回 (allowed, state, remaining_secs)
    fn should_allow(&self, rules: &[IpThreshold]) -> (bool, CircuitState, u64) {
        let state = self.get_state();
        match state {
            CircuitState::Closed => (true, CircuitState::Closed, 0),
            CircuitState::Open => {
                if let Some(opened_elapsed) = self.get_circuit_opened_elapsed() {
                    let triggered_idx = self.triggered_rule_index.load(Ordering::Acquire) as usize;
                    let triggered_idx = triggered_idx.min(rules.len().saturating_sub(1));
                    let circuit_duration = rules[triggered_idx].circuit_duration_secs;

                    if opened_elapsed >= Duration::from_secs(circuit_duration) {
                        // IP 限流无 HalfOpen，到期直接恢复 Closed
                        if self.try_transition(CircuitState::Open, CircuitState::Closed) {
                            for w in &self.windows { w.reset(); }
                            self.clear_circuit_opened_at();
                        }
                        return (true, CircuitState::Closed, 0);
                    }
                    let remaining = circuit_duration.saturating_sub(opened_elapsed.as_secs());
                    (false, CircuitState::Open, remaining)
                } else {
                    // opened_at 丢失（异常情况）→ 强制恢复到 Closed 避免永久封禁
                    warn!("TrafficGuard: IpDimension Open but circuit_opened_at_ms is 0, force recovering");
                    self.try_transition(CircuitState::Open, CircuitState::Closed);
                    for w in &self.windows { w.reset(); }
                    self.clear_circuit_opened_at();
                    (true, CircuitState::Closed, 0)
                }
            }
            // IpDimension 只有 Closed/Open 两态，不会出现 HalfOpen
            _ => (false, CircuitState::Open, 0),
        }
    }

    /// 记录请求，检查是否超限，返回触发的规则索引 (None=未超限)
    fn record_request(&self, rules: &[IpThreshold]) -> Option<usize> {
        // 预计算时间戳，所有窗口共用
        let now_ms = self.get_elapsed_ms();
        // 向所有窗口写入请求
        for w in &self.windows {
            w.record_request_at(now_ms);
        }
        // 检查每个窗口是否超过阈值（> 表示超过 max_requests 才触发，允许恰好 max_requests 次）
        for (i, w) in self.windows.iter().enumerate() {
            let count = w.get_request_count_at(now_ms);
            if count > rules[i].max_requests {
                if self.try_transition(CircuitState::Closed, CircuitState::Open) {
                    self.triggered_rule_index.store(i as u64, Ordering::Release);
                    self.set_circuit_opened_at();
                    return Some(i);
                }
                // 其他线程已触发，不再重复
                return None;
            }
        }
        None
    }
}

impl Clone for IpDimension {
    fn clone(&self) -> Self {
        Self {
            windows: self.windows.clone(),
            state: AtomicU64::new(self.state.load(Ordering::Acquire)),
            circuit_opened_at_ms: AtomicU64::new(self.circuit_opened_at_ms.load(Ordering::Acquire)),
            triggered_rule_index: AtomicU64::new(self.triggered_rule_index.load(Ordering::Acquire)),
            last_access_ms: AtomicU64::new(self.last_access_ms.load(Ordering::Relaxed)),
            created_at: self.created_at,
        }
    }
}

// ============================================================
// 配置
// ============================================================

#[derive(Debug, Clone)]
pub(crate) struct TrafficGuardConfig {
    fuse_rules: Vec<FuseTagRule>,
    ip_throttle: Option<IpThrottle>,
    max_dimensions: usize,
    dimension_expire_secs: u64,
    cleanup_interval_secs: u64,
}

// ============================================================
// Builder
// ============================================================

#[derive(Debug, Clone)]
pub struct TrafficGuardBuilder {
    fuse_rules: Vec<FuseTagRule>,
    ip_throttle: Option<IpThrottle>,
    max_dimensions: usize,
    dimension_expire_secs: u64,
    cleanup_interval_secs: u64,
}

#[allow(dead_code)]
impl TrafficGuardBuilder {
    pub fn new() -> Self {
        Self {
            fuse_rules: Vec::new(),
            ip_throttle: None,
            max_dimensions: 10_000_000,
            dimension_expire_secs: 86400,
            cleanup_interval_secs: 60,
        }
    }

    /// 追加标签熔断规则
    pub fn fuse_rule(mut self, rule: FuseTagRule) -> Self {
        self.fuse_rules.push(rule);
        self
    }

    /// 批量追加标签熔断规则
    pub fn fuse_rules(mut self, rules: Vec<FuseTagRule>) -> Self {
        self.fuse_rules.extend(rules);
        self
    }

    /// 设置 IP 限流
    pub fn ip_throttle(mut self, config: IpThrottle) -> Self {
        self.ip_throttle = Some(config);
        self
    }

    /// 最大维度数（防内存泄漏，默认千万级）
    pub fn max_dimensions(mut self, n: usize) -> Self {
        self.max_dimensions = n;
        self
    }

    /// 维度过期时间(秒，默认86400=24小时)
    pub fn dimension_expire_secs(mut self, secs: u64) -> Self {
        self.dimension_expire_secs = secs;
        self
    }

    /// 清理间隔(秒，默认60)
    pub fn cleanup_interval_secs(mut self, secs: u64) -> Self {
        self.cleanup_interval_secs = secs;
        self
    }

    pub fn build(mut self) -> TrafficGuard {
        self.sanitize_rules();
        self.validate();
        let config = TrafficGuardConfig {
            fuse_rules: self.fuse_rules,
            ip_throttle: self.ip_throttle,
            max_dimensions: self.max_dimensions,
            dimension_expire_secs: self.dimension_expire_secs,
            cleanup_interval_secs: self.cleanup_interval_secs,
        };
        TrafficGuard::with_config(config)
    }

    fn sanitize_rules(&mut self) {
        self.fuse_rules.retain(|rule| {
            let drop_rule = match rule.tag {
                FuseTag::Exact(s) | FuseTag::Prefix(s) => s.trim().is_empty(),
            };
            if drop_rule {
                warn!("TrafficGuard: dropping fuse rule with empty tag");
                return false;
            }
            true
        });

        if let Some(ip) = self.ip_throttle.as_mut() {
            ip.path_rules.retain(|rule| {
                match rule.path {
                    IpPath::Exact(s) | IpPath::Prefix(s) => {
                        let normalized = s.trim().trim_matches('/');
                        if normalized.is_empty() {
                            warn!(
                                "TrafficGuard: dropping ip_throttle path rule with empty path pattern ({})",
                                rule.path.kind_name()
                            );
                            return false;
                        }
                        true
                    }
                    IpPath::None => true,
                }
            });

            if ip.path_rules.is_empty() {
                warn!(
                    "TrafficGuard: dropping ip_throttle config because path_rules is empty"
                );
                self.ip_throttle = None;
            }
        }
    }

    fn validate(&self) {
        if self.fuse_rules.is_empty() {
            panic!("TrafficGuard: fuse_rules must not be empty — at least one FuseTagRule is required");
        }
        for (i, rule) in self.fuse_rules.iter().enumerate() {
            if rule.rules.is_empty() {
                panic!(
                    "TrafficGuard: fuse_rules[{}].rules must not be empty — tag {:?} needs at least one FuseThreshold",
                    i, rule.tag
                );
            }
            for (j, threshold) in rule.rules.iter().enumerate() {
                if threshold.window_secs < 10 {
                    panic!(
                        "TrafficGuard: fuse_rules[{}].rules[{}].window_secs must be >= 10 (got {})",
                        i, j, threshold.window_secs
                    );
                }
                if threshold.max_failures == 0 {
                    panic!(
                        "TrafficGuard: fuse_rules[{}].rules[{}].max_failures must be > 0 (got {})",
                        i, j, threshold.max_failures
                    );
                }
                if threshold.circuit_duration_secs == 0 {
                    panic!(
                        "TrafficGuard: fuse_rules[{}].rules[{}].circuit_duration_secs must be > 0 (got {})",
                        i, j, threshold.circuit_duration_secs
                    );
                }
            }
        }
        if let Some(ref ip) = self.ip_throttle {
            for (i, rule) in ip.path_rules.iter().enumerate() {
                if rule.rules.is_empty() {
                    panic!(
                        "TrafficGuard: ip_throttle.path_rules[{}].rules must not be empty",
                        i
                    );
                }
                for (j, threshold) in rule.rules.iter().enumerate() {
                    if threshold.window_secs < 10 {
                        panic!(
                            "TrafficGuard: ip_throttle.path_rules[{}].rules[{}].window_secs must be >= 10 (got {})",
                            i, j, threshold.window_secs
                        );
                    }
                    if threshold.max_requests == 0 {
                        panic!(
                            "TrafficGuard: ip_throttle.path_rules[{}].rules[{}].max_requests must be > 0 (got {})",
                            i, j, threshold.max_requests
                        );
                    }
                    if threshold.circuit_duration_secs == 0 {
                        panic!(
                            "TrafficGuard: ip_throttle.path_rules[{}].rules[{}].circuit_duration_secs must be > 0 (got {})",
                            i, j, threshold.circuit_duration_secs
                        );
                    }
                }
            }

        }
    }
}

impl Default for TrafficGuardBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// TrafficGuard
// ============================================================

pub struct TrafficGuard {
    config: TrafficGuardConfig,
    /// 标签维度: key = "{tag}" 或 "{tag}:{ip}"
    dimensions: Arc<DashMap<String, FuseDimension>>,
    /// IP 限流维度: key = "{rule_id}:{ip}"（按规则+IP聚合）
    ip_dimensions: Arc<DashMap<String, IpDimension>>,
    /// IP → 标签维度键集合（用于按 IP 清理所有关联维度）
    ip_dimension_keys: Arc<DashMap<String, HashSet<String>>>,
    last_cleanup: Arc<AtomicU64>,
    dimension_count: Arc<AtomicU64>,
}

impl std::fmt::Debug for TrafficGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TrafficGuard")
            .field("fuse_rules_count", &self.config.fuse_rules.len())
            .field("ip_throttle", &self.config.ip_throttle.is_some())
            .field("dimensions_count", &self.dimensions.len())
            .field("ip_dimensions_count", &self.ip_dimensions.len())
            .finish()
    }
}

#[derive(Debug, Clone, Copy)]
enum IpRuleScope {
    Exact,
    Prefix,
    None,
}

#[derive(Debug, Clone)]
struct MatchedIpThrottleRule {
    scope: IpRuleScope,
    pattern: Option<&'static str>,
    rules: Vec<IpThreshold>,
}

#[allow(dead_code)]
impl TrafficGuard {
    pub fn with_config(config: TrafficGuardConfig) -> Self {
        Self {
            config,
            dimensions: Arc::new(DashMap::new()),
            ip_dimensions: Arc::new(DashMap::new()),
            ip_dimension_keys: Arc::new(DashMap::new()),
            last_cleanup: Arc::new(AtomicU64::new(0)),
            dimension_count: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn builder() -> TrafficGuardBuilder {
        TrafficGuardBuilder::new()
    }

    /// 获取客户端 IP
    fn get_client_ip(req: &ServiceRequest) -> String {
        req.peer_addr()
            .map(|addr| addr.ip().to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }

    /// 匹配规则: 首次匹配胜出
    fn match_rule(&self, tag_value: &str) -> Option<(usize, &FuseTagRule)> {
        for (i, rule) in self.config.fuse_rules.iter().enumerate() {
            if rule.tag.matches(tag_value) {
                return Some((i, rule));
            }
        }
        None
    }

    /// 生成维度键
    fn make_dimension_key(tag_value: &str, use_ip: bool, ip: &str) -> String {
        if use_ip {
            format!("{}:{}", tag_value, ip)
        } else {
            tag_value.to_string()
        }
    }

    fn fnv1a64(bytes: &[u8]) -> u64 {
        let mut hash = 0xcbf29ce484222325u64;
        for b in bytes {
            hash ^= *b as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash
    }

    fn make_ip_rule_id(scope: IpRuleScope, pattern: Option<&str>, rules: &[IpThreshold]) -> String {
        let scope_str = match scope {
            IpRuleScope::Exact => "exact",
            IpRuleScope::Prefix => "prefix",
            IpRuleScope::None => "none",
        };
        let pattern = pattern.unwrap_or("NONE").trim();
        // 将所有阈值拼入 payload，确保不同规则集产生不同 rule_id
        let rules_payload: String = rules.iter()
            .map(|r| format!("{}|{}|{}", r.window_secs, r.max_requests, r.circuit_duration_secs))
            .collect::<Vec<_>>()
            .join(";");
        let payload = format!("{}|{}|{}", scope_str, pattern, rules_payload);
        let hash = Self::fnv1a64(payload.as_bytes());
        format!("iprl_{hash:016x}")
    }

    fn make_ip_dimension_key(ip: &str, rule_id: &str) -> String {
        format!("{}:{}", rule_id, ip)
    }

    fn resolve_ip_rule(&self, path: &str) -> Option<MatchedIpThrottleRule> {
        let throttle = self.config.ip_throttle.as_ref()?;

        for rule in &throttle.path_rules {
            if matches!(rule.path, IpPath::Exact(_)) && rule.path.matches(path) {
                return Some(MatchedIpThrottleRule {
                    scope: IpRuleScope::Exact,
                    pattern: rule.path.as_pattern(),
                    rules: rule.rules.clone(),
                });
            }
        }
        for rule in &throttle.path_rules {
            if matches!(rule.path, IpPath::Prefix(_)) && rule.path.matches(path) {
                return Some(MatchedIpThrottleRule {
                    scope: IpRuleScope::Prefix,
                    pattern: rule.path.as_pattern(),
                    rules: rule.rules.clone(),
                });
            }
        }
        for rule in &throttle.path_rules {
            if matches!(rule.path, IpPath::None) {
                return Some(MatchedIpThrottleRule {
                    scope: IpRuleScope::None,
                    pattern: None,
                    rules: rule.rules.clone(),
                });
            }
        }

        None
    }

    /// IP 限流前置检查
    fn check_ip_throttle(&self, ip: &str, path: &str) -> Option<(bool, CircuitState, u64, String)> {
        let matched = self.resolve_ip_rule(path)?;
        let rule_id = Self::make_ip_rule_id(matched.scope, matched.pattern, &matched.rules);
        let dim_key = Self::make_ip_dimension_key(ip, &rule_id);

        // 快速路径：维度已存在
        if let Some(entry) = self.ip_dimensions.get(&dim_key) {
            entry.update_last_access();
            let (allowed, state, remaining) = entry.should_allow(&matched.rules);
            if !allowed {
                let triggered = format!(
                    "{}:{};{}-{}",
                    IP_THROTTLE_TAG, rule_id, matched.rules[0].window_secs, matched.rules[0].max_requests
                );
                return Some((false, state, remaining, triggered));
            }
            let _ = entry.record_request(&matched.rules);
            return Some((true, CircuitState::Closed, 0, String::new()));
        }

        // 慢路径：需要插入新维度
        let (allowed, state, remaining, triggered) = {
            let entry = self
                .ip_dimensions
                .entry(dim_key)
                .or_insert_with(|| IpDimension::new(&matched.rules));
            entry.update_last_access();
            let (allowed, state, remaining) = entry.should_allow(&matched.rules);
            if !allowed {
                let triggered = format!(
                    "{}:{};{}-{}",
                    IP_THROTTLE_TAG, rule_id, matched.rules[0].window_secs, matched.rules[0].max_requests
                );
                (false, state, remaining, triggered)
            } else {
                let _ = entry.record_request(&matched.rules);
                (true, CircuitState::Closed, 0, String::new())
            }
        };

        if !allowed {
            return Some((false, state, remaining, triggered));
        }
        Some((true, CircuitState::Closed, 0, String::new()))
    }

    /// 标签维度前置检查
    fn check_fuse_dimension(&self, dimension_key: &str, rule: &FuseTagRule) -> (bool, CircuitState, u64, usize) {
        // 快速路径：如果维度已存在，直接访问（避免计数检查开销）
        if let Some(entry) = self.dimensions.get(dimension_key) {
            entry.update_last_access();
            return entry.should_allow(&rule.rules);
        }

        // 慢路径：检查维度数量限制后插入
        let current_count = self.dimension_count.load(Ordering::Relaxed);
        if current_count >= self.config.max_dimensions as u64 {
            // 二次确认（Relaxed 可能读到过时值）
            if !self.dimensions.contains_key(dimension_key) {
                return (false, CircuitState::Open, 0, 0);
            }
        }

        let entry = self.dimensions.entry(dimension_key.to_string()).or_insert_with(|| {
            self.dimension_count.fetch_add(1, Ordering::Relaxed);
            FuseDimension::new(&rule.rules)
        });
        entry.update_last_access();
        entry.should_allow(&rule.rules)
    }

    /// 标签维度记录失败
    fn record_fuse_failure(&self, dimension_key: &str, rule: &FuseTagRule) -> Option<usize> {
        if let Some(entry) = self.dimensions.get(dimension_key) {
            entry.record_failure(&rule.rules)
        } else {
            None
        }
    }

    /// 标签维度半开失败
    fn record_fuse_half_open_failure(&self, dimension_key: &str, rule: &FuseTagRule) -> Option<usize> {
        if let Some(entry) = self.dimensions.get(dimension_key) {
            entry.record_half_open_failure(&rule.rules)
        } else {
            None
        }
    }

    /// 标签维度记录成功
    fn record_fuse_success(&self, dimension_key: &str) -> bool {
        if let Some(entry) = self.dimensions.get(dimension_key) {
            entry.record_success()
        } else {
            false
        }
    }

    /// 标签熔断前置检查 —— 检查当前 IP 关联的维度 + use_ip=false 的 Exact 维度
    /// 返回 None=全部放行, Some=(state, remaining_secs, triggered_info)
    fn check_fuse_precheck(&self, ip: &str) -> Option<(CircuitState, u64, String)> {
        // 预计算 IP 后缀，避免循环内重复分配
        let ip_suffix = format!(":{}", ip);

        // ── ① 检查 IP 关联的维度（use_ip=true 的规则）──
        if let Some(keys) = self.ip_dimension_keys.get(ip) {
            for dim_key in keys.iter() {
                // 从维度键中提取 tag_value（格式: "{tag}:{ip}"，注意 IPv6 含冒号，
                // 所以不直接用 strip_suffix，而是通过 match_rule 反向匹配）
                let tag_value = if dim_key.ends_with(&ip_suffix) {
                    &dim_key[..dim_key.len() - ip_suffix.len()]
                } else {
                    dim_key.as_str()
                };

                if let Some((_rule_idx, rule)) = self.match_rule(tag_value) {
                    if let Some(rejection) = self.try_reject(dim_key, rule, tag_value) {
                        return Some(rejection);
                    }
                }
            }
        }

        // ── ② 检查非 IP 隔离的 Exact 维度（use_ip=false，不在 ip_dimension_keys 中）──
        for rule in &self.config.fuse_rules {
            if !rule.use_ip {
                if let FuseTag::Exact(tag) = &rule.tag {
                    let dim_key = TrafficGuard::make_dimension_key(tag, false, ip);
                    if let Some(rejection) = self.try_reject(&dim_key, rule, tag) {
                        return Some(rejection);
                    }
                }
                // Prefix + use_ip=false: 无法预知 tag_value，跳过（文档限制）
            }
        }

        None
    }

    /// 对单个维度执行 should_allow 检查，拒绝时返回 Some
    #[inline]
    fn try_reject(
        &self,
        dim_key: &str,
        rule: &FuseTagRule,
        tag_value: &str,
    ) -> Option<(CircuitState, u64, String)> {
        let (allowed, state, remaining, triggered_idx) =
            self.check_fuse_dimension(dim_key, rule);
        if !allowed {
            let triggered_info = if triggered_idx < rule.rules.len() {
                let r = &rule.rules[triggered_idx];
                format!("{};{}-{}", tag_value, r.window_secs, r.max_failures)
            } else {
                format!("{};unknown", tag_value)
            };
            Some((state, remaining, triggered_info))
        } else {
            None
        }
    }

    /// 半开恢复 —— 遍历 IP 关联的所有维度，对 HalfOpen 状态的维度尝试恢复
    fn recover_half_open_for_ip(&self, ip: &str) {
        if let Some(keys) = self.ip_dimension_keys.get(ip) {
            for dim_key in keys.iter() {
                self.record_fuse_success(dim_key);
            }
        }
    }

    /// 注册 IP → 维度键映射
    fn register_ip_dimension(&self, ip: &str, dimension_key: &str) {
        // 使用 HashSet 实现 O(1) 去重插入，替代原 Vec::contains 的 O(n) 扫描
        let mut entry = self.ip_dimension_keys.entry(ip.to_string()).or_insert_with(HashSet::new);
        entry.insert(dimension_key.to_string());
    }

    /// 构建拒绝响应 — 使用 JsonData+JsonResponse 保证输出格式统一
    fn build_rejection_response(&self, state: CircuitState, remaining_secs: u64, triggered_info: Option<&str>) -> HttpResponse {
        let (message, retry_after, sub_code) = match state {
            CircuitState::Open => ("请求过于频繁，请稍后再试", remaining_secs, "circuit_open"),
            CircuitState::HalfOpen => ("服务正在恢复中，请稍后再试", 10, "circuit_half_open"),
            CircuitState::Closed => ("", 0, "circuit_closed"),
        };

        let json_data = JsonData::default()
            .set_code(429)
            .set_sub_code(sub_code)
            .set_body(serde_json::json!({
                "retry_after": retry_after,
                "circuit_state": format!("{:?}", state),
            }));

        let json_response = JsonResponse::data(json_data).set_message(message);

        let mut builder = HttpResponse::build(StatusCode::TOO_MANY_REQUESTS);
        builder.insert_header(("Retry-After", retry_after.to_string()));
        builder.insert_header(("X-Circuit-State", format!("{:?}", state)));

        if let Some(info) = triggered_info {
            builder.insert_header((X_FUSE_TRIGGERED.clone(), info.to_string()));
        }

        builder.json(json_response.to_value())
    }

    /// 清理过期数据
    pub fn cleanup(&self) {
        let now_secs = Instant::now().elapsed().as_secs();
        let last = self.last_cleanup.swap(now_secs, Ordering::Relaxed);
        if now_secs.saturating_sub(last) < self.config.cleanup_interval_secs {
            return;
        }

        let expire_duration = Duration::from_secs(self.config.dimension_expire_secs);

        // 清理标签维度
        let before = self.dimensions.len();
        self.dimensions.retain(|_, dim| dim.get_last_access_elapsed() < expire_duration);
        let after = self.dimensions.len();
        if before > after {
            let removed = before.saturating_sub(after) as u64;
            self.dimension_count.fetch_sub(removed, Ordering::Relaxed);
        }

        // 清理 IP 限流维度
        self.ip_dimensions.retain(|_, dim| dim.get_last_access_elapsed() < expire_duration);

        // 清理 IP → 维度键映射（移除已不存在的维度键引用）
        self.ip_dimension_keys.retain(|ip, keys| {
            keys.retain(|k| self.dimensions.contains_key(k));
            // 如果 IP 限流维度也不存在了，且无关联标签维度，则移除
            if keys.is_empty() && !self.ip_dimensions.contains_key(ip) {
                false
            } else {
                true
            }
        });
    }

    /// 轻量级清理门控 —— 使用 Relaxed 原子，即使多线程竞争也不过是多执行几次清理
    fn maybe_cleanup(&self) {
        let now_secs = Instant::now().elapsed().as_secs();
        let last = self.last_cleanup.load(Ordering::Relaxed);
        if now_secs.saturating_sub(last) >= self.config.cleanup_interval_secs {
            // CAS 失败说明其他线程正在清理，直接跳过（无需等待）
            if self.last_cleanup.compare_exchange(last, now_secs, Ordering::Relaxed, Ordering::Relaxed).is_ok() {
                self.cleanup();
            }
        }
    }

    pub fn force_cleanup(&self) {
        let now_secs = Instant::now().elapsed().as_secs();
        self.last_cleanup.store(now_secs, Ordering::Relaxed);
        self.cleanup();
    }
}

impl Clone for TrafficGuard {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            dimensions: Arc::clone(&self.dimensions),
            ip_dimensions: Arc::clone(&self.ip_dimensions),
            ip_dimension_keys: Arc::clone(&self.ip_dimension_keys),
            last_cleanup: Arc::clone(&self.last_cleanup),
            dimension_count: Arc::clone(&self.dimension_count),
        }
    }
}

// ============================================================
// Transform + Middleware
// ============================================================

impl<S, B> Transform<S, ServiceRequest> for TrafficGuard
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Transform = TrafficGuardMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(TrafficGuardMiddleware {
            service,
            traffic_guard: self.clone(),
        }))
    }
}

pub struct TrafficGuardMiddleware<S> {
    service: S,
    traffic_guard: TrafficGuard,
}

impl<S, B> Service<ServiceRequest> for TrafficGuardMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    actix_service::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        self.traffic_guard.maybe_cleanup();

        let ip = TrafficGuard::get_client_ip(&req);
        let path = req.path().to_string();

        // ── ① IP 限流前置检查 ──
        if let Some((allowed, state, remaining, triggered)) = self.traffic_guard.check_ip_throttle(&ip, &path) {
            if !allowed {
                let response = self.traffic_guard.build_rejection_response(state, remaining, Some(&triggered));
                return Box::pin(async move {
                    Ok(req.into_response(response.map_into_boxed_body()))
                });
            }
        }

        // ── ② 标签熔断前置检查 ──
        if let Some((state, remaining, triggered)) = self.traffic_guard.check_fuse_precheck(&ip) {
            let response = self.traffic_guard.build_rejection_response(state, remaining, Some(&triggered));
            return Box::pin(async move {
                Ok(req.into_response(response.map_into_boxed_body()))
            });
        }

        let traffic_guard = self.traffic_guard.clone();
        let fut = self.service.call(req);

        Box::pin(async move {
            let result = fut.await;

            match &result {
                Ok(response) => {
                    // ── ③ 读取 X-Fuse 头 ──
                    let fuse_tag = response.headers().get(&X_FUSE)
                        .and_then(|v| v.to_str().ok())
                        .map(|s| s.to_string());

                    if let Some(tag_value) = fuse_tag {
                        let tag_value = tag_value.trim().to_string();
                        if tag_value.is_empty() {
                            warn!("TrafficGuard: got empty X-Fuse tag, ignore this response mark");
                            return result.map(|res| res.map_into_boxed_body());
                        }

                        // ── ④ 规则匹配 ──
                        if let Some((_rule_idx, rule)) = traffic_guard.match_rule(&tag_value) {
                            let dimension_key = TrafficGuard::make_dimension_key(
                                &tag_value, rule.use_ip, &ip,
                            );

                            // 注册 IP → 维度键映射
                            if rule.use_ip {
                                traffic_guard.register_ip_dimension(&ip, &dimension_key);
                            }

                            // ── ⑤ 记录失败 ──
                            if let Some(entry) = traffic_guard.dimensions.get(&dimension_key) {
                                let dim_state = entry.get_state();
                                match dim_state {
                                    CircuitState::Closed => {
                                        if let Some(triggered_idx) = entry.record_failure(&rule.rules) {
                                            let _ = triggered_idx;
                                        }
                                    }
                                    CircuitState::HalfOpen => {
                                        // 探测失败 → 重新熔断
                                        entry.record_half_open_failure(&rule.rules);
                                    }
                                    _ => {}
                                }
                            }
                        }
                        // 无匹配规则 → 旁路，不记录
                    } else {
                        // ── ⑥ 响应成功（无 X-Fuse 头）→ 半开恢复 ──
                        traffic_guard.recover_half_open_for_ip(&ip);
                    }
                }
                Err(_) => {
                    // 内部服务错误 —— 无法确定是否"业务成功"，不触发半开恢复
                }
            }

            result.map(|res| res.map_into_boxed_body())
        })
    }
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuse_tag_matching() {
        let exact = FuseTag::Exact("SMS_SEND");
        assert!(exact.matches("SMS_SEND"));
        assert!(!exact.matches("SMS_SEND_U1"));
        assert!(!exact.matches("SMS"));

        let prefix = FuseTag::Prefix("LOGIN_");
        assert!(prefix.matches("LOGIN_U1"));
        assert!(prefix.matches("LOGIN_U2"));
        assert!(!prefix.matches("LOGOUT_U1"));
    }

    #[test]
    fn test_fuse_tag_to_tag() {
        // Exact: 忽略 suffix
        let exact = FuseTag::Exact("SMS_SEND");
        assert_eq!(exact.to_tag(None), "SMS_SEND");
        assert_eq!(exact.to_tag(Some("ignored")), "SMS_SEND");

        // Prefix: 拼接 suffix
        let prefix = FuseTag::Prefix("LOGIN_");
        assert_eq!(prefix.to_tag(Some("U1")), "LOGIN_U1");
        assert_eq!(prefix.to_tag(Some("U2")), "LOGIN_U2");
        assert_eq!(prefix.to_tag(None), "LOGIN_");
    }

    #[test]
    fn test_builder_validation() {
        // 空规则应 panic
        let result = std::panic::catch_unwind(|| {
            TrafficGuard::builder().build();
        });
        assert!(result.is_err());

        // 规则内空阈值应 panic
        let result = std::panic::catch_unwind(|| {
            TrafficGuard::builder()
                .fuse_rule(FuseTagRule {
                    tag: FuseTag::Exact("TEST".into()),
                    use_ip: false,
                    rules: vec![],
                })
                .build();
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_sliding_window() {
        let window = SlidingWindow::new(60, Instant::now());
        for _ in 0..10 {
            window.record(false);
        }
        let (count, failures) = window.get_stats();
        assert_eq!(count, 10);
        assert_eq!(failures, 0);

        for _ in 0..3 {
            window.record(true);
        }
        let (count, failures) = window.get_stats();
        assert_eq!(count, 13);
        assert_eq!(failures, 3);
    }

    #[test]
    fn test_sliding_window_failure_only() {
        let window = SlidingWindow::new(60, Instant::now());
        for _ in 0..5 {
            window.record_failure();
        }
        let failures = window.get_failure_count();
        assert_eq!(failures, 5);
        // record_failure 不增加请求计数
        let requests = window.get_request_count();
        assert_eq!(requests, 0);
    }

    #[test]
    fn test_fuse_dimension_state_transitions() {
        let rules = vec![
            FuseThreshold { window_secs: 60, max_failures: 5, circuit_duration_secs: 30, half_open_requests: 2 },
        ];
        let dim = FuseDimension::new(&rules);

        // 初始 Closed
        assert_eq!(dim.get_state(), CircuitState::Closed);

        // Closed → Open
        assert!(dim.try_transition(CircuitState::Closed, CircuitState::Open));
        assert_eq!(dim.get_state(), CircuitState::Open);
        dim.set_circuit_opened_at();

        // Open → HalfOpen
        assert!(dim.try_transition(CircuitState::Open, CircuitState::HalfOpen));
        assert_eq!(dim.get_state(), CircuitState::HalfOpen);

        // HalfOpen → Closed
        assert!(dim.try_transition(CircuitState::HalfOpen, CircuitState::Closed));
        assert_eq!(dim.get_state(), CircuitState::Closed);
    }

    #[test]
    fn test_fuse_dimension_half_open_zero_skip() {
        let rules = vec![
            FuseThreshold { window_secs: 60, max_failures: 5, circuit_duration_secs: 0, half_open_requests: 0 },
        ];
        let dim = FuseDimension::new(&rules);

        // Closed → Open
        dim.try_transition(CircuitState::Closed, CircuitState::Open);
        dim.set_circuit_opened_at();

        // should_allow: half_open_requests=0 → 直接恢复 Closed
        let (allowed, state, _, _) = dim.should_allow(&rules);
        assert!(allowed);
        assert_eq!(state, CircuitState::Closed);
    }

    #[test]
    fn test_traffic_guard_build() {
        let breaker = TrafficGuard::builder()
            .fuse_rule(FuseTagRule {
                tag: FuseTag::Exact("TEST".into()),
                use_ip: false,
                rules: vec![
                    FuseThreshold { window_secs: 60, max_failures: 10, circuit_duration_secs: 30, half_open_requests: 3 },
                ],
            })
            .build();

        assert_eq!(breaker.config.fuse_rules.len(), 1);
        assert!(breaker.config.ip_throttle.is_none());
    }

    #[test]
    fn test_fuse_header_helper() {
        let mut response = HttpResponse::Ok().finish();
        fuse_header(&mut response, "LOGIN_U1");
        assert_eq!(
            response.headers().get(&X_FUSE).and_then(|v| v.to_str().ok()),
            Some("LOGIN_U1")
        );
    }

    #[test]
    fn test_ip_throttle_build() {
        let breaker = TrafficGuard::builder()
            .fuse_rule(FuseTagRule {
                tag: FuseTag::Prefix("LOGIN_".into()),
                use_ip: true,
                rules: vec![
                    FuseThreshold { window_secs: 60, max_failures: 20, circuit_duration_secs: 300, half_open_requests: 3 },
                ],
            })
            .ip_throttle(IpThrottle {
                path_rules: vec![IpThrottleRule {
                    path: IpPath::None,
                    rules: vec![IpThreshold {
                        window_secs: 60,
                        max_requests: 500,
                        circuit_duration_secs: 1800,
                    }],
                }],
            })
            .build();

        assert_eq!(breaker.config.fuse_rules.len(), 1);
        assert!(breaker.config.ip_throttle.is_some());
        assert_eq!(breaker.config.max_dimensions, 10_000_000);
        assert_eq!(breaker.config.dimension_expire_secs, 86400);
    }

    #[test]
    fn test_ip_throttle_match_priority_exact_prefix_none() {
        let breaker = TrafficGuard::builder()
            .fuse_rule(FuseTagRule {
                tag: FuseTag::Exact("TEST"),
                use_ip: false,
                rules: vec![FuseThreshold {
                    window_secs: 60,
                    max_failures: 10,
                    circuit_duration_secs: 30,
                    half_open_requests: 1,
                }],
            })
            .ip_throttle(IpThrottle {
                path_rules: vec![
                    IpThrottleRule {
                        path: IpPath::Prefix("a"),
                        rules: vec![IpThreshold {
                            window_secs: 10,
                            max_requests: 50,
                            circuit_duration_secs: 60,
                        }],
                    },
                    IpThrottleRule {
                        path: IpPath::Exact("a"),
                        rules: vec![IpThreshold {
                            window_secs: 10,
                            max_requests: 5,
                            circuit_duration_secs: 60,
                        }],
                    },
                    IpThrottleRule {
                        path: IpPath::None,
                        rules: vec![IpThreshold {
                            window_secs: 10,
                            max_requests: 100,
                            circuit_duration_secs: 60,
                        }],
                    },
                ],
            })
            .build();

        let exact = breaker.resolve_ip_rule("/a").expect("exact should match");
        assert!(matches!(exact.scope, IpRuleScope::Exact));

        let prefix = breaker.resolve_ip_rule("/ab").expect("prefix should match");
        assert!(matches!(prefix.scope, IpRuleScope::Prefix));

        let none = breaker.resolve_ip_rule("/zzz").expect("none should fallback");
        assert!(matches!(none.scope, IpRuleScope::None));
    }

    #[test]
    fn test_ip_throttle_rule_id_stability_and_bucket_isolation() {
        let prefix_rules = vec![IpThreshold {
            window_secs: 10,
            max_requests: 50,
            circuit_duration_secs: 60,
        }];
        let exact_rules = vec![IpThreshold {
            window_secs: 10,
            max_requests: 5,
            circuit_duration_secs: 60,
        }];

        let prefix_a = TrafficGuard::make_ip_rule_id(IpRuleScope::Prefix, Some("a"), &prefix_rules);
        let prefix_b = TrafficGuard::make_ip_rule_id(IpRuleScope::Prefix, Some("a"), &prefix_rules);
        let exact_a = TrafficGuard::make_ip_rule_id(IpRuleScope::Exact, Some("a"), &exact_rules);

        assert_eq!(prefix_a, prefix_b, "same prefix rule must map to same rule_id");
        assert_ne!(prefix_a, exact_a, "exact and prefix for same text must be independent buckets");
    }
}