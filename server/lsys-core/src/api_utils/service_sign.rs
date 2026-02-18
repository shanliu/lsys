//! 服务间调用签名计算
//!
//! 签名算法: X-Signature = MD5(service_api_key + X-Timestamp)

use std::time::{SystemTime, UNIX_EPOCH};

/// HTTP Header 名称
pub const SERVICE_TIMESTAMP_HEADER: &str = "X-Timestamp";
pub const SERVICE_SIGNATURE_HEADER: &str = "X-Signature";

/// 签名计算结果
pub struct ServiceSignResult {
    /// 使用的时间戳
    pub timestamp: String,
    /// 待签名的原始字符串（用于调试）
    pub raw_string: String,
    /// 计算得到的签名
    pub signature: String,
}

/// 计算服务间调用签名
/// 
/// # Arguments
/// * `api_key` - 服务密钥
/// * `timestamp` - Unix 时间戳字符串，None 则自动使用当前时间
/// 
/// # Returns
/// ServiceSignResult 包含时间戳、原始字符串和签名结果
pub fn compute_service_sign(api_key: &str, timestamp: Option<&str>) -> ServiceSignResult {
    let timestamp = timestamp
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs().to_string())
                .unwrap_or_else(|_| "0".to_string())
        });
    
    let raw_string = format!("{}{}", api_key, timestamp);
    let digest = md5::compute(raw_string.as_bytes());
    
    ServiceSignResult {
        timestamp,
        raw_string,
        signature: format!("{:x}", digest),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_with_timestamp() {
        let result = compute_service_sign("test_key", Some("1700000000"));
        assert_eq!(result.timestamp, "1700000000");
        assert_eq!(result.raw_string, "test_key1700000000");
        assert!(!result.signature.is_empty());
    }

    #[test]
    fn test_compute_auto_timestamp() {
        let result = compute_service_sign("test_key", None);
        assert!(!result.timestamp.is_empty());
        assert!(!result.signature.is_empty());
    }
}
