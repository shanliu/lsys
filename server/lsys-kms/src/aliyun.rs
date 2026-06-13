//! 阿里云 KMS 解密器实现
//!
//! 封装阿里云密钥管理服务(Key Management Service)的解密功能。
//! 使用阿里云 RPC 签名方案 v1（HMAC-SHA256）。
//!
//! # 配置示例
//!
//! ```toml
//! [kms_aliyun]
//! access_key_id     = "AKIA..."
//! access_key_secret = "secret..."
//! region            = "cn-beijing"
//!
//! [secret.my_key]
//! source     = "kms"
//! kms        = "aliyun"
//! ciphertext = "base64:..." # 来自阿里云 KMS Encrypt 的 CiphertextBlob（base64 解码后存储）
//! ```

use async_trait::async_trait;
use base64::Engine;
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

use lsys_core::secret::{KmsDecryptor, SecretError};

/// 阿里云 KMS 解密器
pub struct AliyunKmsDecryptor {
    access_key_id: String,
    access_key_secret: String,
    region: String,
    client: reqwest::Client,
}

impl AliyunKmsDecryptor {
    /// 使用默认 HTTP 客户端创建阿里云 KMS 解密器。
    pub fn new(
        access_key_id: impl Into<String>,
        access_key_secret: impl Into<String>,
        region: impl Into<String>,
    ) -> Self {
        AliyunKmsDecryptor {
            access_key_id: access_key_id.into(),
            access_key_secret: access_key_secret.into(),
            region: region.into(),
            client: reqwest::Client::new(),
        }
    }

    /// 使用自定义 HTTP 客户端创建阿里云 KMS 解密器。
    pub fn with_client(
        access_key_id: impl Into<String>,
        access_key_secret: impl Into<String>,
        region: impl Into<String>,
        client: reqwest::Client,
    ) -> Self {
        AliyunKmsDecryptor {
            access_key_id: access_key_id.into(),
            access_key_secret: access_key_secret.into(),
            region: region.into(),
            client,
        }
    }

    /// 返回 KMS API 端点。
    pub fn get_endpoint(&self) -> String {
        format!("https://kms.{}.aliyuncs.com", self.region)
    }
}

/// RFC 3986 percent-encode（阿里云签名要求：只保留 A-Z a-z 0-9 - _ . ~）。
fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for &b in s.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => {
                out.push('%');
                out.push(char::from_digit((b >> 4) as u32, 16).unwrap().to_ascii_uppercase());
                out.push(char::from_digit((b & 0xf) as u32, 16).unwrap().to_ascii_uppercase());
            }
        }
    }
    out
}

/// Unix 秒数转 ISO 8601 UTC 字符串，例如 "2024-01-01T00:00:00Z"。
fn format_iso8601(secs: u64) -> String {
    let date = unix_secs_to_date(secs);
    let h = (secs % 86400) / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    format!("{}T{:02}:{:02}:{:02}Z", date, h, m, s)
}

/// Unix 秒数转 "YYYY-MM-DD"（Howard Hinnant civil_from_days 算法）。
fn unix_secs_to_date(secs: u64) -> String {
    let z = (secs / 86400) as i64 + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{:04}-{:02}-{:02}", y, m, d)
}

#[async_trait]
impl KmsDecryptor for AliyunKmsDecryptor {
    /// 调用阿里云 KMS 2016-01-20 Decrypt 接口解密密文。
    ///
    /// `ciphertext` 为 KMS Encrypt 接口返回的 CiphertextBlob 经 base64 解码后的原始字节。
    async fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, SecretError> {
        // CiphertextBlob 需要以 base64 字符串传递给 API
        let ciphertext_b64 =
            base64::engine::general_purpose::STANDARD.encode(ciphertext);

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| SecretError::KmsDecrypt(format!("clock error: {}", e)))?
            .as_secs();

        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos();

        // 参数 map 使用 BTreeMap 以保证字典序排列
        let mut params: BTreeMap<&str, String> = BTreeMap::new();
        params.insert("Action", "Decrypt".to_string());
        params.insert("Format", "JSON".to_string());
        params.insert("Version", "2016-01-20".to_string());
        params.insert("AccessKeyId", self.access_key_id.clone());
        params.insert("SignatureMethod", "HMAC-SHA256".to_string());
        params.insert("SignatureNonce", format!("{}{}", timestamp, nanos));
        params.insert("SignatureVersion", "1.0".to_string());
        params.insert("Timestamp", format_iso8601(timestamp));
        params.insert("CiphertextBlob", ciphertext_b64);

        // 构造待签名查询字符串（所有 key/value 均需 percent-encode）
        let query_string: String = params
            .iter()
            .map(|(k, v)| format!("{}={}", percent_encode(k), percent_encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        // 待签名字符串：METHOD + "&" + encode("/") + "&" + encode(query)
        let string_to_sign = format!("POST&%2F&{}", percent_encode(&query_string));

        // 签名密钥：AccessKeySecret + "&"
        let signing_key = format!("{}&", self.access_key_secret);
        let sig_bytes =
            hmac_sha256::HMAC::mac(string_to_sign.as_bytes(), signing_key.as_bytes());
        let signature =
            base64::engine::general_purpose::STANDARD.encode(sig_bytes);

        // 将签名追加到请求体
        let body = format!(
            "{}&Signature={}",
            query_string,
            percent_encode(&signature)
        );

        let response = self
            .client
            .post(self.get_endpoint())
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .map_err(|e| {
                SecretError::KmsDecrypt(format!("Aliyun KMS request failed: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(SecretError::KmsDecrypt(format!(
                "Aliyun KMS error: {} - {}",
                status, text
            )));
        }

        let resp: serde_json::Value = response.json().await.map_err(|e| {
            SecretError::KmsDecrypt(format!("Failed to parse Aliyun KMS response: {}", e))
        })?;

        // 检查 API 级别错误
        if let Some(code) = resp.get("Code").and_then(|v| v.as_str()) {
            let msg = resp
                .get("Message")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            return Err(SecretError::KmsDecrypt(format!(
                "Aliyun KMS API error: {} - {}",
                code, msg
            )));
        }

        let plaintext_b64 = resp
            .get("Plaintext")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                SecretError::KmsDecrypt(
                    "Aliyun KMS response missing Plaintext field".to_string(),
                )
            })?;

        base64::engine::general_purpose::STANDARD
            .decode(plaintext_b64)
            .map_err(|e| {
                SecretError::KmsDecrypt(format!(
                    "Aliyun KMS: base64 decode of plaintext failed: {}",
                    e
                ))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aliyun_kms_endpoint() {
        let cases = [
            ("cn-beijing", "https://kms.cn-beijing.aliyuncs.com"),
            ("cn-shanghai", "https://kms.cn-shanghai.aliyuncs.com"),
            ("ap-southeast-1", "https://kms.ap-southeast-1.aliyuncs.com"),
        ];
        for (region, expected) in cases {
            let d = AliyunKmsDecryptor::new("id", "secret", region);
            assert_eq!(d.get_endpoint(), expected);
        }
    }

    #[test]
    fn test_percent_encode() {
        assert_eq!(percent_encode("hello"), "hello");
        assert_eq!(percent_encode("hello world"), "hello%20world");
        assert_eq!(percent_encode("a=b+c"), "a%3Db%2Bc");
        assert_eq!(percent_encode("A-Z_0~9."), "A-Z_0~9.");
    }

    #[test]
    fn test_format_iso8601() {
        // 2024-01-01T00:00:00Z is Unix 1704067200
        assert_eq!(format_iso8601(1704067200), "2024-01-01T00:00:00Z");
    }
}


