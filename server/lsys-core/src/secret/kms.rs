use async_trait::async_trait;

use super::error::SecretError;

/// KMS 解密器 trait。
///
/// 针对不同 KMS 服务（AWS KMS、阿里云 KMS 等）各自实现此 trait，
/// 然后在 [`SecretManagerBuilder::kms_provider`] 中按名称注册。
/// [`SecretManagerBuilder::build`] 时会自动调用对应解密器将密文还原为明文密钥。
#[async_trait]
pub trait KmsDecryptor: Send + Sync {
    /// 将密文字节解密为明文密钥字节。
    async fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, SecretError>;
}
