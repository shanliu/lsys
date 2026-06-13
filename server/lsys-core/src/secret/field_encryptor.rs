use std::sync::Arc;

use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, AeadCore, KeyInit, OsRng},
};
use base64::Engine;

use super::{SecretError, SecretManager};

/// 数据库字段透明加密/解密组件（AES-256-GCM）。
///
/// - 加密模式：加密输出格式为 `base64(12-byte-nonce || ciphertext)`
/// - 明文模式：直接返回原文，不进行加密
///
/// # 示例
///
/// ```rust,ignore
/// // 加密模式（生产环境）
/// let encryptor = Arc::new(FieldEncryptor::new(secret_manager.clone(), "smtp_key", true));
/// let cipher = encryptor.encrypt_str("my-secret-password")?;
/// let plain  = encryptor.decrypt_str(&cipher)?;
/// assert_eq!(plain, "my-secret-password");
///
/// // 明文模式（开发/测试环境）
/// let encryptor = Arc::new(FieldEncryptor::new(secret_manager.clone(), "smtp_key", false));
/// let plain = encryptor.encrypt_str("my-secret-password")?;
/// assert_eq!(plain, "my-secret-password"); // 直接返回原文
/// ```
pub struct FieldEncryptor {
    secret: Arc<SecretManager>,
    key_id: String,
    enable_encryption: bool,
}

impl FieldEncryptor {
    /// 创建一个 `FieldEncryptor`，使用 `SecretManager` 中 `key_id` 对应的 32 字节密钥。
    ///
    /// # 参数
    /// - `secret`: SecretManager 实例
    /// - `key_id`: 密钥标识符
    /// - `enable_encryption`: 是否启用加密
    ///   - `true`: 加密模式，使用 AES-256-GCM 加密数据（生产环境）
    ///   - `false`: 明文模式，直接存储原文（开发/测试环境）
    pub fn new(secret: Arc<SecretManager>, key_id: impl Into<String>, enable_encryption: bool) -> Self {
        Self {
            secret,
            key_id: key_id.into(),
            enable_encryption,
        }
    }

    /// 加密一个字符串。
    ///
    /// - 加密模式（`enable_encryption = true`）：返回 `base64` 格式的密文
    /// - 明文模式（`enable_encryption = false`）：直接返回原文
    pub fn encrypt_str(&self, plaintext: &str) -> Result<String, SecretError> {
        if !self.enable_encryption {
            // 明文模式：直接返回原文
            return Ok(plaintext.to_string());
        }

        // 加密模式：执行 AES-256-GCM 加密
        let key_bytes = self.secret.require(&self.key_id)?;
        if key_bytes.len() != 32 {
            return Err(SecretError::Encrypt(format!(
                "key '{}' must be 32 bytes for AES-256-GCM, got {}",
                self.key_id,
                key_bytes.len()
            )));
        }
        let key = Key::<Aes256Gcm>::from_slice(key_bytes);
        let cipher = Aes256Gcm::new(key);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = cipher
            .encrypt(&nonce, plaintext.as_bytes())
            .map_err(|e| SecretError::Encrypt(format!("AES-GCM encrypt failed: {}", e)))?;

        let mut combined = nonce.to_vec();
        combined.extend_from_slice(&ciphertext);

        let encoded = base64::engine::general_purpose::STANDARD.encode(&combined);
        Ok(encoded)
    }

    /// 解密一个字符串。
    ///
    /// 自动检测数据格式：
    /// - 如果能成功 base64 解码，视为加密数据并执行 AES-256-GCM 解密
    /// - 如果 base64 解码失败，视为明文直接返回
    pub fn decrypt_str(&self, encoded: &str) -> Result<String, SecretError> {
        // 尝试 base64 解码，如果失败则认为是明文
        let combined = match base64::engine::general_purpose::STANDARD.decode(encoded) {
            Ok(data) => data,
            Err(_) => {
                // 解码失败，直接返回原文（明文模式）
                return Ok(encoded.to_string());
            }
        };

        // 加密数据：执行 AES-256-GCM 解密
        if combined.len() < 12 {
            return Err(SecretError::Encrypt(
                "ciphertext is too short (expected nonce + ciphertext)".to_string(),
            ));
        }

        let (nonce_bytes, ciphertext) = combined.split_at(12);

        let key_bytes = self.secret.require(&self.key_id)?;
        if key_bytes.len() != 32 {
            return Err(SecretError::Encrypt(format!(
                "key '{}' must be 32 bytes for AES-256-GCM, got {}",
                self.key_id,
                key_bytes.len()
            )));
        }
        let key = Key::<Aes256Gcm>::from_slice(key_bytes);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext_bytes = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| SecretError::Encrypt(format!("AES-GCM decrypt failed: {}", e)))?;

