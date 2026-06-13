//! 腾讯云 KMS 解密器实现
//!
//! 封装腾讯云密钥管理服务(Key Management Service)的解密功能。
//! 使用腾讯云 TC3-HMAC-SHA256 签名方案。
//!
//! # 配置示例
//!
//! ```toml
//! [kms_tencent]
//! secret_id  = "AKID..."
//! secret_key = "secret..."
//! region     = "ap-beijing"
//!
//! [secret.my_key]
//! source     = "kms"
//! kms        = "tencent"
//! ciphertext = "base64:..." # 来自腾讯云 KMS Encrypt 的 CiphertextBlob（base64 解码后存储）
//! ```

use async_trait::async_trait;
use base64::Engine;
use std::time::{SystemTime, UNIX_EPOCH};

use lsys_core::secret::{KmsDecryptor, SecretError};

/// 腾讯云 KMS 解密器
pub struct TencentKmsDecryptor {
    secret_id: String,
    secret_key: String,
    region: String,
    client: reqwest::Client,
}

impl TencentKmsDecryptor {
    /// 使用默认 HTTP 客户端创建腾讯云 KMS 解密器。
    pub fn new(
        secret_id: impl Into<String>,
        secret_key: impl Into<String>,
        region: impl Into<String>,
    ) -> Self {
        TencentKmsDecryptor {
            secret_id: secret_id.into(),
            secret_key: secret_key.into(),
            region: region.into(),
            client: reqwest::Client::new(),
        }
    }

    /// 使用自定义 HTTP 客户端创建腾讯云 KMS 解密器。
    pub fn with_client(
        secret_id: impl Into<String>,
        secret_key: impl Into<String>,
        region: impl Into<String>,
        client: reqwest::Client,
    ) -> Self {
        TencentKmsDecryptor {
            secret_id: secret_id.into(),
            secret_key: secret_key.into(),
            region: region.into(),
            client,
        }
    }

    /// 返回 KMS API 端点。
    pub fn get_endpoint(&self) -> &'static str {
        "https://kms.tencentcloudapi.com"
    }
}

/// 将字节切片转为十六进制小写字符串。
fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Unix 秒数转 "YYYY-MM-DD"（UTC）。
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
impl KmsDecryptor for TencentKmsDecryptor {
    /// 调用腾讯云 KMS Decrypt 接口解密密文（TC3-HMAC-SHA256 签名）。
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

        let date = unix_secs_to_date(timestamp);
        let host = "kms.tencentcloudapi.com";
        let service = "kms";
        let action = "Decrypt";
        let version = "2019-01-18";
        let algorithm = "TC3-HMAC-SHA256";
        let content_type = "application/json; charset=utf-8";

        let body = serde_json::json!({ "CiphertextBlob": ciphertext_b64 }).to_string();

        // ── Step 1: 构造规范请求 ─────────────────────────────────────────────────
        // 规范头部：按字典序排列，头部名全小写，每条头部以 \n 结尾
        let canonical_headers = format!(
            "content-type:{}\nhost:{}\n",
            content_type, host
        );
        let signed_headers = "content-type;host";
        let hashed_payload = to_hex(&hmac_sha256::Hash::hash(body.as_bytes()));
        // canonical_headers 末尾已含 \n，直接拼接 signed_headers（无需额外 \n 分隔）
        let canonical_request = format!(
            "POST\n/\n\n{}{}\n{}",
            canonical_headers, signed_headers, hashed_payload
        );

        // ── Step 2: 构造待签名字符串 ─────────────────────────────────────────────
        let credential_scope = format!("{}/{}/tc3_request", date, service);
        let hashed_canonical =
            to_hex(&hmac_sha256::Hash::hash(canonical_request.as_bytes()));
        let string_to_sign = format!(
            "{}\n{}\n{}\n{}",
            algorithm, timestamp, credential_scope, hashed_canonical
        );

        // ── Step 3: 派生签名密钥（TC3 密钥派生链） ──────────────────────────────
        let secret_date = hmac_sha256::HMAC::mac(
            date.as_bytes(),
            format!("TC3{}", self.secret_key).as_bytes(),
        );
        let secret_service = hmac_sha256::HMAC::mac(service.as_bytes(), secret_date);
        let secret_signing = hmac_sha256::HMAC::mac(b"tc3_request", secret_service);
        let signature =
            to_hex(&hmac_sha256::HMAC::mac(string_to_sign.as_bytes(), secret_signing));

        // ── Step 4: 组装 Authorization 头 ────────────────────────────────────────
        let authorization = format!(
            "{} Credential={}/{}, SignedHeaders={}, Signature={}",
            algorithm, self.secret_id, credential_scope, signed_headers, signature
        );

        let response = self
            .client
            .post(self.get_endpoint())
            .header("Authorization", &authorization)
            .header("Content-Type", content_type)
            .header("Host", host)
            .header("X-TC-Action", action)
            .header("X-TC-Timestamp", timestamp.to_string())
            .header("X-TC-Version", version)
            .header("X-TC-Region", &self.region)
            .body(body)
            .send()
            .await
            .map_err(|e| {
                SecretError::KmsDecrypt(format!("Tencent KMS request failed: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(SecretError::KmsDecrypt(format!(
                "Tencent KMS error: {} - {}",
                status, text
            )));
        }

        let resp: serde_json::Value = response.json().await.map_err(|e| {
            SecretError::KmsDecrypt(format!("Failed to parse Tencent KMS response: {}", e))
        })?;

        // 检查 API 级别错误（腾讯云错误放在 Response.Error 中）
        if let Some(err) = resp.get("Response").and_then(|r| r.get("Error")) {
            let code = err.get("Code").and_then(|v| v.as_str()).unwrap_or("unknown");
            let msg = err
                .get("Message")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            return Err(SecretError::KmsDecrypt(format!(
                "Tencent KMS API error: {} - {}",
                code, msg
            )));
        }

        // 腾讯云正常响应在 Response.Plaintext
        let plaintext_b64 = resp
            .get("Response")
            .and_then(|r| r.get("Plaintext"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                SecretError::KmsDecrypt(
                    "Tencent KMS response missing Response.Plaintext field".to_string(),
                )
            })?;

        base64::engine::general_purpose::STANDARD
            .decode(plaintext_b64)
            .map_err(|e| {
                SecretError::KmsDecrypt(format!(
                    "Tencent KMS: base64 decode of plaintext failed: {}",
                    e
                ))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tencent_kms_endpoint() {
        let d = TencentKmsDecryptor::new("id", "key", "ap-beijing");
        assert_eq!(d.get_endpoint(), "https://kms.tencentcloudapi.com");
    }

    #[test]
    fn test_tencent_region_support() {
        for region in ["ap-beijing", "ap-shanghai", "ap-hongkong"] {
            let _d = TencentKmsDecryptor::new("id", "key", region);
        }
    }

    #[test]
    fn test_to_hex() {
        assert_eq!(to_hex(&[0x00, 0xff, 0xab]), "00ffab");
    }

    #[test]
    fn test_unix_secs_to_date() {
        // 2024-01-01T00:00:00Z = 1704067200
        assert_eq!(unix_secs_to_date(1704067200), "2024-01-01");
    }
}

