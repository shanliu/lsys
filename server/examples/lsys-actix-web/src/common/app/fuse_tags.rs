//! 流量守护标签与规则定义
//!
//! 所有业务标签和流量守护规则集中在此文件定义，
//! 供 `server/mod.rs` 注册中间件、业务 handler 设置 X-Fuse 头时统一引用。
//!
//! # 使用方式
//!
//! **注册中间件**（`server/mod.rs`）：
//! ```rust
//! use crate::common::app::fuse_tags::build_fuse_rules;
//! let traffic_guard = TrafficGuard::builder()
//!     .fuse_rules(build_fuse_rules())
//!     .ip_throttle(build_ip_throttle())
//!     .build();
//! ```
//!
//! **业务 handler 标记失败**：
//! ```rust
//! use crate::common::app::fuse_tags::TAG_SEND_EMAIL;
//! use crate::common::middleware::traffic_guard::fuse_header;
//!
//! fuse_header(&mut response, &TAG_SEND_EMAIL.to_tag(None));
//! ```

use crate::common::middleware::{
    FuseTag, FuseTagRule, FuseThreshold, IpPath, IpThreshold, IpThrottle, IpThrottleRule,
};

// ============================================================
// 标签常量 — 业务端通过这些常量生成 X-Fuse 头值
// ============================================================

/// 发送邮件接口标签（精确匹配）
pub const TAG_SEND_EMAIL: FuseTag = FuseTag::Exact("SEND_EMAIL");

/// 发送短信接口标签（精确匹配）
pub const TAG_SEND_SMS: FuseTag = FuseTag::Exact("SEND_SMS");

// ============================================================
// 规则构建
// ============================================================

/// 构建所有标签熔断规则
///
/// 规则说明：
/// - **发送邮件**：IP 隔离，60s/30次 + 300s/150次，关闭半开
/// - **发送短信**：IP 隔离，60s/30次 + 300s/150次，关闭半开
pub fn build_fuse_rules() -> Vec<FuseTagRule> {
    vec![
        // 1. 发送邮件：IP隔离, 60秒内30次→30分钟熔断, 300秒内150次→12小时熔断, 关闭半开
        FuseTagRule {
            tag: TAG_SEND_EMAIL,
            use_ip: true,
            rules: vec![
                FuseThreshold {
                    window_secs: 60,
                    max_failures: 30,
                    circuit_duration_secs: 1800,
                    half_open_requests: 0,
                },
                FuseThreshold {
                    window_secs: 300,
                    max_failures: 150,
                    circuit_duration_secs: 43200,
                    half_open_requests: 0,
                },
            ],
        },
        // 2. 发送短信：IP隔离, 60秒内30次→30分钟熔断, 300秒内150次→12小时熔断, 关闭半开
        FuseTagRule {
            tag: TAG_SEND_SMS,
            use_ip: true,
            rules: vec![
                FuseThreshold {
                    window_secs: 60,
                    max_failures: 30,
                    circuit_duration_secs: 1800,
                    half_open_requests: 0,
                },
                FuseThreshold {
                    window_secs: 300,
                    max_failures: 150,
                    circuit_duration_secs: 43200,
                    half_open_requests: 0,
                },
            ],
        },
    ]
}

/// 构建 IP 限流配置（路径优先，全局兜底）
///
/// 规则说明：
/// - **登录接口** `/rest/auth`：10s/120次→5分钟封禁 + 60s/500次→4小时封禁
/// - **用户接口** `/api/user`：10s/500次→30分钟封禁
/// - **全局兜底**：10s/2000次→5分钟封禁
pub fn build_ip_throttle() -> IpThrottle {
    IpThrottle {
        // 优先级：Exact > Prefix > None（None 为全局兜底）
        path_rules: vec![
            // 登录接口：按 IP 限流，10秒内120次→5分钟封禁, 60秒内500次→4小时封禁
            IpThrottleRule {
                path: IpPath::Prefix("/rest/auth"),
                rules: vec![
                    IpThreshold {
                        window_secs: 10,
                        max_requests: 120,
                        circuit_duration_secs: 300,
                    },
                    IpThreshold {
                        window_secs: 60,
                        max_requests: 500,
                        circuit_duration_secs: 14400,
                    },
                ],
            },
            // 用户接口：按 IP 限流，10秒内500次→30分钟封禁
            IpThrottleRule {
                path: IpPath::Prefix("/api/user"),
                rules: vec![
                    IpThreshold {
                        window_secs: 10,
                        max_requests: 500,
                        circuit_duration_secs: 1800,
                    },
                ],
            },
            // None 即全局兜底，仅在未命中 Exact/Prefix 时生效
            IpThrottleRule {
                path: IpPath::None,
                rules: vec![
                    IpThreshold {
                        window_secs: 10,
                        max_requests: 2000,
                        circuit_duration_secs: 300,
                    },
                ],
            },
        ],
    }
}