        String::from_utf8(plaintext_bytes)
            .map_err(|e| SecretError::Encrypt(format!("UTF-8 decode failed: {}", e)))
    }

    /// 计算字符串的 HMAC-SHA256 哈希值并以十六进制（hex）形式返回。
    ///
    /// 注意：哈希功能不受 `enable_encryption` 影响，始终执行相同的哈希计算。
    pub fn hash_str(&self, plaintext: &str) -> Result<String, SecretError> {
        let key_bytes = self.secret.require(&self.key_id)?;
        if key_bytes.len() != 32 {
            return Err(SecretError::Encrypt(format!(
                "key '{}' must be 32 bytes for HMAC-SHA256, got {}",
                self.key_id,
                key_bytes.len()
            )));
        }
        use hmac::digest::KeyInit;
        use sha2::Sha256;
        
        type HmacSha256 = hmac::Hmac<Sha256>;
        let mut mac = <HmacSha256 as KeyInit>::new_from_slice(key_bytes)
            .map_err(|e| SecretError::Encrypt(format!("HMAC key initialization failed: {}", e)))?;
        
        use hmac::Mac;
        mac.update(plaintext.as_bytes());
        let result = mac.finalize();
        Ok(hex::encode(result.into_bytes()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_encryptor() -> FieldEncryptor {
        // 创建一个 32 字节的测试密钥
        let test_key: Vec<u8> = (0..32).collect();
        let mut keys = HashMap::new();
        keys.insert("test_key".to_string(), test_key);
        
        let secret_manager = SecretManager { keys };
        // 默认使用加密模式进行测试
        FieldEncryptor::new(Arc::new(secret_manager), "test_key", true)
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let encryptor = create_test_encryptor();
        let plaintext = "test@example.com";
        
        let encrypted = encryptor.encrypt_str(plaintext).expect("encryption should succeed");
        let decrypted = encryptor.decrypt_str(&encrypted).expect("decryption should succeed");
        
        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_hash_consistency() {
        let encryptor = create_test_encryptor();
        let plaintext = "test@example.com";
        
        let hash1 = encryptor.hash_str(plaintext).expect("hashing should succeed");
        let hash2 = encryptor.hash_str(plaintext).expect("hashing should succeed");
        
        // 同一个明文应该产生相同的哈希值
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA256 输出为 32 字节，hex 编码后为 64 字符
    }

    #[test]
    fn test_hash_uniqueness() {
        let encryptor = create_test_encryptor();
        
        let hash1 = encryptor.hash_str("test1@example.com").expect("hashing should succeed");
        let hash2 = encryptor.hash_str("test2@example.com").expect("hashing should succeed");
        
        // 不同的明文应该产生不同的哈希值
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_encrypt_different_nonce() {
        let encryptor = create_test_encryptor();
        let plaintext = "test@example.com";
        
        let encrypted1 = encryptor.encrypt_str(plaintext).expect("encryption should succeed");
        let encrypted2 = encryptor.encrypt_str(plaintext).expect("encryption should succeed");
        
        // 相同明文每次加密应该产生不同的密文（因为 nonce 不同）
        assert_ne!(encrypted1, encrypted2);
        
        // 但都应该能正确解密
        let decrypted1 = encryptor.decrypt_str(&encrypted1).expect("decryption should succeed");
        let decrypted2 = encryptor.decrypt_str(&encrypted2).expect("decryption should succeed");
        assert_eq!(plaintext, decrypted1);
        assert_eq!(plaintext, decrypted2);
    }
}
