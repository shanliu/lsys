//! REST API 签名计算
//!
//! 签名算法: MD5(url_encoded_params + json_body + app_key)

use std::collections::BTreeMap;
use serde_json::Value;

/// 签名计算结果
pub struct SignResult {
    /// 待签名的原始字符串（用于调试）
    pub raw_string: String,
    /// 计算得到的签名
    pub signature: String,
}

/// REST API 签名数据结构
pub struct RestSignData<'a> {
    pub client_id: &'a str,
    pub version: &'a str,
    pub timestamp: &'a str,
    pub request_ip: Option<&'a str>,
    pub method: Option<&'a str>,
    pub token: Option<&'a str>,
    pub payload: Option<&'a Value>,
}

/// 计算 REST API 签名
/// 
/// # Arguments
/// * `data` - 签名数据
/// * `app_key` - 应用密钥
/// 
/// # Returns
/// SignResult 包含原始字符串和签名结果
pub fn compute_rest_sign(data: &RestSignData, app_key: &str) -> SignResult {
    let mut map_data = BTreeMap::from([
        ("client_id", data.client_id),
        ("version", data.version),
        ("timestamp", data.timestamp),
    ]);
    if let Some(request_ip) = data.request_ip {
        map_data.insert("request_ip", request_ip);
    }
    if let Some(method) = data.method {
        map_data.insert("method", method);
    }
    if let Some(token) = data.token {
        map_data.insert("token", token);
    }
    
    let mut encoded = &mut form_urlencoded::Serializer::new(String::new());
    for (e0, e1) in map_data.into_iter() {
        encoded = encoded.append_pair(e0, e1)
    }
    let mut raw_string = encoded.finish();
    
    if let Some(body) = data.payload {
        raw_string += body.to_string().as_str();
    }
    
    let hash_data = format!("{}{}", raw_string, app_key);
    let digest = md5::compute(hash_data.as_bytes());
    
    SignResult {
        raw_string,
        signature: format!("{:x}", digest),
    }
}
